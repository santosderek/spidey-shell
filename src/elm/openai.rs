use super::api::{fetch_completion, load_history_list, HISTORYBUFFER};
use crate::menu::create_main_menu;
use openai_api_rs::v1::chat_completion::ChatCompletionMessage;

pub async fn create_history_panel() -> LinearLayout {
    let mut history_panel = LinearLayout::vertical();

    history_panel.add_child(TextView::new("History"));
    history_panel.add_child(DummyView);
    history_panel.add_child(
        LinearLayout::vertical()
            .with_name("history_panel")
            .scrollable(),
    );

    // Fetch history list
    let history_list = match load_history_list().await {
        Ok(history_list) => history_list,
        Err(error) => {
            println!("Error trying to load history list: {}", error);
            return history_panel;
        }
    };

    return history_panel;
}

/// Create a chat history view with a text area
pub async fn create_chat_layout() -> Dialog {
    // Create overall layout
    let mut overall_layout = LinearLayout::horizontal();

    // Add chat history list panel
    let chat_history_panel = create_history_panel().await;
    overall_layout.add_child(chat_history_panel.scrollable());

    // Add generation panel
    let mut chat_generation_panel = LinearLayout::vertical();

    chat_generation_panel.add_child(TextView::new("Start a conversation with OpenAI"));
    chat_generation_panel.add_child(DummyView);
    chat_generation_panel.add_child(
        LinearLayout::vertical()
            .with_name("chat_history")
            .scrollable(),
    );
    chat_generation_panel.add_child(DummyView);
    chat_generation_panel.add_child(
        TextArea::new()
            .with_name("openai_input_box")
            .min_size((10, 1)),
    );

    let mut vertical_chat = LinearLayout::vertical();

    vertical_chat.add_child(chat_generation_panel.scrollable());

    let main_menu = create_main_menu().await;
    let dialog = Dialog::around(vertical_chat)
        .title("OpenAI Chat")
        .button("Submit", submit_input)
        .button("Clear", |window| {
            window
                .call_on_name("openai_input_box", |input_box: &mut TextArea| {
                    input_box.set_content("");
                })
                .unwrap();
            window
                .call_on_name("chat_history", |view: &mut LinearLayout| {
                    view.clear();
                })
                .unwrap();
            HISTORYBUFFER.write().unwrap().clear();
        })
        .button("Quit", move |_window| {
            _window.pop_layer();
            _window.add_layer(main_menu);
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

    HISTORYBUFFER.write().unwrap().push(ChatCompletionMessage {
        role: first_completion.message.role.clone(),
        content: completion_text,
        name: None,
        function_call: None,
    });

    // FIXME: Need to use Ratatui to fill history ... probably not in this function though...
    // fill_window_with_history(window);
}
