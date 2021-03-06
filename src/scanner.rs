use crate::{
    error::{ErrorKind, Position},
    token::{Literal, Token, TokenType},
};
use core::panic;
use std::error;
pub struct Scanner {
    chars: Vec<char>,
    pub tokens: Vec<Token>,
    /// First charcter in the lexeme being scanned
    start: usize,
    /// The character considered
    current: usize,
    /// What src line we're on
    line: usize,
}
impl Scanner {
    pub fn new(src: String) -> Self {
        let chars = src.chars().collect::<Vec<char>>();
        Self {
            chars,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    pub fn scan_tokens(&mut self) -> Result<(), Box<dyn error::Error>> {
        while !self.is_at_end() {
            // Always remember the start position of the token, it's not modified anywhere else but here
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            Literal::Nil,
            self.line,
        ));

        Ok(())
    }

    fn advance(&mut self) -> &char {
        let char = &self.chars[self.current];
        self.current += 1;
        char
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Literal) {
        let text = self.chars.substring(self.start, self.current);
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, Literal::Nil);
    }

    fn scan_token(&mut self) -> Result<(), Box<dyn error::Error>> {
        match *self.advance() {
            // fully single characters
            s @ ('(' | ')' | '{' | '}' | ',' | '.' | '-' | '+' | ';' | '*') => {
                self.add_token(match s {
                    '(' => TokenType::LeftParen,
                    ')' => TokenType::RightParen,
                    '{' => TokenType::LeftBrace,
                    '}' => TokenType::RightBrace,
                    ',' => TokenType::Comma,
                    '.' => TokenType::Dot,
                    '-' => TokenType::Minus,
                    '+' => TokenType::Plus,
                    ';' => TokenType::Semicolon,
                    '*' => TokenType::Star,
                    _ => panic!(),
                });
            }
            // possible doubled chars
            // looks really ugly and we could combine them but I can't think of a way not to use doubled match statements
            d @ ('!' | '=' | '<' | '>') => {
                let res = match d {
                    '!' => {
                        if self.next_is('=') {
                            TokenType::BangEqual
                        } else {
                            TokenType::Bang
                        }
                    }
                    '=' => {
                        if self.next_is('=') {
                            TokenType::EqualEqual
                        } else {
                            TokenType::Equal
                        }
                    }
                    '<' => {
                        if self.next_is('=') {
                            TokenType::LessEqual
                        } else {
                            TokenType::Less
                        }
                    }
                    '>' => {
                        if self.next_is('=') {
                            TokenType::GreaterEqual
                        } else {
                            TokenType::Greater
                        }
                    }
                    _ => {
                        panic!()
                    }
                };
                self.add_token(res);
            }
            // any white space
            w if w.is_whitespace() => {}
            // newline
            '\n' => self.line += 1,
            // special character, could be divide, but also could be a comment
            '/' => match self.peek() {
                // Single line comment
                '/' => {
                    // comment until end of line
                    // why not just use next is you ask? well next is always consumes, i thought conditionals were short circuiting but whatever
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                // Multiline line comment
                '*' => {
                    let start_line = self.line;
                    // initiate stack
                    let mut stack = 1;

                    // As long as our stack isn't empty
                    while stack != 0 && !self.is_at_end() {
                        match self.peek() {
                            // if there's a new line add aline
                            '\n' => self.line += 1,
                            // if it's a star, check if it's an end comment
                            '*' => {
                                if self.peek_next() == '/' {
                                    stack -= 1;
                                    self.advance();
                                }
                            }
                            // if it's a slash, check if it's a begin comment
                            '/' => {
                                if self.peek_next() == '*' {
                                    stack += 1;
                                    self.advance();
                                }
                            }
                            _ => {}
                        }
                        // Advance regardless
                        self.advance();
                    }
                    if self.is_at_end() {
                        return Err(Box::new(crate::error::Error::new(
                            ErrorKind::UnterminatedComment(Position::new(start_line, self.start)),
                        )));
                    }
                }
                _ => self.add_token(TokenType::Slash),
            },
            // string literals
            '"' => {
                let start_line = self.line;
                while self.peek() != '"' && !self.is_at_end() {
                    // newlines inside "" don't count
                    if self.peek() == '\n' {
                        self.line += 1;
                    }
                    // remember, advance only changes current, not start
                    self.advance();
                }
                // check if string terminates at end of file w/o closing
                if self.is_at_end() {
                    return Err(Box::new(crate::error::Error::new(
                        ErrorKind::UnterminatedString(Position::new(start_line, self.start)),
                    )));
                }
                // advance one more time, since we stop at the quote
                self.advance();
                // trim the quotes, and add the token
                // substring start + 1 end - 1
                let text = self.chars.substring(self.start + 1, self.current - 1);
                self.add_token_literal(TokenType::String, Literal::String(text));
            }
            // digit
            n if n.is_digit(10) => {
                while self.peek().is_digit(10) {
                    self.advance();
                }
                // if fraction continue, also 0. doesn't work, it has to be 0.(digit+)
                if self.peek() == '.' && self.peek_next().is_digit(10) {
                    // consume .
                    self.advance();
                    // get the digits to the right
                    while self.peek().is_digit(10) {
                        self.advance();
                    }
                }

                // get string
                let text = self.chars.substring(self.start, self.current);
                // parse into f64
                let float = text.parse::<f32>().unwrap();
                // insert float into tokens
                self.add_token_literal(TokenType::Number, Literal::Number(float));
            }
            // letter = keywords, and user-defined variable names
            // if the character is a letter, begin
            n if n.is_alphabetic() => {
                // try to get the full word
                while self.peek().is_alphanumeric() {
                    self.advance();
                }
                // then, match the full word. If it matches up with one of our keywords it's a keyword
                // otherwise, it's an identifier!
                let text = self.chars.substring(self.start, self.current);
                self.add_token(keyword_type(&text));
            }
            _ => {
                return Err(Box::new(crate::error::Error::new(
                    ErrorKind::UnexpectedCharacter(Position::new(self.line, self.start)),
                )));
            }
        };
        Ok(())
    }

    fn next_is(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.chars[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    /// Peek character at current
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    /// Peek character at one after current
    fn peek_next(&self) -> char {
        let idx = self.current + 1;
        if idx > self.chars.len() {
            '\0'
        } else {
            self.chars[idx]
        }
    }
}

trait Substring {
    fn substring(&self, start: usize, end: usize) -> String;
}

impl Substring for Vec<char> {
    fn substring(&self, start: usize, end: usize) -> String {
        self[start..end].iter().cloned().collect::<String>()
    }
}

/// Serves as a hashmap, matches string to thing
fn keyword_type(str: &str) -> TokenType {
    match str {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "func" => TokenType::Func,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        // If it's not any of the above keyworks, let it be a user-defined name lol
        _ => TokenType::Identifier,
    }
}
