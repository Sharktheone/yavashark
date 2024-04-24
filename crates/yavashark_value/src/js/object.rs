use std::collections::HashMap;
use crate::Value;


#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub properties: HashMap<String, Value>,
}