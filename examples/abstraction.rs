use reanki::{Deck, Field, Model, Note, Template};
use std::{fs::File, sync::Arc};

struct MyCard {
    some_field: String,
    another_field: String,
}

impl MyCard {
    fn template() -> Arc<Template> {
        Arc::new(Template::new(
            3,
            "reanki-template".to_string(),
            "q: {{some_field}}".to_string(),
            "a: {{another_field}}".to_string(),
        ))
    }

    fn css() -> String {
        "
#answer {
    font-size: 1.5rem;
}
"
        .to_string()
    }

    // keep in sync with `into_field_values`
    fn fields() -> Vec<Field> {
        vec![
            Field::new("some_field".to_string()),
            Field::new("another_field".to_string()),
        ]
    }

    // keep in sync with `fields`
    fn into_field_values(self) -> Vec<String> {
        vec![self.some_field, self.another_field]
    }
}

fn main() {
    // create deck, model(s), template(s)
    let mut deck = Deck::new(1, "My deck".to_string(), "My reanki deck".to_string());
    let model = Arc::new(Model::new(
        2,
        "My model 2".to_string(),
        MyCard::fields(),
        0,
        MyCard::css(),
        reanki::ModelType::Standard,
    ));
    let template = MyCard::template();

    // create notes
    deck.add_note(Note::new(
        "my-note-6".to_string(),
        model,
        vec![template],
        MyCard {
            some_field: "some value".to_string(),
            another_field: "another value".to_string(),
        }
        .into_field_values(),
    ));

    let out = File::create("./test_deck.apkg").unwrap();
    deck.write(out).unwrap();
}
