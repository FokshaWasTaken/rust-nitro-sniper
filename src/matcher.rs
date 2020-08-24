use regex::Regex;
use serenity::model::channel::Message;

pub fn get_gift_code(message: &Message) -> Option<String> {
    lazy_static! {
        static ref GIFT_PATTERN: Regex = Regex::new(
            "(discord.com/gifts/|discordapp.com/gifts/|discord.gift/)[ ]*([a-zA-Z0-9]{16,24})"
        )
        .unwrap();
    }
    let cleaned_content = sanitize_markdown(&message.content);
    if let Some(captures) = GIFT_PATTERN.captures(&cleaned_content) {
        Some(captures.get(2).unwrap().as_str().to_string())
    } else {
        None
    }
}

fn sanitize_markdown(dirty_string: &str) -> String {
    const MARKDOWN_CHARS: [char; 5] = ['*', '_', '`', '~', '|'];
    let mut output = dirty_string.to_string();
    output.retain(|c| !MARKDOWN_CHARS.contains(&c));
    output
}
