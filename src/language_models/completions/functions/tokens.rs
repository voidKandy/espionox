use super::errors::LexerError;
use tracing::warn;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub typ: TokenType,
    pub literal: String,
}

pub const LP: &str = "(";
pub const RP: &str = ")";
pub const COMMA: &str = ",";
pub const COLON: &str = ":";
pub const BANG: &str = "!";
pub const PIPE: &str = "|";
pub const EQ: &str = "=";

pub const I: &str = "i";
pub const WHERE: &str = "where";
pub const BOOL: &str = "bool";
pub const INTEGER: &str = "integer";
pub const STRING: &str = "string";
pub const ENUM: &str = "enum";
pub const IS: &str = "is";
pub const AM: &str = "am";

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Identifier, // only lowercase and _
    StrLiteral, // string surrounded by '

    Where,
    Bool,
    IsOrAm,
    Integer,
    String,
    Enum,
    I,

    LP,
    RP,
    Comma,
    Colon,
    Bang,
    Pipe,
    Eq,

    Eof,
}

impl TryFrom<&str> for Token {
    type Error = LexerError;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        let typ = match str {
            LP => TokenType::LP,
            RP => TokenType::RP,
            COLON => TokenType::Colon,
            COMMA => TokenType::Comma,
            BANG => TokenType::Bang,
            EQ => TokenType::Eq,
            PIPE => TokenType::Pipe,
            I => TokenType::I,
            WHERE => TokenType::Where,
            BOOL => TokenType::Bool,
            INTEGER => TokenType::Integer,
            STRING => TokenType::String,
            ENUM => TokenType::Enum,
            IS => TokenType::IsOrAm,
            AM => TokenType::IsOrAm,
            other => {
                warn!("Why was this token passed?: {}", other);
                return Err(LexerError::CouldNotCoerceToToken(other.to_owned()));
            }
        };
        Ok(Self {
            typ,
            literal: str.to_owned(),
        })
    }
}

impl Token {
    pub fn new_identifier(str: &str) -> Self {
        Self {
            typ: TokenType::Identifier,
            literal: str.to_owned(),
        }
    }
    pub fn new_str_literal(str: &str) -> Self {
        Self {
            typ: TokenType::StrLiteral,
            literal: str.to_owned(),
        }
    }
    pub fn end() -> Self {
        Self {
            typ: TokenType::Eof,
            literal: String::new(),
        }
    }
}
