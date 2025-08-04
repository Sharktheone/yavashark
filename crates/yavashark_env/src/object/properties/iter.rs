use indexmap::map::Iter;
use yavashark_value::property_key::{BorrowedInternalPropertyKey, PropertyKey};
use crate::object::properties::{ArrayProperties, ObjectProperties};
use crate::ObjectProperty;


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
            InnerObjectPropertiesIter::Array(iter) => if let Some((index, prop)) = iter.next() {
                let key = BorrowedInternalPropertyKey::Index(index);
                Some((key, prop))
            } else {
                let mut iter = self.props.properties.iter();
                
                self.inner = InnerObjectPropertiesIter::Object(iter);
                
                self.next()
            },
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

impl <'a> ObjectPropertiesIter<'a> {
    pub fn new(props: &'a ObjectProperties) -> Self {
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
            ArrayProperties::Sparse(sparse) => {
                sparse.get_internal(self.index)
            }
        }
    }
}


impl<'a> ArrayPropertiesIter<'a> {
    pub fn new(array: &'a ArrayProperties) -> Self {
        Self { array, index: 0 }
    }
}