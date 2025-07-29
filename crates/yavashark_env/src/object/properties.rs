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
    Continuous(ContinuousArrayProperties),
    Sparse(SparseArrayProperties),
}


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContinuousArrayProperties {
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SparseArrayProperties {
    pub properties: Vec<(usize, ObjectProperty)>,
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

impl ContinuousArrayProperties {
    pub fn clear(&mut self) {
        self.properties.clear();
    }

    pub const fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
    
    pub fn insert(&mut self, idx: usize, value: ObjectProperty) -> Option<SparseArrayProperties> {
        if idx < self.properties.len() {
            self.properties[idx] = value;
        } else {
            if idx == self.properties.len() {
                self.properties.push(value)
            } else {
                return Some(
                    self.sparse_with(
                        iter::once(
                            (idx, value)
                        )
                    )
                );
            }
        }
        
        None
    }
    
    pub fn sparse(&mut self) -> SparseArrayProperties {
        let properties = mem::take(&mut self.properties);
        
        let properties = properties
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();
        
        
        SparseArrayProperties {
            properties,
        }
    }
    
    pub fn sparse_with(&mut self, additional: impl Iterator<Item=(usize, ObjectProperty)>) -> SparseArrayProperties {
        let properties = mem::take(&mut self.properties);

        let properties = properties
            .into_iter()
            .enumerate()
            .chain(additional)
            .collect();
        
        
        SparseArrayProperties {
            properties,
        }
    }
}

impl SparseArrayProperties {
    pub fn clear(&mut self) {
        self.properties.clear();
    }

    pub const fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
    
    pub fn is_continuous(&self) -> bool {
        if self.properties.is_empty() {
            return true;
        }
        
        let mut last_idx = 0;
        for (idx, _) in &self.properties {
            if *idx != last_idx {
                return false;
            }
            last_idx += 1;
        }
        
        true
    }
    
    pub fn insert(&mut self, idx: usize, value: ObjectProperty) -> Option<ContinuousArrayProperties> {
        //TODO
        
        None
    }
}
