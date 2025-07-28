#![allow(unused)]

use crate::ObjectProperty;
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use yavashark_value::property_key::PropertyKey;

pub struct ObjectProperties {
    pub properties: IndexMap<PropertyKey, ObjectProperty, FxBuildHasher>,
    pub array: ArrayProperties,
}

impl ObjectProperties {
    pub fn new() -> Self {
        Self {
            properties: IndexMap::default(),
            array: ArrayProperties::Empty,
        }
    }

    pub fn clear(&mut self) {
        self.properties.clear();
        self.array.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.array.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ArrayProperties {
    #[default]
    Empty,
    Continuous(Vec<ObjectProperty>),
    Sparse(Vec<(usize, ObjectProperty)>),
}

impl ArrayProperties {
    pub fn clear(&mut self) {
        match self {
            Self::Empty => {}
            Self::Continuous(arr) => arr.clear(),
            Self::Sparse(arr) => arr.clear(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Continuous(arr) => arr.is_empty(),
            Self::Sparse(arr) => arr.is_empty(),
        }
    }
}
