use crate::utils::ValueIterator;
use crate::value::{IntoValue, MutObj, Obj};
use crate::{MutObject, ObjectHandle, Realm, Res, Value, ValueResult, WeakValue};
use indexmap::IndexSet;
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct WeakSet {
    #[mutable]
    set: IndexSet<WeakValue>,
}

impl WeakSet {
    #[allow(unused)]
    fn new(realm: &mut Realm) -> Res<Self> {
        Self::with_set(realm, IndexSet::new())
    }

    fn with_set(realm: &mut Realm, set: IndexSet<WeakValue>) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableWeakSet {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().weak_set.get(realm)?.clone(),
                ),
                set,
            }),
        })
    }
}

#[props(intrinsic_name = weak_set)]
impl WeakSet {
    #[constructor]
    fn construct(realm: &mut Realm, iter: Option<Value>) -> Res<ObjectHandle> {
        let mut set = IndexSet::new();

        if let Some(iter) = iter {
            let iter = ValueIterator::new(&iter, realm)?;

            while let Some(val) = iter.next(realm)? {
                set.insert(val.downgrade());
            }
        }

        Ok(WeakSet::with_set(realm, set)?.into_object())
    }

    fn add(&self, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        inner.set.insert(value.downgrade());

        Ok(value)
    }

    fn clear(&self) {
        let mut inner = self.inner.borrow_mut();

        inner.set.clear();
    }

    fn delete(&self, key: &Value) -> bool {
        let mut inner = self.inner.borrow_mut();

        inner.set.shift_remove(key)
    }

    fn difference(&self, other: &Self, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();
        let left = &inner.set;
        let inner = other.inner.borrow();
        let right = &inner.set;

        let diff = left.difference(right);

        let (low, up) = diff.size_hint();

        let mut set = IndexSet::with_capacity(up.unwrap_or(low));

        for val in diff {
            set.insert(val.clone());
        }

        Ok(Self::with_set(realm, set)?.into_value())
    }

    fn has(&self, key: &Value) -> bool {
        let inner = self.inner.borrow();

        inner.set.contains(key)
    }

    #[prop("forEach")]
    fn for_each(&self, func: &Value, this: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();

        for key in &inner.set {
            let Some(key) = key.upgrade() else {
                continue;
            };

            func.call(realm, vec![key, this.copy()], realm.global.clone().into())?;
        }

        Ok(Value::Undefined)
    }

    fn intersection(&self, other: &Self, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();
        let left = &inner.set;
        let inner = other.inner.borrow();
        let right = &inner.set;

        let intersection = left.intersection(right);

        let (low, up) = intersection.size_hint();

        let mut set = IndexSet::with_capacity(up.unwrap_or(low));

        for val in intersection {
            set.insert(val.clone());
        }

        Ok(Self::with_set(realm, set)?.into_value())
    }

    #[prop("isDisjointFrom")]
    fn is_disjoint_from(&self, other: &Self) -> bool {
        let inner = self.inner.borrow();
        let left = &inner.set;
        let inner = other.inner.borrow();
        let right = &inner.set;

        left.is_disjoint(right)
    }

    #[prop("isSubsetOf")]
    fn is_subset_of(&self, other: &Self) -> bool {
        let inner = self.inner.borrow();
        let left = &inner.set;
        let inner = other.inner.borrow();
        let right = &inner.set;

        left.is_subset(right)
    }

    #[prop("isSupersetOf")]
    fn is_superset_of(&self, other: &Self) -> bool {
        let inner = self.inner.borrow();
        let left = &inner.set;
        let inner = other.inner.borrow();
        let right = &inner.set;

        left.is_superset(right)
    }

    #[prop("symmetricDifference")]
    fn symmetric_difference(&self, other: &Self, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();
        let left = &inner.set;
        let inner = other.inner.borrow();
        let right = &inner.set;

        let diff = left.symmetric_difference(right);

        let (low, up) = diff.size_hint();

        let mut set = IndexSet::with_capacity(up.unwrap_or(low));

        for val in diff {
            set.insert(val.clone());
        }

        Ok(Self::with_set(realm, set)?.into_value())
    }

    #[get("size")]
    fn size(&self) -> usize {
        let inner = self.inner.borrow();

        inner.set.len()
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
