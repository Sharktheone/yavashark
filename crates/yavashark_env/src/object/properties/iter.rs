use crate::object::properties::{ArrayProperties, ObjectProperties};
use crate::ObjectProperty;
use indexmap::map::{Iter, Keys, Values};
use std::ops::Range;
use std::slice;
use yavashark_value::property_key::{BorrowedInternalPropertyKey, PropertyKey};

pub struct ObjectPropertiesIter<'a> {
    props: &'a ObjectProperties,
    inner: InnerObjectPropertiesIter<'a>,
}

enum InnerObjectPropertiesIter<'a> {
    Array(ArrayPropertiesIter<'a>),
    Object(Iter<'a, PropertyKey, ObjectProperty>),
}

impl<'a> Iterator for ObjectPropertiesIter<'a> {
    type Item = (BorrowedInternalPropertyKey<'a>, &'a ObjectProperty);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            InnerObjectPropertiesIter::Array(iter) => {
                if let Some((index, prop)) = iter.next() {
                    let key = BorrowedInternalPropertyKey::Index(index);
                    Some((key, prop))
                } else {
                    let mut iter = self.props.properties.iter();

                    self.inner = InnerObjectPropertiesIter::Object(iter);

                    self.next()
                }
            }
            InnerObjectPropertiesIter::Object(iter) => {
                let (key, prop) = iter.next()?;

                let key = match key {
                    PropertyKey::String(s) => BorrowedInternalPropertyKey::String(s.as_str()),
                    PropertyKey::Symbol(s) => BorrowedInternalPropertyKey::Symbol(s),
                };

                Some((key, prop))
            }
        }
    }
}

impl<'a> ObjectPropertiesIter<'a> {
    pub const fn new(props: &'a ObjectProperties) -> Self {
        Self {
            props,
            inner: InnerObjectPropertiesIter::Array(props.array.iter()),
        }
    }
}

pub struct ArrayPropertiesIter<'a> {
    array: &'a ArrayProperties,
    index: usize,
}

impl<'a> Iterator for ArrayPropertiesIter<'a> {
    type Item = (usize, &'a ObjectProperty);

    fn next(&mut self) -> Option<Self::Item> {
        match self.array {
            ArrayProperties::Empty => None,
            ArrayProperties::Continuous(continuous) => {
                let prop = continuous.get(self.index)?;

                let item = (self.index, prop);
                self.index += 1;

                Some(item)
            }
            ArrayProperties::Sparse(sparse) => sparse.get_internal(self.index),
        }
    }
}

impl<'a> ArrayPropertiesIter<'a> {
    pub const fn new(array: &'a ArrayProperties) -> Self {
        Self { array, index: 0 }
    }
}

pub struct ObjectPropertiesKeysIter<'a> {
    props: &'a ObjectProperties,
    inner: InnerObjectPropertiesKeysIter<'a>,
}

enum InnerObjectPropertiesKeysIter<'a> {
    Array(ArrayPropertiesKeysIter<'a>),
    Object(Keys<'a, PropertyKey, ObjectProperty>),
}

impl<'a> Iterator for ObjectPropertiesKeysIter<'a> {
    type Item = BorrowedInternalPropertyKey<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            InnerObjectPropertiesKeysIter::Array(iter) => {
                if let Some(index) = iter.next() {
                    Some(BorrowedInternalPropertyKey::Index(index))
                } else {
                    let mut iter = self.props.properties.keys();
                    self.inner = InnerObjectPropertiesKeysIter::Object(iter);
                    self.next()
                }
            }
            InnerObjectPropertiesKeysIter::Object(iter) => {
                let key = iter.next()?;
                match key {
                    PropertyKey::String(s) => Some(BorrowedInternalPropertyKey::String(s.as_str())),
                    PropertyKey::Symbol(s) => Some(BorrowedInternalPropertyKey::Symbol(s)),
                }
            }
        }
    }
}

impl<'a> ObjectPropertiesKeysIter<'a> {
    pub fn new(props: &'a ObjectProperties) -> Self {
        Self {
            props,
            inner: InnerObjectPropertiesKeysIter::Array(ArrayPropertiesKeysIter::new(&props.array)),
        }
    }
}

pub enum ArrayPropertiesKeysIter<'a> {
    Empty,
    Continuous(Range<usize>),
    Sparse(slice::Iter<'a, usize>),
}

impl Iterator for ArrayPropertiesKeysIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArrayPropertiesKeysIter::Empty => None,
            ArrayPropertiesKeysIter::Continuous(range) => range.next(),
            ArrayPropertiesKeysIter::Sparse(iter) => Some(*iter.next()?),
        }
    }
}

impl<'a> ArrayPropertiesKeysIter<'a> {
    pub fn new(array: &'a ArrayProperties) -> Self {
        match array {
            ArrayProperties::Empty => Self::Empty,
            ArrayProperties::Continuous(continuous) => {
                Self::Continuous(0..continuous.properties.len())
            }
            ArrayProperties::Sparse(sparse) => Self::Sparse(sparse.indices.iter()),
        }
    }
}

pub struct ObjectPropertiesValuesIter<'a> {
    props: &'a ObjectProperties,
    inner: InnerObjectPropertiesValuesIter<'a>,
}

enum InnerObjectPropertiesValuesIter<'a> {
    Array(ArrayPropertiesValuesIter<'a>),
    Object(Values<'a, PropertyKey, ObjectProperty>),
}

impl<'a> Iterator for ObjectPropertiesValuesIter<'a> {
    type Item = &'a ObjectProperty;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            InnerObjectPropertiesValuesIter::Array(iter) => {
                if let Some(index) = iter.next() {
                    Some(index)
                } else {
                    let mut iter = self.props.properties.values();
                    self.inner = InnerObjectPropertiesValuesIter::Object(iter);
                    self.next()
                }
            }
            InnerObjectPropertiesValuesIter::Object(iter) => iter.next(),
        }
    }
}

impl<'a> ObjectPropertiesValuesIter<'a> {
    pub fn new(props: &'a ObjectProperties) -> Self {
        Self {
            props,
            inner: InnerObjectPropertiesValuesIter::Array(ArrayPropertiesValuesIter::new(
                &props.array,
            )),
        }
    }
}

pub struct ArrayPropertiesValuesIter<'a> {
    iter: slice::Iter<'a, ObjectProperty>,
}

impl<'a> Iterator for ArrayPropertiesValuesIter<'a> {
    type Item = &'a ObjectProperty;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> ArrayPropertiesValuesIter<'a> {
    pub fn new(array: &'a ArrayProperties) -> Self {
        match array {
            ArrayProperties::Empty => Self { iter: [].iter() },
            ArrayProperties::Continuous(continuous) => Self {
                iter: continuous.properties.iter(),
            },
            ArrayProperties::Sparse(sparse) => Self {
                iter: sparse.properties.iter(),
            },
        }
    }
}
