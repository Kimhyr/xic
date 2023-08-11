use crate::{
    diagnostics::Position,
    syntax::token::{Token, TokenType},
};
use phf::phf_map;
use std::str::Chars;

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "module" => TokenType::Module,
    "trait" => TokenType::Trait,
    "type" => TokenType::Type,
    "extend" => TokenType::Extend,
    "function" => TokenType::Function,
    "value" => TokenType::Value,
    "use" => TokenType::Use,
    "return" => TokenType::Return,
    "true" => TokenType::True,
    "flase" => TokenType::False,
    "bit" => TokenType::Bit,
    "bit8" => TokenType::Bit8,
    "bit16" => TokenType::Bit16,
    "bit32" => TokenType::Bit32,
    "bit64" => TokenType::Bit64,
    "int" => TokenType::Int,
    "int8" => TokenType::Int8,
    "int16" => TokenType::Int16,
    "int32" => TokenType::Int32,
    "int64" => TokenType::Int64,
    "float" => TokenType::Float,
    "float8" => TokenType::Float8,
    "float16" => TokenType::Float16,
    "float32" => TokenType::Float32,
    "float64" => TokenType::Float64,
    "bool" => TokenType::Bool,
    "char" => TokenType::Char,
    "char8" => TokenType::Char8,
    "char16" => TokenType::Char16,
    "char32" => TokenType::Char32,
};

pub struct Parser {}

#[derive(Debug, PartialEq)]
pub enum LexingError {
    MultipleDecimalPoints,
    DecimalParsing,
    BitsParsing,
    UnknownToken,
    End,
    InvalidEscapeSequence,
    IncompleteCharacter,
    IncompleteString,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    iterator: Chars<'a>,
    position: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(iterator: Chars<'a>) -> Self {
        Self {
            iterator: iterator.clone(),
            position: Position { row: 1, column: 1 },
        }
    }

    fn increment(&mut self) -> Option<char> {
        let current = self.iterator.next();
        if let None = current {
            return None;
        }

        let current = current.unwrap();
        if current == '\n' {
            self.position.column = 0;
            self.position.row += 1;
        }
        self.position.column += 1;
        Some(current)
    }

    fn next_numeric(&mut self, mut current: char) -> Result<TokenType, LexingError> {
        #[derive(PartialEq)]
        enum Type {
            Bits,
            Decimal,
        }

        let mut buffer = String::new();
        let mut r#type = Type::Bits;

        loop {
            buffer.push(current);

            let i = self.increment();
            if i == None {
                break;
            }

            current = i.unwrap();
            if current == '.' {
                if r#type == Type::Decimal {
                    return Err(LexingError::MultipleDecimalPoints);
                }
                r#type = Type::Decimal;
            }
        }

        if r#type == Type::Decimal {
            let decimal = buffer.parse::<f64>();
            if let Err(_) = decimal {
                Err(LexingError::DecimalParsing)
            } else {
                Ok(TokenType::Decimal(decimal.unwrap()))
            }
        } else {
            let bits = buffer.parse::<u64>();
            if let Err(_) = bits {
                Err(LexingError::BitsParsing)
            } else {
                Ok(TokenType::Bits(bits.unwrap()))
            }
        }
    }

    fn next_character(&mut self) -> Result<char, LexingError> {
        let current = self.increment();
        if let None = current {
            return Err(LexingError::End);
        }
        let mut current = current.unwrap();

        if current != '\\' {
            return Ok(current);
        }

        let result = self.increment();
        if result == None {
            return Err(LexingError::End);
        }
        current = result.unwrap();

        let result = match current {
            '\\' | '\'' | '\"' => current,
            _ => return Err(LexingError::InvalidEscapeSequence),
        };

        Ok(result)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexingError>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.increment();
        if let None = current {
            return None;
        }

        let mut current = current.unwrap();

        // Skip whitespace.
        loop {
            if !current.is_whitespace() {
                break;
            }

            let i = self.increment();
            if let None = i {
                return None;
            }

            current = i.unwrap();
        }

        let mut token = Token::new(Position {
            row: self.position.row,
            column: self.position.column - 1,
        });

        // Match the start symbol.
        token.r#type = match current {
            '\'' => match self.next_character() {
                Ok(mut ok) => {
                    if ok == '\'' {
                        ok = 0 as char;
                        self.increment();
                    } else {
                        let current = self.increment();
                        if let None = current {
                            return Some(Err(LexingError::IncompleteCharacter));
                        }

                        let current = current.unwrap();
                        if current != '\'' {
                            return Some(Err(LexingError::IncompleteCharacter));
                        }
                    }
                    TokenType::Character(ok)
                }
                Err(e) => {
                    if e != LexingError::End {
                        return Some(Err(e));
                    }
                    return Some(Err(LexingError::IncompleteCharacter));
                }
            },
            '"' => {
                let mut buffer = String::new();
                loop {
                    if let Some(c) = self.iterator.clone().peekable().peek() {
                        if *c == '\"' {
                            self.increment();
                            break;
                        }
                    }
                    let current = self.next_character();
                    if let Err(e) = current {
                        if e != LexingError::End {
                            return Some(Err(e));
                        }
                        return Some(Err(LexingError::IncompleteString));
                    }
                    let current = current.unwrap();
                    buffer.push(current);
                }

                if buffer.is_empty() {
                    buffer.push(0 as char);
                }
                TokenType::String(buffer)
            }
            '.' => TokenType::FullStop,
            ',' => TokenType::Comma,
            ':' => TokenType::Colon,
            ';' => TokenType::Semicolon,
            '=' => TokenType::EqualsSign,
            '+' => {
                if let Some(next) = self.iterator.clone().peekable().peek() {
                    if next.is_numeric() {
                        let result = self.next_numeric(current);
                        if let Err(e) = result {
                            return Some(Err(e));
                        }
                        result.unwrap()
                    } else {
                        TokenType::PlusSign
                    }
                } else {
                    TokenType::PlusSign
                }
            }
            '-' => {
                if let Some(next) = self.iterator.clone().peekable().peek() {
                    if *next == '>' {
                        TokenType::RightwardsArrow
                    } else if next.is_numeric() {
                        let result = self.next_numeric(current);
                        if let Err(e) = result {
                            return Some(Err(e));
                        }
                        result.unwrap()
                    } else {
                        TokenType::MinuxSign
                    }
                } else {
                    TokenType::MinuxSign
                }
            }
            '*' => TokenType::Asterisk,
            '/' => TokenType::Solidus,
            '\\' => TokenType::ReverseSolidus,
            '|' => TokenType::VerticalLine,
            '!' => TokenType::ExclamationMark,
            '?' => TokenType::QuestionMark,
            '@' => TokenType::ComercialAt,
            '#' => TokenType::NumberSign,
            '{' => TokenType::LeftCurlyBracket,
            '}' => TokenType::RightCurlyBracket,
            '(' => TokenType::LeftParenthesis,
            ')' => TokenType::RightParenthesis,
            '<' => TokenType::LeftAngleBracket,
            '>' => TokenType::RightAngleBracket,
            '[' => TokenType::LeftSquareBracket,
            ']' => TokenType::RightSquareBracket,
            _ => {
                if current.is_numeric() {
                    let result = self.next_numeric(current);
                    if let Err(e) = result {
                        return Some(Err(e));
                    }
                    result.unwrap()
                } else if current.is_alphabetic() {
                    let mut buffer = String::new();

                    loop {
                        buffer.push(current);

                        let i = self.increment();
                        if i == None {
                            break;
                        }

                        current = i.unwrap();
                        if !current.is_alphabetic() && current != '_' && !current.is_numeric() {
                            break;
                        }
                    }

                    if let Some(keyword) = KEYWORDS.get(buffer.as_str()) {
                        match keyword {
                            TokenType::True => TokenType::Boolean(true),
                            TokenType::False => TokenType::Boolean(false),
                            _ => keyword.clone(),
                        }
                    } else {
                        TokenType::Identifier(buffer)
                    }
                } else {
                    return Some(Err(LexingError::UnknownToken));
                }
            }
        };

        Some(Ok(token))
    }
}

#[test]
pub fn test_lexer() {
    // use std::env;
    use std::{fs::File, io::Read};
    let mut file = File::open("/home/k/projects/xic/tests/lexing_test_file.xi").unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();

    let mut lexer = Lexer::new(buffer.chars());
    while let Some(token) = lexer.next() {
        match token {
            Ok(token) => println!("{:?}", token),
            Err(error) => println!("{:?}", error),
        }
    }
    panic!();
}
