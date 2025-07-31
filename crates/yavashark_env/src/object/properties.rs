#![allow(unused)]

use crate::{Error, ObjectProperty, Res};
use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::{iter, mem};
use std::cmp::Ordering;
use yavashark_value::property_key::{InternalPropertyKey, PropertyKey};

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
        if let InternalPropertyKey::Index(idx) = key {
            self.array.insert(idx, value);
            return Ok(());
        }

        let key = PropertyKey::from(key);

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
                return Some(self.sparse_with(iter::once((idx, value))));
            }
        }

        None
    }

    pub fn sparse(&mut self) -> SparseArrayProperties {
        let properties = mem::take(&mut self.properties);

        let properties = properties.into_iter().enumerate().collect::<Vec<_>>();

        SparseArrayProperties { properties }
    }

    pub fn sparse_with(
        &mut self,
        additional: impl Iterator<Item = (usize, ObjectProperty)>,
    ) -> SparseArrayProperties {
        let properties = mem::take(&mut self.properties);

        let properties = properties
            .into_iter()
            .enumerate()
            .chain(additional)
            .collect();

        SparseArrayProperties { properties }
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

        for (last_idx, (idx, _)) in self.properties.iter().enumerate() {
            if *idx != last_idx {
                return false;
            }
        }

        true
    }

    fn find_position(&self, target_idx: usize) -> (usize, bool) {
        let mut left = 0;
        let mut right = self.properties.len();

        while left < right {
            let mid = left + (right - left) / 2;
            let mid_idx = self.properties[mid].0;

            match mid_idx.cmp(&target_idx) {
                Ordering::Equal => return (mid, true),
                Ordering::Less => left = mid + 1,
                Ordering::Greater => right = mid,
            }
        }

        (left, false)
    }

    pub fn insert(
        &mut self,
        idx: usize,
        value: ObjectProperty,
    ) -> Option<ContinuousArrayProperties> {
        let (pos, found) = self.find_position(idx);

        if found {
            self.properties[pos].1 = value;
        } else {
            self.properties.insert(pos, (idx, value));
        }

        if self.is_continuous() {
            let properties = self
                .properties
                .iter()
                .map(|(_, prop)| prop.clone())
                .collect();
            Some(ContinuousArrayProperties { properties })
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
            properties: vec![value],
        }
    }
}

impl From<Vec<(usize, ObjectProperty)>> for SparseArrayProperties {
    fn from(properties: Vec<(usize, ObjectProperty)>) -> Self {
        Self { properties }
    }
}
