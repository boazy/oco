use std::borrow::Cow;

pub fn quote(in_str: &str) -> Cow<str> {
    if in_str.is_empty() {
        "\"\"".into()
    } else if in_str.bytes().any(|c| should_be_quoted(c as char)) {
        let mut out: Vec<u8> = Vec::new();
        out.push(b'"');
        for c in in_str.bytes() {
            match c as char {
                '$' | '`' | '"' | '\\' => out.push(b'\\'),
                _ => ()
            }
            out.push(c);
        }
        out.push(b'"');

        // Unsafe code: we know that the input string is valid UTF-8 (it was an &str and we only added ASCII characters)
        unsafe { String::from_utf8_unchecked(out) }.into()
    } else {
        in_str.into()
    }
}

fn should_be_quoted(c: char) -> bool {
    matches!(c,
        '|' | '&' | ';' | '<' | '>' | '(' | ')' | '$' | '*' |
        '?' | '[' | '#' | '~' | '%' | ' ' | '"' | '`'|
        '\\' | '\'' | '\t' | '\r' | '\n'
    )
}