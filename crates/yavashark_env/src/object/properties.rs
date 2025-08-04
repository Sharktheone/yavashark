#![allow(unused)]

pub mod iter;

use crate::{Error, ObjectProperty, Res};
use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::cmp::Ordering;
use std::{iter, mem};
use yavashark_value::property_key::{BorrowedPropertyKey, InternalPropertyKey, PropertyKey};
use std::mem;
use crate::array::ArrayIterator;
use crate::object::properties::iter::ArrayPropertiesIter;

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

    pub fn insert(&mut self, key: InternalPropertyKey, value: ObjectProperty) -> Res {
        let key = match key {
            InternalPropertyKey::Index(idx) => {
                self.array.insert(idx, value);
                return Ok(());
            }
            InternalPropertyKey::String(s) => PropertyKey::String(s),
            InternalPropertyKey::Symbol(s) => PropertyKey::Symbol(s),
        };

        let entry = self.properties.entry(key);

        match entry {
            Entry::Occupied(mut e) => {
                if !e.get().attributes.is_writable() {
                    return Err(Error::ty_error(format!(
                        "Cannot write to property `{}`: property is not writable",
                        e.key()
                    )));
                }

                e.insert(value);
            }
            Entry::Vacant(e) => {
                e.insert(value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &InternalPropertyKey) -> Option<&ObjectProperty> {
        match key {
            InternalPropertyKey::String(s) => self
                .properties
                .get(&BorrowedPropertyKey::String(s.as_str())),
            InternalPropertyKey::Symbol(s) => self.properties.get(&BorrowedPropertyKey::Symbol(s)),
            InternalPropertyKey::Index(idx) => self.array.get(*idx),
        }
    }

    pub fn remove(&mut self, key: &InternalPropertyKey) -> Res {
        match key {
            InternalPropertyKey::Index(idx) => {
                self.array.remove(*idx);
                Ok(())
            }
            InternalPropertyKey::String(s) => {
                self.properties
                    .shift_remove(&BorrowedPropertyKey::String(s.as_str()));
                Ok(())
            }
            InternalPropertyKey::Symbol(s) => {
                self.properties
                    .shift_remove(&BorrowedPropertyKey::Symbol(s));
                Ok(())
            }
        }
    }
    
    pub fn contains_key(&self, key: &InternalPropertyKey) -> bool {
        match key {
            InternalPropertyKey::Index(idx) => self.array.contains_key(*idx),
            InternalPropertyKey::String(s) => {
                self.properties.contains_key(&BorrowedPropertyKey::String(s.as_str()))
            }
            InternalPropertyKey::Symbol(s) => {
                self.properties.contains_key(&BorrowedPropertyKey::Symbol(s))
            }
            
        }
        
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
    pub indices: Vec<usize>,
    pub properties: Vec<ObjectProperty>,
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

    pub fn insert(&mut self, idx: usize, value: ObjectProperty) {
        match self {
            Self::Empty => {
                if idx == 0 {
                    *self = Self::Continuous(value.into());
                } else {
                    *self = Self::Sparse((idx, value).into());
                }
            }
            Self::Continuous(arr) => {
                if let Some(arr) = arr.insert(idx, value) {
                    *self = Self::Sparse(arr);
                }
            }
            Self::Sparse(arr) => {
                if let Some(arr) = arr.insert(idx, value) {
                    *self = Self::Continuous(arr);
                }
            }
        }
    }

    pub fn get(&self, idx: usize) -> Option<&ObjectProperty> {
        match self {
            Self::Empty => None,
            Self::Continuous(arr) => arr.get(idx),
            Self::Sparse(arr) => arr.get(idx),
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<ObjectProperty> {
        match self {
            Self::Empty => None,
            Self::Continuous(arr) => match arr.remove(idx) {
                Ok(Some(value)) => Some(value),
                Ok(None) => None,
                Err(sparse) => {
                    *self = Self::Sparse(sparse);
                    None
                }
            },
            Self::Sparse(arr) => arr.remove(idx),
        }
    }

    pub fn contains_key(&self, idx: usize) -> bool {
        match self {
            Self::Empty => false,
            Self::Continuous(arr) => idx < arr.properties.len(),
            Self::Sparse(arr) => arr.indices.binary_search(&idx).is_ok(),
        }
    }
    
    pub fn iter(&self) -> ArrayPropertiesIter {
        ArrayPropertiesIter::new(self)
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
        match idx.cmp(&self.properties.len()) {
            Ordering::Less => self.properties[idx] = value,
            Ordering::Equal => self.properties.push(value),
            Ordering::Greater => return Some(self.sparse_with(std::iter::once((idx, value)))),
        }

        None
    }

    pub fn get(&self, idx: usize) -> Option<&ObjectProperty> {
        self.properties.get(idx)
    }

    pub fn remove(&mut self, idx: usize) -> Result<Option<ObjectProperty>, SparseArrayProperties> {
        if idx >= self.properties.len() {
            return Ok(None);
        }

        if idx == self.properties.len() - 1 {
            return Ok(self.properties.pop());
        }

        let mut sparse = self.sparse();

        sparse.remove(idx);

        Err(sparse)
    }

    pub fn sparse(&mut self) -> SparseArrayProperties {
        let properties = mem::take(&mut self.properties);

        let indices = (0..properties.len()).collect::<Vec<_>>();

        SparseArrayProperties {
            indices,
            properties,
        }
    }

    pub fn sparse_with(
        &mut self,
        additional: impl Iterator<Item = (usize, ObjectProperty)>,
    ) -> SparseArrayProperties {
        let mut properties = mem::take(&mut self.properties);
        let mut indices = (0..properties.len()).collect::<Vec<_>>();

        let (min, max) = additional.size_hint();

        let reserve = max.unwrap_or(min);

        properties.reserve(reserve);
        indices.reserve(reserve);

        for (idx, value) in additional {
            if idx < properties.len() {
                properties[idx] = value;
            } else {
                properties.push(value);
                indices.push(idx);
            }

            //TODO: currently this is JS-level UB when additional is not sorted
        }

        SparseArrayProperties {
            indices,
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

        for (pos, idx) in self.indices.iter().enumerate() {
            if *idx != pos {
                return false;
            }
        }

        true
    }

    fn find_position(&self, target_idx: usize) -> (usize, bool) {
        let mut left = 0;
        let mut right = self.indices.len();

        while left < right {
            let mid = left + (right - left) / 2;
            let mid_idx = self.indices[mid];

            match mid_idx.cmp(&target_idx) {
                Ordering::Equal => return (mid, true),
                Ordering::Less => left = mid + 1,
                Ordering::Greater => right = mid,
            }
        }

        (left, false)
    }
    
    fn get_internal(&self, offset: usize) -> Option<(usize, &ObjectProperty)> {
        let idx = *self.indices.get(offset)?;
        let value = self.properties.get(offset)?;
        
        Some((idx, value))
    }

    pub fn insert(
        &mut self,
        idx: usize,
        value: ObjectProperty,
    ) -> Option<ContinuousArrayProperties> {
        let (pos, found) = self.find_position(idx);

        if found {
            self.properties[pos] = value;
        } else {
            self.properties.insert(pos, value);
        }

        if self.is_continuous() {
            let properties = mem::take(&mut self.properties);

            Some(ContinuousArrayProperties { properties })
        } else {
            None
        }
    }

    pub fn get(&self, idx: usize) -> Option<&ObjectProperty> {
        let (pos, found) = self.find_position(idx);
        if found {
            Some(&self.properties[pos])
        } else {
            None
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<ObjectProperty> {
        let (pos, found) = self.find_position(idx);
        if found {
            Some(self.properties.remove(pos))
        } else {
            None
        }
    }
}

impl From<ObjectProperty> for ContinuousArrayProperties {
    fn from(value: ObjectProperty) -> Self {
        Self {
            properties: vec![value],
        }
    }
}

impl From<Vec<ObjectProperty>> for ContinuousArrayProperties {
    fn from(properties: Vec<ObjectProperty>) -> Self {
        Self { properties }
    }
}

impl From<(usize, ObjectProperty)> for SparseArrayProperties {
    fn from(value: (usize, ObjectProperty)) -> Self {
        Self {
            indices: vec![value.0],
            properties: vec![value.1],
        }
    }
}
