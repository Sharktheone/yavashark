use std::collections::HashMap;
use super::Value;


#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub properties: HashMap<String, Value>,
}