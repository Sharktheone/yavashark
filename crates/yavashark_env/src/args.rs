use crate::conversion::FromValueOutput;
use crate::{Realm, Res, Value};
use std::mem;
use std::slice::IterMut;

pub struct Extractor<'a> {
    pub args: IterMut<'a, Value>,
}

impl<'a> Extractor<'a> {
    pub fn new(args: &'a mut [Value]) -> Self {
        Self {
            args: args.iter_mut(),
        }
    }
}

pub trait ExtractValue<T>: Sized {
    type Output;
    fn extract(&mut self, realm: &mut Realm) -> Res<Self::Output>;
}

impl<T: FromValueOutput> ExtractValue<T> for Extractor<'_> {
    type Output = T::Output;
    fn extract(&mut self, realm: &mut Realm) -> Res<Self::Output> {
        let val = self
            .args
            .next()
            .map_or(Value::Undefined, |val| mem::replace(val, Value::Undefined));

        T::from_value_out(val, realm)
    }
}
