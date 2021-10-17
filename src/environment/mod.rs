use std::collections::HashMap;

use crate::{interpreter::Object, token::Token};

use self::error::{ErrorKind, RuntimeError};

pub mod error;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: &str, obj: Object) {
        self.values.insert(name.to_string(), obj);
    }
    pub fn get(&mut self, name: &Token) -> Result<&mut Object> {
        if let Some(obj) = self.values.get_mut(&name.lexeme) {
            Ok(obj)
        } else {
            Err(Box::new(RuntimeError::new(ErrorKind::UndefinedVariable(
                name.clone(),
            ))))
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}