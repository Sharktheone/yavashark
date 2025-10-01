use crate::array::Array;
use crate::utils::ValueIterator;
use crate::value::{Constructor, MutObj, Obj};
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Value, ValueResult, WeakValue};
use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};

#[object]
#[derive(Debug)]
pub struct WeakMap {
    #[mutable]
    map: IndexMap<WeakValue, WeakValue, FxBuildHasher>,
}

#[object(constructor)]
#[derive(Debug)]
pub struct WeakMapConstructor {}

impl Constructor for WeakMapConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let mut map = IndexMap::<_, _, FxBuildHasher>::default();

        if let Some(iter) = args.first() {
            let iter = ValueIterator::new(iter, realm)?;

            while let Some(val) = iter.next(realm)? {
                let key = val.get_property(&0.into(), realm)?;
                let value = val.get_property(&1.into(), realm)?;

                map.insert(key.downgrade(), value.downgrade());
            }
        }

        let map = WeakMap {
            inner: RefCell::new(MutableWeakMap {
                object: MutObject::with_proto(realm.intrinsics.weak_map.clone()),
                map,
            }),
        };

        Ok(map.into_value())
    }
}

impl WeakMapConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: ObjectHandle) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableWeakMapConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.initialize(func)?;

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

        inner.map.get(key).map_or_else(
            || Ok(Value::Undefined),
            |value| Ok(value.upgrade().unwrap_or(Value::Undefined)),
        )
    }

    fn has(&self, key: &Value) -> bool {
        let inner = self.inner.borrow();

        inner.map.contains_key(key)
    }

    fn set(&self, key: &Value, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        inner.map.insert(key.downgrade(), value.downgrade());

        Ok(value)
    }

    #[prop("forEach")]
    fn for_each(&self, func: &Value, this: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();

        for (key, value) in &inner.map {
            let Some(key) = key.upgrade() else {
                continue;
            };

            let Some(value) = value.upgrade() else {
                continue;
            };

            func.call(
                realm,
                vec![value, key, this.copy()],
                realm.global.clone().into(),
            )?;
        }

        Ok(Value::Undefined)
    }

    #[prop("getOrInsert")]
    fn get_or_insert(&self, key: &Value, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        inner
            .map
            .entry(key.downgrade())
            .or_insert(value.downgrade());

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

        match inner.map.entry(key.downgrade()) {
            Entry::Occupied(mut entry) => {
                if let Some(value) = entry.get().upgrade() {
                    Ok(value)
                } else {
                    let value = callback.call(realm, vec![key], Value::Undefined)?;

                    if value.is_undefined() {
                        entry.shift_remove();
                    } else {
                        entry.insert(value.downgrade());
                    }

                    Ok(value)
                }
            }

            Entry::Vacant(entry) => {
                let value = callback.call(realm, vec![key], Value::Undefined)?;

                if !value.is_undefined() {
                    entry.insert(value.downgrade());
                }

                Ok(value)
            }
        }
    }

    #[get("size")]
    fn size(&self) -> ValueResult {
        let inner = self.inner.borrow();

        Ok((inner.map.len() as i32).into())
    }

    fn keys(&self, #[realm] realm: &Realm) -> ValueResult {
        let inner = self.inner.borrow();

        let keys = inner
            .map
            .keys()
            .filter_map(WeakValue::upgrade)
            .collect::<Vec<_>>();

        Ok(Array::with_elements(realm, keys)?.into_value())
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
