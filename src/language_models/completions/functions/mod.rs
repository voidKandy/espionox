use self::{
    errors::{FunctionError, ParserError},
    lexer::Lexer,
    parser::Parser,
};
use anyhow::anyhow;
use std::collections::HashMap;
use tracing_log::log::info;

mod errors;
mod lexer;
mod parser;
mod tests;
mod tokens;

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub params: HashMap<String, FunctionParam>,
}

impl TryFrom<&str> for Function {
    type Error = FunctionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut l = Lexer::new(value);
        let stream = l.lex_input()?;
        info!("lexed input: {:?}", stream);
        let mut parser = Parser::try_from(stream)?;
        info!("built parser: {:?}", parser);
        Ok(parser.parse_whole_function()?)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionParam {
    pub description: Option<String>,
    pub typ: ParamType,
    pub required: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionParamBuilder {
    description: Option<String>,
    typ: Option<ParamType>,
    required: bool,
}

impl TryInto<FunctionParam> for FunctionParamBuilder {
    type Error = ParserError;
    fn try_into(self) -> Result<FunctionParam, Self::Error> {
        info!("coercing into function: {:?}", self);
        Ok(FunctionParam {
            description: self.description,
            typ: self
                .typ
                .ok_or(ParserError::MissingField("typ".to_owned()))?,
            required: self.required,
        })
    }
}

impl FunctionParam {
    fn empty() -> FunctionParamBuilder {
        FunctionParamBuilder {
            description: None,
            typ: None,
            required: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParamType {
    String,
    Integer,
    Bool,
    Enum(Vec<String>),
}

impl TryFrom<&str> for ParamType {
    type Error = ParserError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            tokens::STRING => Ok(ParamType::String),
            tokens::BOOL => Ok(ParamType::Bool),
            tokens::INTEGER => Ok(ParamType::Integer),
            lit => Err(anyhow!("Invalid parameter identifier literal: {:?}", lit).into()),
        }
    }
}
