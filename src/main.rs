use cursive::{
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, NamedView, TextArea, TextView},
    Cursive,
};

fn create_chat_history() -> NamedView<LinearLayout> {
    let mut chat_history = LinearLayout::vertical();

    chat_history.add_child(TextView::new("Hello, World!"));

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

    window.call_on_name("chat_history", |view: &mut LinearLayout| {
        view.add_child(TextView::new(input));
    });
}

fn main() {
    let mut main_window = cursive::default();
    main_window
        .load_toml(include_str!("./assets/theme.toml"))
        .unwrap();

    let mut vertical_chat = LinearLayout::vertical();

    vertical_chat.add_child(create_chat_layout());

    main_window.add_layer(
        Dialog::around(vertical_chat)
            .title("OpenAI Chat")
            .button("Submit", submit_input)
            .button("Quit", |_window| _window.quit()),
    );

    main_window.run();
}
