use crate::{Realm, Result, Symbol, Value};

pub struct ValueIterator(Value);

impl ValueIterator {
    pub fn new(val: &Value, realm: &mut Realm) -> Result<Self> {
        let iter = val.call_method(&Symbol::ITERATOR.into(), realm, Vec::new())?;

        Ok(Self(iter))
    }

    pub fn next(&self, realm: &mut Realm) -> Result<Option<Value>> {
        let res = self.0.call_method(&"next".into(), realm, Vec::new())?;
        let this = res.clone();

        let res = res.as_object()?;

        if res
            .get_property(&"done".into())?
            .resolve(this.clone(), realm)?
            .is_truthy()
        {
            return Ok(None);
        }

        let val = res.get_property(&"value".into())?;

        Ok(Some(val.resolve(this, realm)?))
    }
}
