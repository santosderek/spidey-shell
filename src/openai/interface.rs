use super::api::{fetch_completion, HISTORY};

use copypasta::{ClipboardContext, ClipboardProvider};
use cursive::{
    align::HAlign,
    view::{Nameable, Resizable, Scrollable},
    views::{Button, Dialog, DummyView, LinearLayout, SelectView, TextArea, TextView},
    Cursive, View,
};

use openai_api_rs::v1::chat_completion::ChatCompletionMessage;

/// Create a chat history view with a text area
pub fn create_chat_layout() -> Dialog {
    let mut chat_history = LinearLayout::vertical();

    chat_history.add_child(TextView::new("Start a conversation with OpenAI"));
    chat_history.add_child(DummyView);
    chat_history.add_child(
        LinearLayout::vertical()
            .with_name("chat_history")
            .scrollable(),
    );
    chat_history.add_child(DummyView);
    chat_history.add_child(
        TextArea::new()
            .with_name("openai_input_box")
            .min_size((10, 1)),
    );

    let mut vertical_chat = LinearLayout::vertical();

    vertical_chat.add_child(chat_history.scrollable());

    let dialog = Dialog::around(vertical_chat)
        .title("OpenAI Chat")
        .button("Submit", submit_input)
        .button("Clear", |window| {
            window
                .call_on_name("openai_input_box", |view: &mut TextArea| {
                    view.set_content("");
                })
                .unwrap();
            window
                .call_on_name("chat_history", |view: &mut LinearLayout| {
                    view.clear();
                })
                .unwrap();
            HISTORY.write().unwrap().clear();
        })
        .button("Quit", |_window| {
            _window.pop_layer();
            _window.add_layer(crate::menu::create_main_menu());
        });

    return dialog;
}

/// Submit the input from the text area to the chat history
pub fn submit_input(window: &mut Cursive) {
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

    HISTORY.write().unwrap().push(ChatCompletionMessage {
        role: first_completion.message.role.clone(),
        content: completion_text,
        name: None,
        function_call: None,
    });

    fill_window_with_history(window);
}

fn copy_to_clipboard<'a>(_window: &'a mut Cursive, text: &String) -> () {
    let mut clipboard = match ClipboardContext::new() {
        Ok(clipboard) => clipboard,
        Err(error) => {
            println!("Error trying to get clipboard: {}", error);
            return;
        }
    };

    match clipboard.set_contents(text.to_owned()) {
        Ok(_) => {}
        Err(error) => println!("Error trying to copy to clipboard: {}", error),
    };
}

pub fn fill_window_with_history(window: &mut Cursive) {
    window.call_on_name("chat_history", |view: &mut LinearLayout| {
        view.clear();

        for (position, message) in HISTORY.read().unwrap().iter().enumerate() {
            view.add_child(TextView::new(" "));

            let message_text: String = format!("{:?}: {}", message.role, message.content);
            let horizontal_layout = LinearLayout::horizontal()
                .child(
                    TextView::new(message_text.clone())
                        .h_align(HAlign::Left)
                        .with_name("history_text_".to_owned() + &position.to_string()),
                )
                .child(TextView::new(" ").h_align(HAlign::Right))
                .child(Button::new("Copy", move |window| {
                    let message_text = window
                        .call_on_name(
                            &("history_text_".to_owned() + &position.to_string()),
                            |view: &mut TextView| (*view.get_content()).source().to_owned(),
                        )
                        .unwrap();
                    copy_to_clipboard(window, &message_text);
                }));

            view.add_child(horizontal_layout);
        }
    });
}
