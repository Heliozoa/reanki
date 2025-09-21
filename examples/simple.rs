use reanki::{Deck, Field, Model, ModelType, Note, Template};
use std::{fs::File, sync::Arc};

fn main() -> Result<(), reanki::Error> {
    // Create a model and template.
    let css = "
#answer {
    font-size: 1.5rem;
}
"
    .to_string();
    // A model defines what fields our cards will have.
    let model = Arc::new(Model::new(
        1,
        "My model 1".to_string(),
        vec![
            Field::new("question-field".to_string()),
            Field::new("answer-field".to_string()),
        ],
        0,
        css,
        ModelType::Standard,
    ));
    // A template defines how those fields are displayed.
    let template = Arc::new(Template::new(
        2,
        "reanki-template".to_string(),
        "<div>{{question-field}}</div>".to_string(),
        "<div id=answer>{{answer-field}}</div>".to_string(),
    ));

    // Create a deck and notes.
    // A deck is just a collection of notes.
    let mut deck = Deck::new(3, "My deck".to_string(), "My reanki deck".to_string());
    // A note fills in the fields in our model with some information.
    // A card is a note that has been applied to a template.
    // For example, if we had two templates here, we would end up with two cards for this one note.
    let note = Note::new(
        "my-note-4".to_string(),
        model,
        vec![template],
        vec!["question-value".to_string(), "answer-value".to_string()],
    );
    deck.add_note(note);

    // write file
    let out = File::create("./test_deck.apkg").unwrap();
    deck.write(out)?;
    Ok(())
}
