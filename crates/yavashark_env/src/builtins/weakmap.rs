use crate::utils::ValueIterator;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Value, ValueResult, WeakValue};
use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, MutObj, Obj};

#[object]
#[derive(Debug)]
pub struct WeakMap {
    // #[gc(untyped)] //TODO: this is a memleak!
    #[mutable]
    map: IndexMap<Value, WeakValue, FxBuildHasher>,
}

#[object(constructor)]
#[derive(Debug)]
pub struct WeakMapConstructor {}

impl Constructor<Realm> for WeakMapConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let mut map = IndexMap::<_, _, FxBuildHasher>::default();

        if let Some(iter) = args.first() {
            let iter = ValueIterator::new(iter, realm)?;

            while let Some(val) = iter.next(realm)? {
                let key = val.get_property(&0.into(), realm)?;
                let value = val.get_property(&1.into(), realm)?;

                map.insert(key, value.downgrade());
            }
        }

        let map = WeakMap {
            inner: RefCell::new(MutableWeakMap {
                object: MutObject::with_proto(realm.intrinsics.weak_map.clone().into()),
                map,
            }),
        };

        Ok(map.into_value())
    }
}

impl WeakMapConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableWeakMapConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl WeakMapConstructor {}

#[properties_new(constructor(WeakMapConstructor::new))]
impl WeakMap {
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

        inner
            .map
            .get(key)
            .map_or_else(|| Ok(Value::Undefined), |value| Ok(value.upgrade().unwrap_or(Value::Undefined)))
    }

    fn has(&self, key: &Value) -> bool {
        let inner = self.inner.borrow();

        inner.map.contains_key(key)
    }

    fn set(&self, key: Value, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        inner.map.insert(key, value.downgrade());

        Ok(value)
    }

    #[prop("forEach")]
    fn for_each(&self, func: &Value, this: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();

        for (key, value) in &inner.map {
            func.call(
                realm,
                vec![key.copy(), value.upgrade().unwrap_or(Value::Undefined), this.copy()],
                realm.global.clone().into(),
            )?;
        }

        Ok(Value::Undefined)
    }

    #[prop("getOrInsert")]
    fn get_or_insert(&self, key: Value, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        inner.map.entry(key).or_insert(value.downgrade());
        
        Ok(value)
    }

    #[prop("getOrInsertComputed")]
    fn get_or_insert_computed(
        &self,
        key: Value,
        callback: ObjectHandle,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        if !callback.is_function() {
            return Err(Error::ty("Callback must be a function"));
        }

        match inner.map.entry(key) {
            Entry::Occupied(entry) => Ok(entry.get().upgrade().unwrap_or(Value::Undefined)),

            Entry::Vacant(entry) => {
                let value = callback.call(realm, vec![entry.key().copy()], Value::Undefined)?;

                if !value.is_undefined() {
                    entry.insert(value.downgrade());
                }

                Ok(value)
            }
        }
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
