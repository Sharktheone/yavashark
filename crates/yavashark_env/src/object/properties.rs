use rustc_hash::FxHashMap;
use yavashark_value::property_key::PropertyKey;
use crate::ObjectProperty;

pub struct ObjectProperties {
    pub values: Vec<ObjectProperty>,
    pub properties: FxHashMap<PropertyKey, usize>,
    pub array: ArrayProperties,
}


impl ObjectProperties {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            properties: FxHashMap::default(),
            array: ArrayProperties::Empty,
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


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ArrayProperties {
    #[default]
    Empty,
    Continuous(Vec<usize>),
    Sparse(Vec<(usize, usize)>), 
}

impl ArrayProperties {
    pub fn clear(&mut self) {
        match self {
            Self::Empty => {}
            Self::Continuous(arr) => arr.clear(),
            Self::Sparse(arr) => arr.clear(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Continuous(arr) => arr.is_empty(),
            Self::Sparse(arr) => arr.is_empty(),
        }
    }
}