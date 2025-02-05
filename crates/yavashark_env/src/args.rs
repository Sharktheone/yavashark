use std::mem;
use std::slice::IterMut;
use yavashark_value::{Realm};
use crate::{Value, Result, Error};
use crate::conversion::FromValueOutput;

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

trait ExtractValue<T>: Sized {
    type Output;
    fn extract(&mut self) -> Result<Self::Output>;
}

impl<T: FromValueOutput, R: Realm> ExtractValue<T> for Extractor<'_> {
    type Output = T::Output;
    fn extract(&mut self) -> Result<Self::Output> {
        let val = self
            .args
            .next()
            .ok_or_else(|| Error::ty_error("Expected a value".to_owned()))?;
        let val = mem::replace(val, Value::Undefined);

        T::from_value_out(val)
    }
}

impl<T: FromValueOutput, R: Realm> ExtractValue<Option<T>> for Extractor<'_> {
    type Output = Option<T::Output>;

    fn extract(&mut self) -> Result<Self::Output> {
        let Some(val) = self.args.next() else {
            return Ok(None);
        };

        let val = mem::replace(val, Value::Undefined);

        Ok(Some(T::from_value_out(val)?))
    }
}
