use std::{
    cmp::Ordering,
    collections::VecDeque,
    iter::{Peekable, repeat},
    str::{Chars, Lines},
};

use itertools::Itertools;
use time::{Date, macros::format_description};
use unicode_bidi::{BidiDataSource, HardcodedBidiData, data_source::BidiMatchedOpeningBracket};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Date(Date),
    String(String),
    Number(i32),
    Minus,
    Plus,
    Debit,
    Credit,
    Assign,
    Colon,
    ColonBlockEnd,
    Semicolon,
    Newline,
    Identifier(String),
    BlockStart(char),
    BlockEnd(char),
    Indent,
    Dedent,
    Auto,
    Section,
    EOF,
}

impl Token {
    pub fn normalise(&self) -> Token {
        match self {
            Self::Newline => Token::Semicolon,
            Self::Colon => Token::BlockStart(':'),
            Self::ColonBlockEnd => Token::BlockEnd(':'),
            Self::Indent => Token::BlockStart('\t'),
            Self::Dedent => Token::BlockEnd('\t'),
            t => t.clone(),
        }
    }
}

pub struct Lexer<'a> {
    lines: Lines<'a>,
    indent_stack: Vec<i32>,
    token_queue: VecDeque<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines(),
            indent_stack: Vec::new(),
            token_queue: VecDeque::new(),
        }
    }

    fn tokenize_line(&mut self, line: &str) {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
            return;
        }
        let mut line_iter = trimmed_line.chars().peekable();
        let indent = line.find(|c: char| !c.is_whitespace()).unwrap_or(0) as i32;

        match self.indent_stack.last().copied() {
            None => {
                if indent > 0 {
                    self.indent_stack.push(indent);
                    self.token_queue.push_back(Token::Indent);
                } else {
                    self.token_queue.push_back(Token::Newline);
                }
            }
            Some(prev_indent) if indent > prev_indent => {
                self.indent_stack.push(indent);
                self.token_queue.push_back(Token::Indent);
            }
            Some(prev_indent) if indent < prev_indent => {
                while let Some(prev_indent) = self.indent_stack.last().copied() {
                    match indent.cmp(&prev_indent) {
                        Ordering::Less => {
                            self.indent_stack.pop();
                            self.token_queue.push_back(Token::Dedent);
                        }
                        Ordering::Greater => {
                            self.indent_stack.push(indent);
                            break;
                        }
                        _ => {
                            self.token_queue.push_back(Token::Newline);
                            break;
                        }
                    }
                }
            }
            Some(_) => {
                self.token_queue.push_back(Token::Newline);
            }
        }

        let mut colon_count = 0;

        while let Some(&c) = line_iter.peek() {
            match c {
                ':' => {
                    line_iter.next();
                    self.token_queue.push_back(Token::Colon);
                    colon_count += 1;
                }
                ';' => {
                    line_iter.next();
                    self.token_queue.push_back(Token::Semicolon);
                }
                '=' => {
                    line_iter.next();
                    self.token_queue.push_back(Token::Assign);
                }
                '-' => {
                    line_iter.next();
                    match line_iter.peek() {
                        Some('-') => {
                            break;
                        }
                        _ => {
                            self.token_queue.push_back(Token::Minus);
                            continue;
                        }
                    }
                }
                '\u{2212}' => {
                    line_iter.next();
                    self.token_queue.push_back(Token::Minus);
                }
                '+' => {
                    line_iter.next();
                    self.token_queue.push_back(Token::Plus);
                }
                '§' => {
                    line_iter.next();
                    self.token_queue.push_back(Token::Section);
                }
                c if c.is_ascii_digit() => {
                    let st = line_iter
                        .peeking_take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
                        .collect();
                    if let Some(t) = self.date_or_number(st) {
                        self.token_queue.push_back(t);
                    }
                }
                c if self.is_quotation(c) => {
                    line_iter.next();
                    self.string(&mut line_iter, c);
                }
                c if c.is_alphabetic() => {
                    self.identifier(&mut line_iter);
                }
                _ => {
                    let c = line_iter.next().unwrap();
                    if let Some(BidiMatchedOpeningBracket { opening, is_open }) =
                        BidiDataSource::bidi_matched_opening_bracket(&HardcodedBidiData, c)
                    {
                        if is_open {
                            self.token_queue.push_back(Token::BlockStart(opening));
                        } else {
                            self.token_queue.push_back(Token::BlockEnd(opening));
                        }
                    }
                }
            }
        }
        self.token_queue
            .extend(repeat(Token::ColonBlockEnd).take(colon_count));
    }

    fn string(&mut self, line_iter: &mut Peekable<Chars>, delimiter: char) {
        let string_content: String = line_iter.take_while(|c| *c != delimiter).collect();
        self.token_queue.push_back(Token::String(string_content));
    }

    fn is_quotation(&self, c: char) -> bool {
        matches!(
            c,
            '"' | '\''
                | '\u{00AB}'
                | '\u{2018}'
                | '\u{201B}'
                | '\u{201C}'
                | '\u{201F}'
                | '\u{2039}'
                | '\u{2E02}'
                | '\u{2E04}'
                | '\u{2E09}'
                | '\u{2E0C}'
                | '\u{2E1C}'
                | '\u{2E20}'
                | '\u{00BB}'
                | '\u{2019}'
                | '\u{201D}'
                | '\u{203A}'
                | '\u{2E03}'
                | '\u{2E05}'
                | '\u{2E0A}'
                | '\u{2E0D}'
                | '\u{2E1D}'
                | '\u{2E21}'
        )
    }

    fn identifier(&mut self, line_iter: &mut Peekable<Chars>) {
        let ident: String = line_iter
            .peeking_take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        match ident.as_str() {
            "AUTO" => {
                self.token_queue.push_back(Token::Auto);
            }
            "C" | "CR" | "Cr" | "CREDIT" | "KREDIT" => {
                self.token_queue.push_back(Token::Credit);
            }
            "D" | "DR" | "Dr" | "DEBIT" | "DEBET" => {
                self.token_queue.push_back(Token::Debit);
            }
            "cr" if line_iter.peek() == Some(&'.') => {
                line_iter.next();
                self.token_queue.push_back(Token::Credit);
            }
            "dr" if line_iter.peek() == Some(&'.') => {
                line_iter.next();
                self.token_queue.push_back(Token::Debit);
            }
            ident => {
                self.token_queue.push_back(Token::Identifier(ident.into()));
            }
        }

        // if ident == "AUTO" {
        //     self.token_queue.push_back(Token::Auto);
        // } else {
        //     self.token_queue.push_back(Token::Identifier(ident))
        // }
    }

    fn parse_decimal(&mut self, s: String) -> Option<Token> {
        let mut decimal_separator_seen = false;
        let mut decimal_places = 0;
        let mut amount = 0;
        let s_iter = s.chars();
        for c in s_iter {
            match c {
                c if c.is_ascii_digit() => {
                    if decimal_places < 2 {
                        amount *= 10;
                        amount += c.to_digit(10).expect("digit is not digit \u{1F914}") as i32;
                        if decimal_separator_seen {
                            decimal_places += 1;
                        }
                    }
                }
                '.' | ',' => {
                    decimal_separator_seen = true;
                }
                _ => {}
            }
        }
        Some(Token::Number(amount * 10_i32.pow(2 - decimal_places)))
    }

    fn parse_date(&mut self, s: String) -> Option<Token> {
        let date_format =
            format_description!("[day padding:none].[month padding:none].[year padding:none]");
        let date = if s.ends_with('.') {
            Date::parse(format!("{s}0").as_str(), date_format)
        } else {
            Date::parse(s.as_str(), date_format)
        };
        match date {
            Ok(date) => Some(Token::Date(date)),
            Err(_) => None,
        }
    }

    fn date_or_number(&mut self, s: String) -> Option<Token> {
        let comma_count = s.matches(',').count();
        let point_count = s.matches('.').count();
        if comma_count + point_count == 0 {
            return Some(Token::Number(
                s.parse::<i32>().expect("invalid number") * 100,
            ));
        } else if comma_count + point_count == 1 {
            return self.parse_decimal(s);
        } else if comma_count == 0 {
            if point_count == 1 {
                return self.parse_decimal(s);
            } else if point_count == 2 {
                return self.parse_date(s);
            }
        }
        None
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.token_queue.is_empty() {
            if let Some(l) = self.lines.next() {
                match l {
                    _ if l.find(|c: char| !c.is_whitespace()).is_none()
                        || l.trim().starts_with("--") =>
                    {
                        return self.next_token();
                    }
                    _ => {
                        self.tokenize_line(l);
                    }
                }
            } else {
                for _ in 0..self.indent_stack.len() {
                    self.token_queue.push_back(Token::Dedent);
                }
                self.token_queue.push_back(Token::EOF);
            }
        }
        self.token_queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string() {
        let mut lexer = Lexer::new(r#"'"' »abc»"#);
        lexer.next_token();
        assert_eq!(lexer.next_token(), Some(Token::String("\"".into())));
        assert_eq!(lexer.next_token(), Some(Token::String("abc".into())));
    }

    #[test]
    fn comment() {
        let mut lexer = Lexer::new("-- aaa\n- b\n\n  --ö\n\na");
        assert_eq!(lexer.next_token(), Some(Token::Newline));
        assert_eq!(lexer.next_token(), Some(Token::Minus));
        assert_eq!(lexer.next_token(), Some(Token::Identifier("b".into())));
        assert_eq!(lexer.next_token(), Some(Token::Newline));
        assert_eq!(lexer.next_token(), Some(Token::Identifier("a".into())));
    }

    #[test]
    fn debit() {
        let mut lexer = Lexer::new("D Dr DR DEBIT DEBET dr.");
        lexer.next_token();
        assert_eq!(lexer.next_token(), Some(Token::Debit));
        assert_eq!(lexer.next_token(), Some(Token::Debit));
        assert_eq!(lexer.next_token(), Some(Token::Debit));
        assert_eq!(lexer.next_token(), Some(Token::Debit));
        assert_eq!(lexer.next_token(), Some(Token::Debit));
        assert_eq!(lexer.next_token(), Some(Token::Debit));
    }
    #[test]
    fn credit() {
        let mut lexer = Lexer::new("C Cr CR CREDIT KREDIT cr.");
        lexer.next_token();
        assert_eq!(lexer.next_token(), Some(Token::Credit));
        assert_eq!(lexer.next_token(), Some(Token::Credit));
        assert_eq!(lexer.next_token(), Some(Token::Credit));
        assert_eq!(lexer.next_token(), Some(Token::Credit));
        assert_eq!(lexer.next_token(), Some(Token::Credit));
        assert_eq!(lexer.next_token(), Some(Token::Credit));
    }

}
