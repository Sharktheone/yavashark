#![allow(unused)]
use crate::Symbol;
use std::marker::PhantomData;
use yavashark_value::property_key::BorrowedPropertyKey;
use yavashark_value::Attributes;

type PreallocPropertyKey = BorrowedPropertyKey<'static>;

pub trait PreallocProperties<const N: usize, const S: usize, const G: usize> {
    const PROPS: [(PreallocPropertyKey, Attributes); N];
    const SETTERS: [PreallocPropertyKey; S];
    const GETTERS: [PreallocPropertyKey; G];
}

pub struct PreallocObject<
    P: PreallocProperties<N, S, G>,
    const N: usize,
    const S: usize,
    const G: usize,
> {
    pub properties: [i32; N],
    pub get: [i32; S],
    pub set: [i32; G],
    _marker: PhantomData<P>,
}

impl<P: PreallocProperties<N, S, G>, const N: usize, const S: usize, const G: usize> Default
    for PreallocObject<P, N, S, G>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P: PreallocProperties<N, S, G>, const N: usize, const S: usize, const G: usize>
    PreallocObject<P, N, S, G>
{
    pub const fn new() -> Self {
        let mut properties = [0; N];
        let mut get = [0; S];
        let mut set = [0; G];
        Self {
            properties,
            get,
            set,
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

impl PreallocProperties<4, 0, 0> for SomeObj {
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

    const SETTERS: [PreallocPropertyKey; 0] = [];
    const GETTERS: [PreallocPropertyKey; 0] = [];
}
