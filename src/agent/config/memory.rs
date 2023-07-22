use super::super::super::io::commander::Commander;
use super::super::handler::context::Context;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::error::Error;

thread_local! {
    static SHORT_TERM_MEMORY: RefCell<Vec<Value>> = RefCell::new(Vec::new());
}

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

    pub fn load_short_term(&self) -> Vec<Value> {
        SHORT_TERM_MEMORY.with(|st_mem| st_mem.borrow().clone())
    }

    pub fn save_to_short_term(&self, content: Vec<Value>) {
        SHORT_TERM_MEMORY.with(|st_mem| {
            content.into_iter().for_each(|c| {
                st_mem.borrow_mut().push(c);
            })
        });
    }
}
