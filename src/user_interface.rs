use crate::ai::{fetch_completion, HISTORY};

use cursive::{
    view::{Nameable, Resizable, Scrollable},
    views::{Dialog, DummyView, LinearLayout, NamedView, TextArea, TextView},
    Cursive, CursiveRunnable, event::{Event, Key},
};
use openai_api_rs::v1::chat_completion::ChatCompletionMessage;

/// Create a chat history view
fn create_chat_history() -> NamedView<LinearLayout> {
    let mut chat_history = LinearLayout::vertical();

    chat_history.with_name("chat_history")
}

/// Create a chat history view with a text area
fn create_chat_layout() -> LinearLayout {
    let mut chat_history = LinearLayout::vertical();

    chat_history.add_child(TextView::new("Start a conversation with OpenAI"));
    chat_history.add_child(DummyView);
    chat_history.add_child(create_chat_history());
    chat_history.add_child(DummyView);
    chat_history.add_child(
        TextArea::new()
            .with_name("openai_input_box")
            .min_size((10, 1)),
    );

    chat_history
}

/// Submit the input from the text area to the chat history
fn submit_input(window: &mut Cursive) -> () {
    let input = window
        .call_on_name("openai_input_box", |view: &mut TextArea| {
            let content = view.get_content().to_string();
            view.set_content("");
            content
        })
        .unwrap();

    let completion = match fetch_completion(&input) {
        Ok(completion) => completion,
        Err(error) => {
            println!("Error trying to fetch completion: {}", error);
            return;
        }
    };

    let first_completion = match completion.choices.first() {
        Some(completion) => completion,
        None => {
            println!("Error! Could not get first completion");
            return;
        }
    };

    let completion_text = match &first_completion.message.content {
        Some(text) => text.to_owned(),
        None => String::from("Error! Could not get completion text"),
    };

    window.call_on_name("chat_history", |view: &mut LinearLayout| {
        view.clear();

        for message in HISTORY.lock().unwrap().iter() {
            let message_text: String = format!("{:?}: {}", message.role, message.content);
            view.add_child(TextView::new(" "));
            view.add_child(TextView::new(message_text));
        }
        // blank line for spacing
        view.add_child(TextView::new(" "));
        view.add_child(TextView::new(format!(
            "{:?}: {}",
            first_completion.message.role, completion_text
        )));
    });

    HISTORY.lock().unwrap().push(ChatCompletionMessage {
        role: first_completion.message.role.clone(),
        content: completion_text,
        name: None,
        function_call: None,
    });
}

/// Create the main window for the application
pub fn create_main_window() -> CursiveRunnable {
    let mut main_window = cursive::default();
    main_window
        .load_toml(include_str!("./assets/theme.toml"))
        .unwrap();

    let mut vertical_chat = LinearLayout::vertical();

    vertical_chat.add_child(create_chat_layout().scrollable());

    main_window.add_layer(
        Dialog::around(vertical_chat)
            .title("OpenAI Chat")
            .button("Submit", submit_input)
            .button("Quit", |_window| _window.quit()),
    );

    main_window.add_global_callback(Event::Ctrl(Key::Enter), submit_input);

    return main_window;
}
