#![allow(unused)]
use std::marker::PhantomData;
use yavashark_value::Attributes;
use yavashark_value::property_key::BorrowedPropertyKey;
use crate::Symbol;

type PreallocPropertyKey = BorrowedPropertyKey<'static>;


pub trait PreallocProperties<const N: usize> {
    const PROPS: [(PreallocPropertyKey, Attributes); N];
}

pub struct PreallocObject<P: PreallocProperties<N>, const N: usize> {
    pub properties: [i32; N],
    _marker: PhantomData<P>,
}

impl<P: PreallocProperties<N>, const N: usize> Default for PreallocObject<P, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: PreallocProperties<N>, const N: usize> PreallocObject<P, N> {
    pub const fn new() -> Self {
        let mut properties = [0; N];
        Self {
            properties,
            _marker: PhantomData,
        }
    }

    pub fn get_property_name(&self, index: usize) -> Option<PreallocPropertyKey> {
        P::PROPS.get(index).map(|(key, _)| *key)
    }

    pub fn get_property(&self, name: &PreallocPropertyKey) -> Option<i32> {
        for (i, (key, _)) in P::PROPS.iter().enumerate() {
            if key == name {
                return Some(self.properties[i]);
            }
        }
        None
    }

    pub fn set_property(&mut self, name: PreallocPropertyKey, value: i32) -> Option<i32> {
        for (i, (key, _)) in P::PROPS.iter().enumerate() {
            if key == &name {
                let old_value = self.properties[i];
                self.properties[i] = value;
                return Some(old_value);
            }
        }
        None
    }
}

pub struct SomeObj;

impl PreallocProperties<4> for SomeObj {
    const PROPS: [(PreallocPropertyKey, Attributes); 4] = [
        (
            PreallocPropertyKey::String("someProperty"),
            Attributes::write_config(),
        ),
        (
            PreallocPropertyKey::Symbol(Symbol::ITERATOR),
            Attributes::config(),
        ),
        (
            PreallocPropertyKey::String("anotherProperty"),
            Attributes::enumerable(),
        ),
        (
            PreallocPropertyKey::Symbol(Symbol::ASYNC_ITERATOR),
            Attributes::write(),
        ),
    ];
}
