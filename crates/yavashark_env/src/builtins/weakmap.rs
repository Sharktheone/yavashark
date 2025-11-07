use crate::array::Array;
use crate::utils::ValueIterator;
use crate::value::{IntoValue, MutObj};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value, ValueResult, WeakValue};
use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct WeakMap {
    #[mutable]
    map: IndexMap<WeakValue, WeakValue, FxBuildHasher>,
}

#[props(intrinsic_name = weak_map)]
impl WeakMap {
    #[constructor]
    fn construct(realm: &mut Realm, iter: Option<Value>) -> Res<WeakMap> {
        let mut map = IndexMap::<_, _, FxBuildHasher>::default();

        if let Some(iter) = iter {
            let iter = ValueIterator::new(&iter, realm)?;

            while let Some(val) = iter.next(realm)? {
                let key = val.get_property(0, realm)?;
                let value = val.get_property(1, realm)?;

                map.insert(key.downgrade(), value.downgrade());
            }
        }

        let map = WeakMap {
            inner: RefCell::new(MutableWeakMap {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().weak_map.get(realm)?.clone(),
                ),
                map,
            }),
        };

        Ok(map)
    }

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

        if !callback.is_callable() {
            return Err(Error::ty("Callback must be a function"));
        }

        match inner.map.entry(key.downgrade()) {
            Entry::Occupied(mut entry) => {
                if let Some(value) = entry.get().upgrade() {
                    Ok(value)
                } else {
                    let value = callback.call(vec![key], Value::Undefined, realm)?;

                    if value.is_undefined() {
                        entry.shift_remove();
                    } else {
                        entry.insert(value.downgrade());
                    }

                    Ok(value)
                }
            }

            Entry::Vacant(entry) => {
                let value = callback.call(vec![key], Value::Undefined, realm)?;

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

    fn keys(&self, #[realm] realm: &mut Realm) -> ValueResult {
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
