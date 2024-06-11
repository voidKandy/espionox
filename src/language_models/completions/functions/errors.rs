use crate::errors::error_chain_fmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type FunctionResult<T> = Result<T, FunctionError>;
#[derive(thiserror::Error)]
pub enum FunctionError {
    #[error(transparent)]
    Parser(#[from] ParserError),
    Lexer(#[from] LexerError),
}

impl Debug for FunctionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for FunctionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let display = match self {
            Self::Parser(err) => err.to_string(),
            Self::Lexer(err) => err.to_string(),
        };
        write!(f, "{}", display)
    }
}

pub type ParserResult<T> = Result<T, ParserError>;
#[derive(thiserror::Error)]
pub enum ParserError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    MissingField(String),
    ParamNotFound(String),
    UnexpectedToken,
    NextTokenIsNone,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let display = match self {
            Self::Undefined(err) => err.to_string(),
            Self::UnexpectedToken => "Unexpected Token".to_string(),
            Self::NextTokenIsNone => "Next Token Is None".to_string(),
            Self::MissingField(field) => format!("Missing Field: {}", field),
            Self::ParamNotFound(param) => format!("Param not found: {}", param),
        };
        write!(f, "{}", display)
    }
}

pub type LexerResult<T> = Result<T, LexerError>;
#[derive(thiserror::Error)]
pub enum LexerError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    CouldNotCoerceToToken(String),
}

impl Debug for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let display = match self {
            Self::Undefined(err) => err.to_string(),
            Self::CouldNotCoerceToToken(tok) => format!("Could not coerce to token: {:?}", tok),
        };
        write!(f, "{}", display)
    }
}
