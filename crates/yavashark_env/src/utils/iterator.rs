use crate::array::Array;
use crate::{ObjectHandle, Realm, Res, Symbol, Value};
use std::cell::Cell;

pub struct ValueIterator(Value);

impl ValueIterator {
    pub fn new(val: &Value, realm: &mut Realm) -> Res<Self> {
        let iter = val.call_method(&Symbol::ITERATOR.into(), realm, Vec::new())?;

        Ok(Self(iter))
    }

    pub fn next(&self, realm: &mut Realm) -> Res<Option<Value>> {
        let res = self.0.call_method(&"next".into(), realm, Vec::new())?;

        let res = res.as_object()?;

        if res
            .resolve_property("done", realm)?
            .unwrap_or(Value::Undefined)
            .is_truthy()
        {
            return Ok(None);
        }

        let val = res.resolve_property("value", realm)?;

        Ok(val)
    }

    pub fn close(self, realm: &mut Realm) -> Res {
        if let Some(return_method) = self.0.get_property_opt("return", realm)? {
            return_method.call(realm, Vec::new(), self.0)?;
        }

        Ok(())
    }
}

pub struct ArrayLike {
    val: Value,
    len: Cell<usize>,
    idx: Cell<usize>,
    values: Option<Vec<Value>>,
    iter: Option<ObjectHandle>,
}

impl ArrayLike {
    pub fn is_array_like(val: &Value, realm: &mut Realm) -> Res<bool> {
        if let Ok(Some(_)) = val.downcast::<Array>() {
            return Ok(true);
        }

        let Value::Object(o) = val else {
            return Ok(false);
        };

        if o.contains_key(Symbol::ITERATOR.into(), realm)? {
            return Ok(true);
        }

        if o.contains_own_key("length".into(), realm)? {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn new(val: Value, realm: &mut Realm) -> Res<Self> {
        if let Some(array) = val.downcast::<Array>()? {
            let values = array.to_vec()?;

            return Ok(Self {
                val: Value::Undefined,
                len: Cell::new(values.len()),
                idx: Cell::new(0),
                values: Some(values),
                iter: None,
            });
        }

        if let Some(iter) = val.get_property_opt(Symbol::ITERATOR, realm)? {
            let iter = iter.call(realm, Vec::new(), val)?.to_object()?;

            return Ok(Self {
                val: Value::Undefined,
                len: Cell::new(0),
                idx: Cell::new(0),
                values: None,
                iter: Some(iter),
            });
        }

        let len = val.get_property("length", realm)?.to_number(realm)?;

        Ok(Self {
            val,
            len: Cell::new(len as usize),
            idx: Cell::new(0),
            values: None,
            iter: None,
        })
    }

    pub fn next(&mut self, realm: &mut Realm) -> Res<Option<Value>> {
        if let Some(values) = &mut self.values {
            if self.idx.get() >= values.len() {
                return Ok(None);
            }

            let val = values[self.idx.get()].clone();

            self.idx.set(self.idx.get() + 1);

            return Ok(Some(val));
        }

        if let Some(iter) = &self.iter {
            let next = iter.call_method("next", realm, Vec::new())?;
            let next = next.as_object()?;

            let done = next
                .get_opt("done", realm)?
                .unwrap_or(Value::Undefined)
                .is_truthy();

            if done {
                return Ok(None);
            }

            let val = next.get_opt("value", realm)?.unwrap_or(Value::Undefined);

            self.idx.set(self.idx.get() + 1);

            return Ok(Some(val));
        }

        let idx = self.idx();
        let len = self.len();

        if idx >= len {
            return Ok(None);
        }

        let val = self
            .val
            .get_property_opt(idx, realm)?
            .unwrap_or(Value::Undefined);

        self.idx.set(idx + 1);

        Ok(Some(val))
    }

    pub fn close(&mut self, realm: &mut Realm) -> Res {
        if let Some(iter) = &self.iter {
            if let Some(return_method) = iter.get_opt("return", realm)? {
                return_method.call(realm, Vec::new(), iter.clone().into())?;
            }
        }

        Ok(())
    }

    pub const fn len(&self) -> usize {
        self.len.get()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn idx(&self) -> usize {
        self.idx.get()
    }

    pub fn to_vec_no_close(&mut self, realm: &mut Realm) -> Res<Vec<Value>> {
        if let Some(values) = &mut self.values {
            return Ok(values.clone());
        }

        let mut res = Vec::with_capacity(self.len());
        let idx = self.idx();
        self.idx.set(0);

        while let Some(val) = self.next(realm)? {
            res.push(val);
        }

        self.idx.set(idx);

        Ok(res)
    }

    pub fn to_vec(&mut self, realm: &mut Realm) -> Res<Vec<Value>> {
        let res = self.to_vec_no_close(realm)?;
        self.close(realm)?;
        Ok(res)
    }
}
