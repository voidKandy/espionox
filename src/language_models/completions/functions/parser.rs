use super::errors::{ParserError, ParserResult};
use super::tokens::{Token, TokenType, I};
use super::{Function, FunctionParam, ParamType};
use anyhow::anyhow;
use std::collections::{HashMap, VecDeque};
use tracing::{info, warn};

#[derive(Debug)]
pub struct Parser {
    stream: Vec<Token>,
    current: Token,
    next: Option<Token>,
}

fn remove_first_element<T>(vec: &mut Vec<T>) -> anyhow::Result<T> {
    if !vec.is_empty() {
        return Ok(vec.remove(0));
    }
    Err(anyhow!("vec empty!"))
}

impl TryFrom<Vec<Token>> for Parser {
    type Error = ParserError;
    fn try_from(mut stream: Vec<Token>) -> Result<Self, Self::Error> {
        let current = remove_first_element(&mut stream)?;
        let next = Some(remove_first_element(&mut stream)?);
        Ok(Self {
            stream,
            next,
            current,
        })
    }
}

impl Parser {
    // make this return a result
    fn next_token(&mut self) -> ParserResult<()> {
        self.current = self.next.take().ok_or(ParserError::NextTokenIsNone)?;
        self.next = remove_first_element(&mut self.stream).ok();
        Ok(())
    }

    fn expect_token(&self, tokentyp: TokenType) -> ParserResult<bool> {
        if self.next.as_ref().ok_or(ParserError::NextTokenIsNone)?.typ == tokentyp {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn parse_whole_function(&mut self) -> ParserResult<Function> {
        if self.current.typ != TokenType::Identifier {
            return Err(ParserError::UnexpectedToken);
        }
        let name = self.current.literal.drain(..).as_str().to_owned();
        self.expect_token(TokenType::LP)?;
        self.next_token()?;
        let params = self.parse_parameter_list()?;
        let mut function = Function {
            params,
            name,
            description: String::new(),
        };
        if self.expect_token(TokenType::Where)? {
            self.next_token()?;
            self.parse_where_clause(&mut function)?;
        }
        Ok(function)
    }

    #[tracing::instrument(name = "parse where clause")]
    pub(super) fn parse_where_clause(&mut self, function: &mut Function) -> ParserResult<()> {
        if self.current.typ != TokenType::Where {
            return Err(ParserError::UnexpectedToken);
        }
        if !self.expect_token(TokenType::I)? && !self.expect_token(TokenType::Identifier)? {
            return Err(anyhow!("unexpected token in where clause: {:?}", self.next).into());
        }
        self.next_token()?;
        while self.current.typ != TokenType::Eof {
            let id = self.current.literal.drain(..).as_str().to_owned();
            self.expect_token(TokenType::IsOrAm)?;
            self.next_token()?;
            self.expect_token(TokenType::StrLiteral)?;
            self.next_token()?;
            if id == I {
                function.description = self.current.literal.drain(..).as_str().to_owned();
            } else {
                let param = function
                    .params
                    .get_mut(&id)
                    .ok_or(ParserError::ParamNotFound(id))?;
                param.description = Some(self.current.literal.drain(..).as_str().to_owned())
            }
            self.next_token()?;
        }
        Ok(())
    }

    #[tracing::instrument(name = "parse param list")]
    pub(super) fn parse_parameter_list(&mut self) -> ParserResult<HashMap<String, FunctionParam>> {
        if self.current.typ != TokenType::LP {
            return Err(ParserError::UnexpectedToken);
        }
        self.next_token()?;
        let mut ret = HashMap::new();
        let mut current_param = FunctionParam::empty();
        let mut name: Option<String> = None;
        loop {
            match &self.current.typ {
                TokenType::Identifier => {
                    if name.is_none() {
                        name = Some(self.current.literal.drain(..).as_str().to_owned());
                        current_param.required = self.expect_token(TokenType::Bang)?;
                        if current_param.required {
                            self.next_token()?;
                        }
                    } else {
                        return Err(anyhow!("got an identifier where we shouldn't have").into());
                    }
                }

                t @ TokenType::Comma | t @ TokenType::RP => {
                    ret.insert(
                        name.take().ok_or(Into::<ParserError>::into(anyhow!(
                            "name is none when it shouldn't be"
                        )))?,
                        current_param.try_into()?,
                    );
                    current_param = FunctionParam::empty();
                    if *t == TokenType::RP {
                        break;
                    }
                }

                TokenType::Colon => {
                    if current_param.typ.is_none() {
                        if self.expect_token(TokenType::Integer)?
                            || self.expect_token(TokenType::String)?
                            || self.expect_token(TokenType::Bool)?
                        {
                            self.next_token()?;
                            current_param.typ =
                                Some(ParamType::try_from(self.current.literal.as_str())?);
                        } else if self.expect_token(TokenType::Enum)? {
                            self.next_token()?;
                            self.expect_token(TokenType::LP)?;
                            self.next_token()?;
                            let mut type_variants = vec![];

                            while !self.expect_token(TokenType::RP)? {
                                self.next_token()?;
                                match &self.current.typ {
                                    TokenType::StrLiteral => {
                                        type_variants.push(self.current.literal.to_owned());
                                    }
                                    t => {
                                        if *t != TokenType::Pipe {
                                            return Err(anyhow!(
                                                "unexpected token in enum declaration: {:?}",
                                                t
                                            )
                                            .into());
                                        }
                                    }
                                }
                            }
                            current_param.typ = Some(ParamType::Enum(type_variants));
                            self.next_token()?;
                        }
                    } else {
                        return Err(anyhow!("current param should be none, but it isn't").into());
                    }
                }
                tok => {
                    return Err(anyhow!("Unexpected token in parameter list: {:?}", tok).into());
                }
            }
            warn!(
                "end of match, current return state: {:?}\nMy state: {:?}",
                ret, self
            );
            self.next_token()?;
        }
        Ok(ret)
    }
}
