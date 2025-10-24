use crate::array::Array;
use crate::utils::ValueIterator;
use crate::value::{Constructor, IntoValue, MutObj, Obj};
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use indexmap::map::Entry;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};

#[object]
#[derive(Debug)]
pub struct Map {
    // #[gc(untyped)] //TODO: this is a memleak!
    #[mutable]
    pub map: IndexMap<Value, Value, FxBuildHasher>,
}

#[object(constructor)]
#[derive(Debug)]
pub struct MapConstructor {}

impl Constructor for MapConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        let mut map = IndexMap::<_, _, FxBuildHasher>::default();

        if let Some(iter) = args.first() {
            let iter = ValueIterator::new(iter, realm)?;

            while let Some(val) = iter.next(realm)? {
                let key = val.get_property(0, realm)?;
                let value = val.get_property(1, realm)?;

                map.insert(key, value);
            }
        }

        let map = Map {
            inner: RefCell::new(MutableMap {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().map.get(realm)?.clone(),
                ),
                map,
            }),
        };

        Ok(map.into_object())
    }
}

impl MapConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: ObjectHandle, realm: &mut Realm) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableMapConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl MapConstructor {}

#[properties_new(intrinsic_name(map), constructor(MapConstructor::new))]
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

        inner
            .map
            .get(key)
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
            func.call(
                realm,
                vec![value.copy(), key.copy(), this.copy()],
                realm.global.clone().into(),
            )?;
        }

        Ok(Value::Undefined)
    }

    #[prop("getOrInsert")]
    fn get_or_insert(&self, key: Value, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        Ok(inner.map.entry(key).or_insert(value).clone())
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

        match inner.map.entry(key) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),

            Entry::Vacant(entry) => {
                let value = callback.call(vec![entry.key().copy()], Value::Undefined, realm)?;

                if !value.is_undefined() {
                    entry.insert(value.copy());
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

        let keys = inner.map.keys().cloned().collect::<Vec<_>>();

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
