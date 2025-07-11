use rustc_hash::FxHashMap;
use yavashark_value::property_key::PropertyKey;
use crate::ObjectProperty;

pub struct ObjectProperties {
    pub values: Vec<ObjectProperty>,
    pub properties: FxHashMap<PropertyKey, usize>,
    pub array: Vec<(usize, usize)>, // (index, length)
}


impl ObjectProperties {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            properties: FxHashMap::default(),
            array: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.properties.clear();
        self.array.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}