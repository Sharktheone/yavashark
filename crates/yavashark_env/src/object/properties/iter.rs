use crate::object::properties::ArrayProperties;
use crate::ObjectProperty;

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