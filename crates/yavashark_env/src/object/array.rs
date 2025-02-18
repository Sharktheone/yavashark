use std::cell::{Cell, RefCell};
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, properties, properties_new};
use yavashark_value::{BoxedObj, Constructor, Obj};

use crate::object::Object;
use crate::realm::Realm;
use crate::utils::ValueIterator;
use crate::{Error, ObjectHandle, Res, Result, Value, ValueResult, Variable};
use crate::{MutObject, ObjectProperty};

#[object(direct(length), to_string)]
#[derive(Debug)]
pub struct Array {}

impl Array {
    pub fn with_elements(realm: &Realm, elements: Vec<Value>) -> Result<Self> {
        let array = Self::new(realm.intrinsics.array.clone().into());

        let mut inner = array.inner.try_borrow_mut()?;

        inner.object.set_array(elements);
        inner.length.value = Value::Number(inner.object.array.len() as f64);

        drop(inner);

        Ok(array)
    }

    pub fn from_array_like(realm: &Realm, array_like: Value) -> Result<Self> {
        let Value::Object(array_like) = array_like else {
            return Err(Error::ty_error(format!(
                "Expected object, found {array_like:?}"
            )));
        };

        let array = Self::new(realm.intrinsics.array.clone().into());

        let mut inner = array.inner.try_borrow_mut()?;

        let len = array_like.get_property(&"length".into())?.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = array_like.get_array_or_done(idx)?;

            if let Some(val) = val {
                inner.object.array.push((idx, Variable::new(val).into()));
            }
        }

        inner.length.value = Value::Number(len as f64);

        drop(inner);

        Ok(array)
    }

    #[must_use]
    pub fn new(proto: Value) -> Self {
        Self {
            inner: RefCell::new(MutableArray {
                object: MutObject::with_proto(proto),
                length: ObjectProperty::new(Value::Number(0.0)),
            }),
        }
    }

    #[must_use]
    pub fn from_realm(realm: &Realm) -> Self {
        Self::new(realm.intrinsics.array.clone().into())
    }

    pub fn override_to_string(&self, realm: &mut Realm) -> Result<String> {
        let mut buf = String::new();

        let inner = self.inner.try_borrow()?;

        for (_, value) in &inner.object.array {
            buf.push_str(&value.value.to_string(realm)?);
            buf.push_str(", ");
        }

        buf.pop();
        buf.pop();

        Ok(buf)
    }

    pub fn override_to_string_internal(&self) -> Result<String> {
        use std::fmt::Write as _;

        let mut buf = String::new();

        let inner = self.inner.try_borrow()?;

        for (_, value) in &inner.object.array {
            let _ = write!(buf, "{}", value.value);

            buf.push_str(", ");
        }

        buf.pop();
        buf.pop();

        Ok(buf)
    }

    pub fn insert_array(&self, val: Value, idx: usize) -> Res {
        let mut inner = self.inner.try_borrow_mut()?;

        inner.object.insert_array(idx, val.into());

        Ok(())
    }

    pub fn as_vec(&self) -> Result<Vec<Value>> {
        let inner = self.inner.try_borrow()?;

        Ok(inner
            .object
            .array
            .iter()
            .map(|(_, v)| v.value.clone())
            .collect())
    }

    pub fn push(&self, value: Value) -> ValueResult {
        let mut inner = self.inner.try_borrow_mut()?;

        let index = inner.object.array.last().map_or(0, |(i, _)| *i + 1);

        inner
            .object
            .array
            .push((index, Variable::new(value).into()));
        inner.length.value = Value::Number(index as f64 + 1.0);

        Ok(Value::Undefined)
    }
}

impl Constructor<Realm> for Array {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let this = Self::new(realm.intrinsics.array.clone().into());

        let values = args
            .into_iter()
            .map(ObjectProperty::new)
            .enumerate()
            .collect::<Vec<_>>();

        let mut inner = this.inner.try_borrow_mut()?;

        inner.object.array = values;
        inner.length.value = Value::Number(inner.object.array.len() as f64);

        drop(inner);

        Ok(this.into_object().into())
    }
}

fn convert_index(idx: isize, len: usize) -> usize {
    if idx < 0 {
        (len as isize + idx).max(0) as usize
    } else {
        idx as usize
    }
}
#[properties_new(constructor(ArrayConstructor::new))]
impl Array {
    fn at(#[this] this: &Value, idx: usize) -> ValueResult {
        let this = this.as_object()?;

        let (_, val) = this.get_array_or_done(idx)?;

        Ok(val.map_or(Value::Undefined, |v| v))
    }

    fn concat(#[this] this: Value, #[realm] realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let array = Self::from_realm(realm);

        let mut push_to = |val: Value| -> Res {
            if let Value::Object(obj) = &val {
                if obj.contains_key(&"length".into())? {
                    let iter = ValueIterator::new(&val, realm)?;

                    while let Some(val) = iter.next(realm)? {
                        array.push(val)?;
                    }
                    return Ok(());
                }
            }

            array.push(val)?;

            Ok(())
        };

        push_to(this)?;

        for arg in args {
            push_to(arg)?;
        }

        Ok(array.into_value())
    }

    #[prop("copyWithin")]
    fn copy_within(
        #[this] this_val: Value,
        target: isize,
        start: isize,
        end: Option<isize>,
    ) -> ValueResult {
        let this = this_val.as_object()?;

        let len = this.get_property(&"length".into())?.value.as_number() as usize;

        let target = convert_index(target, len);
        let start = convert_index(start, len);
        let end = end.map_or(len, |end| convert_index(end, len));

        for (count, idx) in (start..end).enumerate() {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                this.define_property((target + count).into(), val)?;
            }
        }

        Ok(this_val)
    }

    fn entries(#[this] this: Value, #[realm] realm: &Realm) -> ValueResult {
        let this = this.to_object()?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone().into()),
            }),
            array: this,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_value())
    }

    fn every(#[this] this: &Value, #[realm] realm: &mut Realm, func: &ObjectHandle) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val, idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if x.is_falsey() {
                    return Ok(Value::Boolean(false));
                }
            }
        }

        Ok(Value::Boolean(true))
    }

    fn fill(
        #[this] this_val: Value,
        value: &Value,
        start: Option<isize>,
        end: Option<isize>,
    ) -> ValueResult {
        let this = this_val.as_object()?;

        let len = this.get_property(&"length".into())?.value.as_number() as usize;

        let start = start.map_or(0, |start| convert_index(start, len));
        let end = end.map_or(len, |end| convert_index(end, len));

        for idx in start..end {
            this.define_property(idx.into(), value.clone())?;
        }

        Ok(this_val)
    }

    fn filter(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let array = Self::from_realm(realm);

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if x.is_truthy() {
                    array.push(val)?;
                }
            }
        }

        Ok(array.into_value())
    }

    fn find(#[this] this: &Value, #[realm] realm: &mut Realm, func: &ObjectHandle) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if x.is_truthy() {
                    return Ok(val);
                }
            }
        }

        Ok(Value::Undefined)
    }

    fn find_index(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if x.is_truthy() {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    #[prop("findLast")]
    fn find_last(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if x.is_truthy() {
                    return Ok(val);
                }
            }
        }

        Ok(Value::Undefined)
    }

    #[prop("findLastIndex")]
    fn find_last_index(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if x.is_truthy() {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    fn flat(#[this] this: &Value, #[realm] realm: &mut Realm, depth: Option<isize>) -> ValueResult {
        fn flatten(array: &Array, realm: &mut Realm, val: Value, depth: isize) -> Res {
            if depth == 0 {
                array.push(val)?;
                return Ok(());
            }

            if let Value::Object(obj) = &val {
                if obj.contains_key(&"length".into())? {
                    let iter = ValueIterator::new(&val, realm)?;

                    while let Some(val) = iter.next(realm)? {
                        flatten(array, realm, val, depth - 1)?;
                    }
                    return Ok(());
                }
            }

            array.push(val)?;

            Ok(())
        }
        let this = this.as_object()?;

        let array = Self::from_realm(realm);

        let depth = depth.unwrap_or(1);

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                flatten(&array, realm, val, depth)?;
            }
        }

        Ok(array.into_value())
    }
    
    #[prop("flatMap")]
    fn flat_map(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = this.as_object()?;

        let array = Self::from_realm(realm);

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;

                if let Value::Object(obj) = &x {
                    if obj.contains_key(&"length".into())? {
                        let iter = ValueIterator::new(&x, realm)?;

                        while let Some(val) = iter.next(realm)? {
                            array.push(val)?;
                        }
                        continue;
                    }
                }

                array.push(x)?;
            }
        }

        Ok(array.into_value())
    }
    
    #[prop("forEach")]
    fn for_each(#[this] this: &Value, #[realm] realm: &mut Realm, func: &ObjectHandle) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;
            }
        }

        Ok(Value::Undefined)
    }
    
    fn includes(#[this] this: &Value, search_element: &Value, from_index: Option<isize>) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let from_index = from_index.unwrap_or(0);

        let from_index = convert_index(from_index, len);

        for idx in from_index..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                if val.eq(search_element) {
                    return Ok(Value::Boolean(true));
                }
            }
        }

        Ok(Value::Boolean(false))
    }
    
    #[prop("indexOf")]
    fn index_of(#[this] this: &Value, search_element: &Value, from_index: Option<isize>) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let from_index = from_index.unwrap_or(0);

        let from_index = convert_index(from_index, len);

        for idx in from_index..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                if val.eq(search_element) {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    #[prop("push")]
    fn push_js(#[this] this: &Value, #[variadic] args: &[Value]) -> ValueResult {
        let this = this.as_object()?;

        let mut idx = this.get_property(&"length".into())?.value.as_number() as usize;

        for arg in args {
            this.define_property(idx.into(), arg.clone())?;
            idx += 1;
        }

        this.define_property("length".into(), idx.into())?;

        Ok(idx.into())
    }

    fn join(#[this] this: &Value, #[realm] realm: &mut Realm, separator: &Value) -> ValueResult {
        let this = this.as_object()?;

        let mut buf = String::new();

        let len = this.get_property(&"length".into())?.value.as_number() as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                buf.push_str(&val.to_string(realm)?);
            }

            if idx < len - 1 {
                buf.push_str(&separator.to_string(realm)?);
            }
        }

        Ok(buf.into())
    }

    #[prop(crate::Symbol::ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator(&self, #[realm] realm: &Realm, #[this] this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone().into()),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        let iter: Box<dyn Obj<Realm>> = Box::new(iter);

        Ok(iter.into())
    }

    fn map(#[this] this: &Value, #[realm] realm: &mut Realm, func: &ObjectHandle) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let array = Self::from_realm(realm);

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                let x = func.call(realm, vec![val], realm.global.clone().into())?;

                array.insert_array(x, idx)?;
            }
        }

        Ok(array.into_value())
    }
}

#[object]
#[derive(Debug)]
pub struct ArrayConstructor {}

impl ArrayConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, proto: &Value) -> Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableArrayConstructor {
                object: MutObject::with_proto(proto.copy()),
            }),
        };

        this.initialize(proto.copy())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl ArrayConstructor {
    #[prop("isArray")]
    fn is_array(test: Value) -> bool {
        let this: Result<OwningGcGuard<BoxedObj<Realm>, Array>, _> =
            yavashark_value::FromValue::from_value(test);

        this.is_ok()
    }

    fn of(#[realm] realm: &Realm, args: Vec<Value>) -> ValueResult {
        let array = Array::with_elements(realm, args)?;

        Ok(array.into_value())
    }
}

#[object]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ArrayIterator {
    array: ObjectHandle,
    next: Cell<usize>,
    done: Cell<bool>,
}

#[properties]
impl ArrayIterator {
    #[prop]
    pub fn next(&self, _args: Vec<Value>, realm: &Realm) -> ValueResult {
        if self.done.get() {
            let obj = Object::new(realm);
            obj.define_property("value".into(), Value::Undefined)?;
            obj.define_property("done".into(), Value::Boolean(true))?;
            return Ok(obj.into());
        }

        let (done, value) = self.array.get_array_or_done(self.next.get())?;

        self.next.set(self.next.get() + 1);

        if done {
            self.done.set(true);
            let obj = Object::new(realm);
            obj.define_property("value".into(), Value::Undefined)?;
            obj.define_property("done".into(), Value::Boolean(true))?;
            return Ok(obj.into());
        }

        let value = value.map_or_else(
            || {
                self.done.set(true);
                Value::Undefined
            },
            |value| value,
        );

        let obj = Object::new(realm);
        obj.define_property("value".into(), value)?;
        obj.define_property("done".into(), Value::Boolean(self.done.get()))?;

        Ok(obj.into())
    }
}
