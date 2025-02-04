use std::mem;
use std::slice::IterMut;
use crate::{Error, FromValueOutput, Realm, Value};

pub struct Extractor<'a, R: Realm> {
    pub args: IterMut<'a, Value<R>>,
}

impl<'a, R: Realm> Extractor<'a, R> {
    pub fn new(args: &'a mut [Value<R>]) -> Self {
        Self {
            args: args.iter_mut(),
        }
    }
} 

trait ExtractValue<T, R: Realm>: Sized {
    type Output;
    fn extract(&mut self) -> Result<Self::Output, Error<R>>;
}

impl<T: FromValueOutput<R>, R: Realm> ExtractValue<T, R> for Extractor<'_, R> {
    type Output = T::Output;
    fn extract(&mut self) -> Result<Self::Output, Error<R>> {
        let val = self
            .args
            .next()
            .ok_or_else(|| Error::ty_error("Expected a value".to_owned()))?;
        let val = mem::replace(val, Value::Undefined);

        T::from_value_out(val)
    }
}

impl<T: FromValueOutput<R>, R: Realm> ExtractValue<Option<T>, R> for Extractor<'_, R> {
    type Output = Option<T::Output>;

    fn extract(&mut self) -> Result<Self::Output, Error<R>> {
        let Some(val) = self.args.next() else {
            return Ok(None);
        };

        let val = mem::replace(val, Value::Undefined);

        Ok(Some(T::from_value_out(val)?))
    }
}
