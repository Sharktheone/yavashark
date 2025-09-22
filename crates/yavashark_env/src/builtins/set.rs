use crate::utils::ValueIterator;
use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use indexmap::IndexSet;
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, MutObj, Obj};

#[object]
#[derive(Debug)]
pub struct Set {
    // #[gc(untyped)] //TODO: this is a memleak!
    #[mutable]
    set: IndexSet<Value>,
}

impl Set {
    #[allow(unused)]
    fn new(realm: &Realm) -> Self {
        Self::with_set(realm, IndexSet::new())
    }

    fn with_set(realm: &Realm, set: IndexSet<Value>) -> Self {
        Self {
            inner: RefCell::new(MutableSet {
                object: MutObject::with_proto(realm.intrinsics.set.clone().into()),
                set,
            }),
        }
    }
}

#[object(constructor)]
#[derive(Debug)]
pub struct SetConstructor {}

impl Constructor<Realm> for SetConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let mut set = IndexSet::new();

        if let Some(iter) = args.first() {
            let iter = ValueIterator::new(iter, realm)?;

            while let Some(val) = iter.next(realm)? {
                set.insert(val);
            }
        }

        Ok(Set::with_set(realm, set).into_value())
    }
}

impl SetConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableSetConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl SetConstructor {}

#[properties_new(constructor(SetConstructor::new))]
impl Set {
    fn add(&self, value: Value) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        inner.set.insert(value.copy());

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

    fn difference(&self, other: &Self, #[realm] realm: &Realm) -> ValueResult {
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

        Ok(Self::with_set(realm, set).into_value())
    }

    fn has(&self, key: &Value) -> bool {
        let inner = self.inner.borrow();

        inner.set.contains(key)
    }

    #[prop("forEach")]
    fn for_each(&self, func: &Value, this: &Value, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.borrow();

        for key in &inner.set {
            func.call(
                realm,
                vec![key.copy(), this.copy()],
                realm.global.clone().into(),
            )?;
        }

        Ok(Value::Undefined)
    }

    fn intersection(&self, other: &Self, #[realm] realm: &Realm) -> ValueResult {
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

        Ok(Self::with_set(realm, set).into_value())
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
    fn symmetric_difference(&self, other: &Self, #[realm] realm: &Realm) -> ValueResult {
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

        Ok(Self::with_set(realm, set).into_value())
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
