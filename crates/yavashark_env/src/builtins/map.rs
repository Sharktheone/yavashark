use std::cell::RefCell;
use indexmap::IndexMap;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, MutObj, Obj};
use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use crate::utils::ValueIterator;

#[object]
#[derive(Debug)]
pub struct Map {
    // #[gc(untyped)] //TODO: this is a memleak!
    #[mutable]
    map: IndexMap<Value, Value>,
}



#[object(constructor)]
#[derive(Debug)]
pub struct MapConstructor {}

impl Constructor<Realm> for MapConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let mut map = IndexMap::new();
        
        if let Some(iter) = args.first() {
            let iter = ValueIterator::new(iter, realm)?;
            
            while let Some(val) =  iter.next(realm)? {
                let key = val.get_property(&0.into(), realm)?;
                let value = val.get_property(&1.into(), realm)?;
                
                map.insert(key, value);
            }
        }
        
        let map = Map {
            inner: RefCell::new(MutableMap {
                object: MutObject::with_proto(realm.intrinsics.map.clone().into()),
                map,
            }),
        };

        Ok(map.into_value())
    }
}

impl MapConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableMapConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };
        
        this.initialize(func.copy())?;
        
        Ok(this.into_object())
    }
}


#[properties_new(raw)]
impl MapConstructor {
}

#[properties_new(constructor(MapConstructor::new))]
impl Map {
    fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        
        inner.map.clear();
    }
    
    fn delete(&self, key: &Value) -> bool {
        let mut inner = self.inner.borrow_mut();
        
        inner.map.shift_remove(key).is_some()
    }
    
    fn get(&self, key: &Value) -> ValueResult {
        let inner = self.inner.borrow();
        
        inner.map.get(key)
            .map_or_else(|| Ok(Value::Undefined), |value| Ok(value.clone()))
    }
    
    fn has(&self, key: &Value) -> bool {
        let inner = self.inner.borrow();
        
        inner.map.contains_key(key)
    }
    
    fn set(&self, key: Value, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();
        
        inner.map.insert(key, value.copy());
        
        Ok(value)
    }
    
    #[prop("forEach")]
    fn for_each(&self, func: &Value, this: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();
        
        for (key, value) in &inner.map {
            func.call(realm, vec![key.copy(), value.copy(), this.copy()], realm.global.clone().into())?;
        }
        
        Ok(Value::Undefined)
    }
    
    // fn entries(&self, #[realm] realm: &mut Realm) -> ValueResult {
    //     let inner = self.inner.borrow();
    //     
    //     let array = 
    //     
    //     for (key, value) in &inner.map {
    //         let entry = realm.array_new();
    //         
    //         entry.push(key.copy());
    //         entry.push(value.copy());
    //         
    //         arr.push(entry.into());
    //     }
    //     
    //     Ok(arr.into())
    // }
    
    
    
    
    
    
    
}