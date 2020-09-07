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
        let code = captures.get(2).unwrap().as_str();
        if get_code_legitimacy_probability(code) > 0.9_f64 {
            Some(code.to_string())
        } else {
            None
        }
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

fn get_code_legitimacy_probability(code: &str) -> f64 {
    let (mut lower, mut upper, mut numeric) = (0, 0, 0);

    for c in code.chars() {
        if c.is_ascii_digit() {
            numeric += 1;
        } else if c.is_uppercase() {
            upper += 1;
        } else {
            lower += 1;
        }
    }

    let length = code.len() as f64;
    let percentage_lower = lower as f64 / length;
    let percentage_upper = upper as f64 / length;
    let percentage_numeric = numeric as f64 / length;

    let chance_char = 26_f64 / 62_f64;
    let chance_num = 10_f64 / 62_f64;

    let error_lower = (chance_char - percentage_lower).abs();
    let error_upper = (chance_char - percentage_upper).abs();
    let error_numeric = (chance_num - percentage_numeric).abs();

    let error_amount = error_lower + error_upper + error_numeric;
    1_f64 / (1_f64 + (error_amount * 25_f64 - 15_f64).exp())
}