use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, properties, properties_new};
use yavashark_value::{BoxedObj, Constructor, Func, Obj};

use crate::object::Object;
use crate::realm::Realm;
use crate::utils::ValueIterator;
use crate::{Error, ObjectHandle, Res, Value, ValueResult, Variable};
use crate::{MutObject, ObjectProperty};

#[object(direct(length), to_string)]
#[derive(Debug)]
pub struct Array {}

impl Array {
    pub fn with_elements(realm: &Realm, elements: Vec<Value>) -> Res<Self> {
        let array = Self::new(realm.intrinsics.array.clone().into());

        let mut inner = array.inner.try_borrow_mut()?;

        inner.object.set_array(elements);
        inner.length.value = Value::Number(inner.object.array.len() as f64);

        drop(inner);

        Ok(array)
    }

    pub fn with_len(realm: &Realm, len: usize) -> Res<Self> {
        let array = Self::new(realm.intrinsics.array.clone().into());

        let mut inner = array.inner.try_borrow_mut()?;

        inner.length.value = Value::Number(len as f64);

        drop(inner);

        Ok(array)
    }

    pub fn from_array_like(realm: &Realm, array_like: Value) -> Res<Self> {
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

    pub fn override_to_string(&self, realm: &mut Realm) -> Res<String> {
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

    pub fn override_to_string_internal(&self) -> Res<String> {
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
        let len = inner.length.value.to_number_or_null();

        if idx >= len as usize {
            inner.length.value = Value::Number(idx as f64 + 1.0);
        }

        Ok(())
    }

    pub fn as_vec(&self) -> Res<Vec<Value>> {
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

#[must_use]
pub fn convert_index(idx: isize, len: usize) -> usize {
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
    fn for_each(
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
                func.call(
                    realm,
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;
            }
        }

        Ok(Value::Undefined)
    }

    fn includes(
        #[this] this: &Value,
        search_element: &Value,
        from_index: Option<isize>,
    ) -> ValueResult {
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
    fn index_of(
        #[this] this: &Value,
        search_element: &Value,
        from_index: Option<isize>,
    ) -> ValueResult {
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

    fn keys(#[this] this: Value, #[realm] realm: &Realm) -> ValueResult {
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

    #[prop("lastIndexOf")]
    fn last_index_of(
        #[this] this: &Value,
        search_element: &Value,
        from_index: Option<isize>,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let from_index = from_index.unwrap_or(len as isize - 1);

        let from_index = convert_index(from_index, len);

        for idx in (0..=from_index).rev() {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                if val.eq(search_element) {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
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

    fn pop(#[this] this: &Value) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        if len == 0 {
            this.define_property("length".into(), 0.into())?;
            return Ok(Value::Undefined);
        }

        let idx = len - 1;

        let (_, val) = this.get_array_or_done(idx)?;

        this.define_property("length".into(), idx.into())?;
        this.define_property(idx.into(), Value::Undefined)?;

        Ok(val.unwrap_or(Value::Undefined))
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

    fn reduce(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        initial_value: &Value,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let mut acc = initial_value.clone();

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                acc = func.call(
                    realm,
                    vec![acc, val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;
            }
        }

        Ok(acc)
    }

    #[prop("reduceRight")]
    fn reduce_right(
        #[this] this: &Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        initial_value: &Value,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let mut acc = initial_value.clone();

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                acc = func.call(
                    realm,
                    vec![acc, val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                )?;
            }
        }

        Ok(acc)
    }

    fn reverse(#[this] this_val: Value) -> ValueResult {
        let this = this_val.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let mut idx = 0;

        while idx < len / 2 {
            let (left, left_val) = this.get_array_or_done(idx)?;
            let (right, right_val) = this.get_array_or_done(len - idx - 1)?;

            if let Some(left_val) = left_val {
                this.define_property(right.into(), left_val)?;
            }

            if let Some(right_val) = right_val {
                this.define_property(left.into(), right_val)?;
            }

            idx += 1;
        }

        Ok(this_val)
    }

    fn shift(#[this] this: &Value) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        if len == 0 {
            this.define_property("length".into(), 0.into())?;
            return Ok(Value::Undefined);
        }

        let (_, val) = this.get_array_or_done(0)?;

        for idx in 1..len {
            let (_, val) = this.get_array_or_done(idx)?;

            this.define_property((idx - 1).into(), val.unwrap_or(Value::Undefined))?;
        }

        this.define_property("length".into(), (len - 1).into())?;

        Ok(val.unwrap_or(Value::Undefined))
    }

    fn slice(
        #[this] this: &Value,
        start: isize,
        end: Option<isize>,
        #[realm] realm: &Realm,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let start = convert_index(start, len);
        let end = end.map_or(len, |end| convert_index(end, len));

        let array = Self::from_realm(realm);

        for idx in start..end {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                array.push(val)?;
            }
        }

        Ok(array.into_value())
    }

    fn some(#[this] this: &Value, #[realm] realm: &mut Realm, func: &ObjectHandle) -> ValueResult {
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
                    return Ok(Value::Boolean(true));
                }
            }
        }

        Ok(Value::Boolean(false))
    }

    // fn sort(#[this] this_val: Value, #[realm] realm: &mut Realm, func: &Value) -> ValueResult {} // TODO

    fn splice(
        #[this] this: &Value,
        start: isize,
        delete_count: Option<isize>,
        items: Vec<Value>,
        #[realm] realm: &Realm,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let start = convert_index(start, len);

        let delete_count = delete_count.unwrap_or(len as isize - start as isize);

        let delete_count = delete_count.max(0) as usize;

        let mut deleted = Vec::new();

        for idx in start..start + delete_count {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                deleted.push(val);
            }
        }

        let mut idx = start;

        let item_len = items.len();
        for item in items {
            this.define_property(idx.into(), item)?;
            idx += 1;
        }

        let mut shift = delete_count as isize - item_len as isize;

        while shift > 0 {
            this.define_property((idx as isize + shift).into(), Value::Undefined)?;
            shift -= 1;
        }

        let new_len = len as isize + item_len as isize - delete_count as isize;

        this.define_property("length".into(), new_len.into())?;

        Ok(Self::with_elements(realm, deleted)?.into_value())
    }

    #[prop("toReversed")]
    fn js_to_reversed(#[this] this: &Value, #[realm] realm: &Realm) -> ValueResult {
        let this = this.as_object()?;

        let array = Self::from_realm(realm);

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                array.push(val)?;
            }
        }

        Ok(array.into_value())
    }

    #[prop("toSorted")]
    fn js_to_sorted(
        #[this] this: &Value,
        func: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let mut values = Vec::new();

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx)?;

            if let Some(val) = val {
                values.push(val);
            }
        }

        if let Some(func) = func {
            values.sort_by(|a, b| {
                let Ok(x) = func.call(
                    realm,
                    vec![a.clone(), b.clone()],
                    realm.global.clone().into(),
                ) else {
                    //TODO: this is NOT good, we can't throw the error here
                    return Ordering::Equal;
                };

                x.as_number().partial_cmp(&0.0).unwrap_or(Ordering::Equal)
            });
        } else {
            values.sort_by_key(|a| a.to_string(realm).unwrap_or_default());
        }

        Ok(Self::with_elements(realm, values)?.into_value())
    }

    fn unshift(#[this] this: &Value, args: Vec<Value>) -> ValueResult {
        let this = this.as_object()?;

        let len = this.get_property(&"length".into())?;

        let len = len.value.as_number() as usize;

        let mut idx = args.len() + len;

        while idx > 0 {
            let (_, val) = this.get_array_or_done(idx - args.len())?;

            this.define_property(idx.into(), val.unwrap_or(Value::Undefined))?;

            idx -= 1;
        }

        for (idx, arg) in args.into_iter().enumerate() {
            this.define_property(idx.into(), arg)?;
        }

        this.define_property("length".into(), idx.into())?;

        Ok(idx.into())
    }

    fn values(#[this] this: Value, #[realm] realm: &Realm) -> ValueResult {
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

    fn with(
        #[this] this: &Value,
        idx: isize,
        val: &Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let len = this
            .get_property(&"length".into(), realm)?
            .to_number(realm)? as usize;

        let mut vals = Vec::with_capacity(len);

        let idx = convert_index(idx, len);

        for i in 0..len {
            if i == idx {
                vals.push(val.clone());
            } else {
                vals.push(this.get_property(&i.into(), realm)?.clone());
            }
        }

        Ok(Self::with_elements(realm, vals)?.into_value())
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
    
    
    #[prop("toString")]
    fn to_string_js(&self, #[realm] realm: &mut Realm) -> Res<String> {
        self.override_to_string(realm)
    }
}

#[object(constructor, function)]
#[derive(Debug)]
pub struct ArrayConstructor {}

impl Constructor<Realm> for ArrayConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        if args.len() == 1 {
            if let Value::Number(num) = &args[0] {
                return Ok(Array::with_len(realm, *num as usize)?.into_value());
            }
        }

        let this = Array::new(realm.intrinsics.array.clone().into());

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

impl Func<Realm> for ArrayConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _: Value) -> ValueResult {
        Constructor::construct(self, realm, args)
    }
}

impl ArrayConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, proto: &Value) -> Res<ObjectHandle> {
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
        let this: Res<OwningGcGuard<BoxedObj<Realm>, Array>, _> =
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
