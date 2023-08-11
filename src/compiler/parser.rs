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
    "false" => TokenType::False,
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
    IntegerParsing,
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

    pub fn position(&self) -> &Position {
        &self.position
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

    fn next_numeric(
        &mut self,
        mut current: char,
        mut r#type: TokenType,
    ) -> Result<TokenType, LexingError> {
        let mut buffer = String::new();

        loop {
            buffer.push(current);

            let i = self.increment();
            if i == None {
                break;
            }

            current = i.unwrap();
            if current == '.' {
                if r#type == TokenType::DecimalLiteral {
                    return Err(LexingError::MultipleDecimalPoints);
                }
                r#type = TokenType::DecimalLiteral;
            } else if !current.is_numeric() {
                break;
            }
        }

        match r#type {
            TokenType::DecimalLiteral => {
                let decimal = buffer.parse::<f64>();
                if let Err(_) = decimal {
                    Err(LexingError::DecimalParsing)
                } else {
                    Ok(TokenType::Decimal(decimal.unwrap()))
                }
            }
            TokenType::BitsLiteral => {
                let bits = buffer.parse::<u64>();
                if let Err(_) = bits {
                    Err(LexingError::BitsParsing)
                } else {
                    Ok(TokenType::Bits(bits.unwrap()))
                }
            }
            TokenType::IntegerLiteral => {
                let integer = buffer.parse::<i64>();
                if let Err(_) = integer {
                    Err(LexingError::IntegerParsing)
                } else {
                    Ok(TokenType::Integer(integer.unwrap()))
                }
            }
            _ => unreachable!(),
        }
    }

    fn next_character(&mut self) -> Result<(bool, char), LexingError> {
        let current = self.increment();
        if let None = current {
            return Err(LexingError::End);
        }
        let mut current = current.unwrap();

        if current != '\\' {
            return Ok((false, current));
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

        Ok((true, result))
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
                Ok((is_escaped, mut ok)) => {
                    if ok == '\'' && !is_escaped {
                        ok = '\0';
                    } else {
                        let current = self.increment();
                        if let None = current {
                            return Some(Err(LexingError::IncompleteCharacter));
                        }

                        let current = current.unwrap();
                        println!("{:?} {:?}", ok, current);

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
                    let current = self.next_character();
                    if let Err(e) = current {
                        if e != LexingError::End {
                            return Some(Err(e));
                        }
                        return Some(Err(LexingError::IncompleteString));
                    }

                    let (is_escaped, current) = current.unwrap();
                    if current == '"' && !is_escaped {
                        break;
                    }
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
                        let result = self.next_numeric(current, TokenType::IntegerLiteral);
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
                        self.increment();
                        TokenType::RightwardsArrow
                    } else if next.is_numeric() {
                        let result = self.next_numeric(current, TokenType::IntegerLiteral);
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
                    let result = self.next_numeric(current, TokenType::BitsLiteral);
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
    use std::{fs::File, io::Read};
    let mut file = File::open("/home/k/projects/xic/tests/lexing_test_file.xi").unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();

    let tokens = vec![
        // TokenType::Bits(u64),
        // TokenType::Integer(i64),
        // TokenType::Decimal(f64),
        // TokenType::Boolean(bool),
        TokenType::Integer(21),
        TokenType::Integer(-21),
        TokenType::Decimal(21.21),
        TokenType::Bits(21),
        TokenType::Boolean(true),
        TokenType::Boolean(false),
        TokenType::Identifier(String::from("C_oolIdentifier32_")),
        TokenType::Character('\0'),
        TokenType::String(String::from("\0")),
        TokenType::Character('\''),
        TokenType::String(String::from("Hello,\'\" World!")),
        TokenType::Module,
        TokenType::Trait,
        TokenType::Type,
        TokenType::Extend,
        TokenType::Function,
        TokenType::Value,
        TokenType::Use,
        TokenType::Return,
        TokenType::Bit,
        TokenType::Bit8,
        TokenType::Bit16,
        TokenType::Bit32,
        TokenType::Bit64,
        TokenType::Int,
        TokenType::Int8,
        TokenType::Int16,
        TokenType::Int32,
        TokenType::Int64,
        TokenType::Float,
        TokenType::Float8,
        TokenType::Float16,
        TokenType::Float32,
        TokenType::Float64,
        TokenType::Bool,
        TokenType::Char,
        TokenType::Char8,
        TokenType::Char16,
        TokenType::Char32,
        TokenType::FullStop,
        TokenType::Comma,
        TokenType::Colon,
        TokenType::Semicolon,
        TokenType::EqualsSign,
        TokenType::PlusSign,
        TokenType::MinuxSign,
        TokenType::Asterisk,
        TokenType::Solidus,
        TokenType::ReverseSolidus,
        TokenType::VerticalLine,
        TokenType::ExclamationMark,
        TokenType::QuestionMark,
        TokenType::ComercialAt,
        TokenType::NumberSign,
        TokenType::RightwardsArrow,
        TokenType::LeftCurlyBracket,
        TokenType::RightCurlyBracket,
        TokenType::LeftParenthesis,
        TokenType::RightParenthesis,
        TokenType::LeftAngleBracket,
        TokenType::RightAngleBracket,
        TokenType::LeftSquareBracket,
        TokenType::RightSquareBracket,
    ];

    let mut lexer = Lexer::new(buffer.chars());
    for token in tokens {
        if let Some(result) = lexer.next() {
            if let Ok(result) = result {
                println!("Expected: {:?}, Recieved: {:?}", token, result.r#type);
                if result.r#type != token {
                    panic!();
                }
            } else {
                println!("{:?} {:?}", lexer.position(), result);
                panic!();
            }
        } else {
            break;
        }
    }
}
