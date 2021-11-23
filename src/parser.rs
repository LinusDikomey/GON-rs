use std::{collections::HashMap, iter::Peekable, str::Chars};

use crate::{Gon, GonError};

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

pub(crate) trait Parser {
    fn next(&mut self) -> Option<char>;
    fn peek(&mut self) -> Option<char>;
    fn parse_object(&mut self) -> Result<Gon, GonError> {
        let mut map = HashMap::new();
        while !matches!(self.peek(), Some('}') | None) {
            let key = self.parse_string()?;
            self.skip_whitespace_and_token(':');
            let val = self.parse_val()?;
            if map.get(&key).is_some() {
                return Err(GonError::DuplicateKey(key));
            }
            map.insert(key, val);
            self.skip_whitespace_and_token(',');
        }
        Ok(Gon::Object(map))
    }
    
    fn parse_val<'a>(&mut self) -> Result<Gon, GonError> {
        match self.peek() {
            Some('{') => {
                self.next();
                self.skip_whitespace();
                let val = self.parse_object()?;
                if !matches!(self.next(), Some('}')) {
                    return Err(GonError::ClosingBraceExpected);
                }
                Ok(val)
            },
            Some('[') => {
                self.next();
                let mut arr = Vec::new();
                self.skip_whitespace();
                loop {
                    match self.peek() {
                        Some(']') => {
                            self.next();
                            break;
                        },
                        None => return Err(GonError::ClosingBracketExpected),
                        _ => {
                            arr.push(self.parse_val()?);
                            self.skip_whitespace_and_token(',');
                        }
                    }
                }
                Ok(Gon::Array(arr))
            }
            Some(_) => self.parse_string().map(|val| Gon::Value(val)),
            None => Err(GonError::ValueExpected)
        }
    }
    
    fn parse_string(&mut self) -> Result<String, GonError> {
        Ok(match self.peek() {
            Some('\"') => {
                self.next();
                let mut res = String::new();
                loop {
                    match self.next() {
                        Some('\\') => res.push(self.parse_escape()?),
                        Some('\"') => break,
                        Some(c) => res.push(c),
                        None => return Err(GonError::QuoteExpected)
                    }
                }
                res
            },
            Some(_) => {
                let mut res = String::new();
                loop {
                    match self.peek() {
                        Some('\\') => {
                            self.next();
                            self.parse_escape()?;
                        },
                        Some('{' | '}' |  '[' | ']' | ':' | ',') => break,
                        Some(c) if is_whitespace(c) => break,
                        None => break,
                        Some(_) => res.push(self.next().unwrap())
                    }
                }
                res
            },
            None => return Err(GonError::StringExpected)
        })
    }

    fn parse_escape(&mut self) -> Result<char, GonError> {
        Ok(match self.next() {
            Some('"') => '\"',
            Some('\\') => '\\',
            Some('/') => '/',
            Some('b') => '\x08',
            Some('f') => '\x0C',
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            // Unicode escape codes are supported in json but not supported right now
            Some('u') => return Err(GonError::HexEscapesNotSupported),
            Some(c) => return Err(GonError::UnexpectedEscapeCharacter(c)),
            None => return Err(GonError::EscapeCharacterExpected)
        })
    }
    
    
    
    fn skip_whitespace(&mut self) {
        while self.peek().map_or(false, |c| is_whitespace(c)) {
            self.next();
        }
    }
    
    /// Skips whitespace and and a single optional provided token. Returns if that token was skipped
    fn skip_whitespace_and_token(&mut self, c: char) -> bool {
        self.skip_whitespace();
        let skip = self.peek() == Some(c);
        if skip {
            self.next();
        }
        self.skip_whitespace();
        skip
    }
}

pub(crate) struct StrParser<'p>(Peekable<Chars<'p>>);
impl<'p> StrParser<'p> {
    pub(crate) fn new(s: &'p str) -> Self {
        Self(s.chars().peekable())
    }
}
impl<'p> Parser for StrParser<'p> {
    fn next(&mut self) -> Option<char> {
        self.0.next()
    }

    fn peek(&mut self) -> Option<char> {
        self.0.peek().map(|c| *c)
    }
}