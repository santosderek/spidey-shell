use copypasta::{ClipboardContext, ClipboardProvider};

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
