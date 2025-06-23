mod schema;

use diesel::{ConnectionError, SqliteConnection, prelude::*};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use serde_json::{Map, Value};
use std::{
    collections::{HashMap, hash_map::Entry},
    io::{Seek, Write},
    sync::Arc,
    time::UNIX_EPOCH,
};
use thiserror::Error;
use zip::{ZipWriter, write::SimpleFileOptions};

const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

/// reanki error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Zip error: {message}. Caused by: {source}.")]
    Zip {
        message: &'static str,
        source: zip::result::ZipError,
    },
    #[error("I/O error: {message}. Caused by: {source}")]
    Io {
        message: &'static str,
        source: std::io::Error,
    },
    #[error("Database connection error: {message}. Caused by: {source}")]
    DieselConn {
        message: &'static str,
        source: ConnectionError,
    },
    #[error("Database error: {message}. Caused by: {source}")]
    Diesel {
        message: &'static str,
        source: diesel::result::Error,
    },
    #[error("Error: {message}. Caused by: {source}")]
    Generic {
        message: &'static str,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Diesel error")]
    DieselFrom(#[from] diesel::result::Error),
}

macro_rules! error {
    ($error:tt :: $variant:tt , $message:literal) => {
        |source| $error::$variant {
            message: $message,
            source,
        }
    };
}

// deck config json object
#[derive(Debug)]
struct Dconf {
    deck_id: i64,
}

impl Dconf {
    fn new(deck_id: i64) -> Self {
        Self { deck_id }
    }

    fn to_anki_json(&self, dconf_timestamp: i64) -> Value {
        serde_json::json!({
            dconf_timestamp.to_string(): {
            //self.deck_id.to_string(): {
                // deck id
                "id": self.deck_id,
                // modified timestamp
                "mod": dconf_timestamp,
                // deck config name
                "name": "reanki-dconf",
                // "update sequence number"
                "usn": 0,
                // timer max
                "maxTaken": 0,
                // autoplay audio
                "autoplay": false,
                // timer, hide = 0, show = 1
                "timer": 0,
                // new card conf
                "new": {},
                // review card conf
                "rev": {},
                // lapsed card conf
                "lapse": {},
            }
        })
    }
}

/// Anki note field.
#[derive(Debug)]
pub struct Field {
    name: String,
    font: Option<String>,
    size: Option<i64>,
    rtl: bool,
}

impl Field {
    /// Create a new Field with the default font and size and rtl set to false.
    pub fn new(name: String) -> Self {
        Self {
            name,
            font: None,
            size: None,
            rtl: false,
        }
    }

    /// Set the field font.
    pub fn font(mut self, font: String) -> Self {
        self.font = Some(font);
        self
    }

    /// Set the field font size.
    pub fn size(mut self, size: i64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the field rtl (right-to-left). Set this to true for right-to-left languages like arabic.
    pub fn rtl(mut self, rtl: bool) -> Self {
        self.rtl = rtl;
        self
    }

    fn to_anki_json(&self) -> Value {
        serde_json::json!({
            // field name
            "name": self.name,
            // not sure what this does...
            "sticky": false,
            // right-to-left
            "rtl": false,
            // field text font
            "font": self.font.as_deref().unwrap_or("Arial"),
            // field text font size
            "size": self.size.unwrap_or(20),
        })
    }
}

/// Anki card template.
#[derive(Debug)]
pub struct Template {
    id: i64,
    name: String,
    qfmt: String,
    afmt: String,
}

impl Template {
    /// Creates a new Template.
    pub fn new(id: i64, name: String, question_template: String, answer_template: String) -> Self {
        Self {
            id,
            name,
            qfmt: question_template,
            afmt: answer_template,
        }
    }

    fn to_anki_json(&self) -> Value {
        serde_json::json!({
            // template name
            "name": self.name,
            // question
            "qfmt": self.qfmt,
            // answer
            "afmt": self.afmt,
        })
    }
}

/// Anki model type. Currently only standard type cards are supported.
#[derive(Debug)]
pub enum ModelType {
    /// Standard model format with a question on the front and an answer in the back.
    Standard,
    // Cloze,
}

impl ModelType {
    fn to_anki_json_format(&self) -> i64 {
        match self {
            Self::Standard => 0,
            // Self::Cloze => 1,
        }
    }
}

/// Anki note model.
#[derive(Debug)]
pub struct Model {
    id: i64,
    name: String,
    fields: Vec<Field>,
    sort_field: i64,
    css: String,
    model_type: ModelType,
}

impl Model {
    /// Creates a new Model.
    pub fn new(
        id: i64,
        name: String,
        fields: Vec<Field>,
        sort_field: i64,
        css: String,
        model_type: ModelType,
    ) -> Self {
        Self {
            id,
            name,
            fields,
            sort_field,
            css,
            model_type,
        }
    }

    fn to_anki_json<'a, I: Iterator<Item = &'a Template>>(
        &self,
        deck_id: i64,
        templates: I,
        model_timestamp: i64,
    ) -> Value {
        let fields = self
            .fields
            .iter()
            .map(Field::to_anki_json)
            .collect::<Vec<_>>();
        let templates = templates.map(|t| t.to_anki_json()).collect::<Vec<_>>();

        serde_json::json!({
            // model id
            "id": self.id,
            // model name
            "name": self.name,
            // model type, standard or cloze
            "type": self.model_type.to_anki_json_format(),
            // modified timestamp
            "mod": model_timestamp,
            // "update sequence number"
            "usn": 0,
            // sort field index
            "sortf": self.sort_field,
            // deck id
            "did": deck_id,
            // templates json array
            "tmpls": templates,
            // fields json array
            "flds": fields,
            // CSS
            "css": self.css,
        })
    }
}

#[derive(Debug)]
/// An Anki card.
struct Card;

impl Card {
    fn write_to_db(
        template_ord: i64,
        card_ord: u16,
        note_id: i64,
        model: &Model,
        deck: &Deck,
        conn: &mut SqliteConnection,
        card_id_timestamp: i64,
    ) -> Result<(), Error> {
        use schema::cards;

        let card_ord: i64 = card_ord.into();
        diesel::insert_into(cards::table)
            .values((
                // doesn't seem to be used for anything except the created at timestamp
                cards::id.eq(card_id_timestamp),
                // note id
                cards::nid.eq(note_id),
                // deck id
                cards::did.eq(deck.id),
                // template index in the model json
                cards::ord.eq(template_ord),
                // modified timestamp
                cards::mod_.eq(card_id_timestamp),
                // "update sequence number"
                cards::usn.eq(0),
                // model type
                cards::type_.eq(model.model_type.to_anki_json_format()),
                // card position in the queue, 0 = new
                cards::queue.eq(0),
                // for new cards, the order in which cards are studied starting from 1
                cards::due.eq(card_ord),
                // interval...
                cards::ivl.eq(0),
                // ease factor
                cards::factor.eq(0),
                // number of reviews
                cards::reps.eq(0),
                // number of lapses
                cards::lapses.eq(0),
                // reps left today and reps left until graduation
                cards::left.eq(0),
                // original due for filtered decks
                cards::odue.eq(0),
                // original deck id for filtered decks
                cards::odid.eq(0),
                // flag, none = 0, red = 1, orange = 2, green = 3, blue = 4
                cards::flags.eq(0),
                // unused
                cards::data.eq(""),
            ))
            .execute(conn)
            .map_err(error!(Error::Diesel, "Failed to insert into cards"))?;

        Ok(())
    }
}

/// An Anki note.
#[derive(Debug)]
pub struct Note {
    guid: String,
    model: Arc<Model>,
    tags: Option<Vec<String>>,
    templates: Vec<Arc<Template>>,
    field_values: Vec<String>,
    card_ord: u16,
}

impl Note {
    /// Create a new Anki note.
    ///  The `guid` should be unique and should not change.
    pub fn new(
        guid: String,
        model: Arc<Model>,
        templates: Vec<Arc<Template>>,
        field_values: Vec<String>,
    ) -> Self {
        Self {
            guid,
            model,
            tags: None,
            templates,
            field_values,
            card_ord: 1,
        }
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    // The order in which new cards of this note appear, starting from 1.
    pub fn order(mut self, ord: u16) -> Self {
        self.card_ord = ord;
        self
    }

    // writes the note into a sqlite db
    fn write_to_db(
        &self,
        deck: &Deck,
        templates: &HashMap<i64, (i64, Arc<Template>)>,
        conn: &mut SqliteConnection,
        timestamp_secs: i64,
        card_id_timestamp: &mut i64,
    ) -> Result<(), Error> {
        use schema::notes;

        let fields = self.field_values.join("\x1f");
        let tags = self.tags.as_ref().map(|t| t.join(" ")).unwrap_or_default();

        let note_id = diesel::insert_into(notes::table)
            .values((
                notes::guid.eq(&self.guid),
                notes::mid.eq(self.model.id),
                notes::mod_.eq(timestamp_secs),
                notes::usn.eq(0),
                notes::tags.eq(tags),
                notes::flds.eq(fields),
                notes::sfld.eq(0),
                notes::csum.eq(0),
                notes::flags.eq(0),
                notes::data.eq(""),
            ))
            .returning(notes::id)
            .get_result::<Option<i64>>(conn)
            .map_err(error!(Error::Diesel, "Failed to insert note"))?
            .expect("Did not receive an id back from the database after an insert");

        for template in &self.templates {
            let template_ord = templates.get(&template.id).map(|t| t.0).unwrap_or_default();
            *card_id_timestamp += 1;
            Card::write_to_db(
                template_ord,
                self.card_ord,
                note_id,
                &self.model,
                deck,
                conn,
                *card_id_timestamp,
            )?;
        }

        Ok(())
    }
}

pub type TemplateMap = HashMap<i64, (i64, Arc<Template>)>;

/// Anki deck, a collection of notes.
#[derive(Debug)]
pub struct Deck {
    id: i64,
    name: String,
    description: String,
    notes: Vec<Note>,
    // model id => template id => (template ord, template)
    model_to_templates: HashMap<i64, (Arc<Model>, TemplateMap)>,
}

impl Deck {
    /// Create a new deck. Note that the deck id 1 is special and corresponds to the default deck.
    pub fn new(id: i64, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            notes: Vec::new(),
            model_to_templates: HashMap::new(),
        }
    }

    /// Add a note to the deck.
    pub fn add_note(&mut self, note: Note) {
        let (_model, template_map) = self
            .model_to_templates
            .entry(note.model.id)
            .or_insert_with(|| (note.model.clone(), HashMap::new()));

        let mut new_template_ord = template_map.len() as i64;
        for template in &note.templates {
            let entry = template_map.entry(template.id);
            if let Entry::Vacant(entry) = entry {
                entry.insert((new_template_ord, template.clone()));
                new_template_ord += 1;
            }
        }
        self.notes.push(note);
    }

    /// Write the deck into the writer in the apkg format.
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<(), Error> {
        // write to sqlite db
        let mut conn = SqliteConnection::establish(":memory:").map_err(error!(
            Error::DieselConn,
            "Failed to establish connection to in-memory sqlite database"
        ))?;

        conn.exclusive_transaction(move |tx| {
            tx.run_pending_migrations(MIGRATIONS).map_err({
                error!(
                    Error::Generic,
                    "Failed to run migrations for in-memory sqlite database"
                )
            })?;
            self.write_to_db(tx)?;
            Result::<(), Error>::Ok(())
        })?;
        let buf = conn.serialize_database_to_buffer();

        // write zip
        let mut zip = ZipWriter::new(writer);
        zip.start_file("collection.anki2", SimpleFileOptions::default())
            .map_err(error!(Error::Zip, "Failed to start file in zip archive"))?;
        zip.write_all(buf.as_slice()).map_err(error!(
            Error::Io,
            "Failed to write anki collection into zip"
        ))?;
        Ok(())
    }

    // writes the deck into a sqlite db
    fn write_to_db(&self, conn: &mut SqliteConnection) -> Result<(), Error> {
        let timestamp = UNIX_EPOCH.elapsed().unwrap();
        let timestamp_secs = timestamp.as_secs() as i64;
        let timestamp_millis = timestamp.as_millis() as i64;
        let mut card_id_timestamp = timestamp_millis;

        Col::write_to_db(self, conn, timestamp_secs, timestamp_millis)?;
        for note in &self.notes {
            let (_m, model_templates) = self.model_to_templates.get(&note.model.id).unwrap();
            note.write_to_db(
                self,
                model_templates,
                conn,
                timestamp_secs,
                &mut card_id_timestamp,
            )?;
        }
        Ok(())
    }

    fn to_value(&self, timestamp_millis: i64) -> Value {
        serde_json::json!(
            {
                self.id.to_string(): {
                    // deck id
                    "id": self.id,
                    // modified timestamp
                    "mod": timestamp_millis,
                    // deck name
                    "name": self.name,
                    // "update sequence number"
                    "usn": 0,
                    // unsure
                    "lrnToday": [0,0],
                    // unsure
                    "revToday": [0,0],
                    // unsure
                    "newToday": [0,0],
                    // unsure
                    "timeToday": [0,0],
                    // unsure
                    "collapsed": true,
                    // unsure
                    "browserCollapsed": true,
                    // deck description
                    "desc": self.description,
                    // unfiltered (standard) deck = 0, filtered deck = 1
                    "dyn": 0,
                    // the key of the corresponding deck config in the dconf JSON in the col table
                    "conf": timestamp_millis,
                    // custom study
                    "extendNew": 0,
                    // custom study
                    "extendRev": 0,
                }
            }
        )
    }
}

/// Anki collection.
struct Col;

impl Col {
    fn write_to_db(
        deck: &Deck,
        conn: &mut SqliteConnection,
        timestamp_secs: i64,
        timestamp_millis: i64,
    ) -> Result<(), Error> {
        use schema::col;

        let conf = serde_json::json!({});
        let models = deck
            .model_to_templates
            .values()
            .map(|(m, templates)| {
                (
                    m.id.to_string(),
                    m.to_anki_json(
                        deck.id,
                        templates.values().map(|(_ord, t)| t.as_ref()),
                        timestamp_millis,
                    ),
                )
            })
            .collect::<Map<_, _>>();
        let models = Value::Object(models);

        let dconf = Dconf::new(deck.id);
        let tags = "{}";

        diesel::insert_into(col::table)
            .values((
                // id, only one col
                col::id.eq(1),
                // creation timestamp
                col::crt.eq(timestamp_secs),
                // modification timestamp
                col::mod_.eq(timestamp_millis),
                // schema modification time
                col::scm.eq(timestamp_millis),
                // schema version or something
                col::ver.eq(11),
                // unused
                col::dty.eq(0),
                // "update sequence number"
                col::usn.eq(0),
                // last sync timestamp
                col::ls.eq(0),
                // various optional configurations
                col::conf.eq(conf.to_string()),
                // models json
                col::models.eq(models.to_string()),
                // decks json
                col::decks.eq(deck.to_value(timestamp_millis).to_string()),
                // deck config json
                col::dconf.eq(dconf.to_anki_json(timestamp_millis).to_string()),
                // tags, unsure
                col::tags.eq(tags),
            ))
            .execute(conn)
            .map_err(error!(Error::Diesel, "Failed to insert collection"))?;
        Ok(())
    }
}
