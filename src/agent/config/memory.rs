use super::super::super::io::commander::Commander;
use super::super::handler::context::Context;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Memory {
    LongTerm,
    ShortTerm,
    Temporary,
}

impl Memory {
    pub fn init(self) -> Context {
        match self {
            Memory::LongTerm => {
                Context::new(self.load_long_term().unwrap(), self, Commander::new())
            }
            _ => Context::new(vec![], self, Commander::new()),
        }
    }

    pub fn load_long_term(&self) -> Result<Vec<Value>, Box<dyn Error>> {
        unimplemented!();
    }

    pub fn load_short_term(&self) -> Result<Vec<Value>, Box<dyn Error>> {
        dotenv::dotenv().ok();
        let mut contents = String::new();
        fs::File::open(env::var("SHORT_TERM_PATH").unwrap())?.read_to_string(&mut contents)?;
        println!("{contents}");
        match serde_json::from_str(&contents) {
            Ok(Value::Array(array)) => {
                return Ok(array);
            }
            Err(err) => {
                return Err(format!("Problem getting Json from String: {err:?}").into());
            }
            Ok(data) => Ok(vec![data]),
        }
    }

    pub fn save_to_short_term(&self, content: Vec<Value>) -> bool {
        dotenv::dotenv().ok();
        fs::write(
            env::var("SHORT_TERM_PATH").unwrap(),
            format!(
                "[{}]",
                content
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        )
        .unwrap();
        true
    }
}
