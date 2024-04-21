use std::collections::HashMap;
use crate::Value;

pub struct Object {
    pub properties: HashMap<String, Value>,
}