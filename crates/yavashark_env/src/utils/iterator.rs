use std::cell::Cell;
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


pub struct ArrayLike {
    val: Value,
    len: Cell<usize>,
    idx: Cell<usize>,
    
}

impl ArrayLike {
    pub fn new(val: Value, realm: &mut Realm) -> Result<Self> {
        let len = val.get_property(&"length".into(), realm)?.to_number(realm)?;

        Ok(Self {
            val,
            len: Cell::new(len as usize),
            idx: Cell::new(0),
        })
    }
    
    pub fn next(&self, realm: &mut Realm) -> Result<Option<Value>> {
        let idx = self.idx();
        let len = self.len();
        
        if idx >= len {
            return Ok(None);
        }
        
        let val = self.val.get_property(&idx.into(), realm)?;
        
        self.idx.set(idx + 1);
        
        Ok(Some(val))
    }
    
    pub fn len(&self) -> usize {
        self.len.get()
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn idx(&self) -> usize {
        self.idx.get()
    }
    
    pub fn to_vec(&self, realm: &mut Realm) -> Result<Vec<Value>> {
        let mut res = Vec::with_capacity(self.len());
        let idx = self.idx();
        self.idx.set(0);
        
        while let Some(val) = self.next(realm)? {
            res.push(val);
        }
        
        self.idx.set(idx);
        
        Ok(res)
    }
}
