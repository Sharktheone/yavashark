use crate::builtins::{Map, NumberConstructor, Proxy, Set};
use crate::console::print::{PrettyObjectOverride, PrettyPrint};
use crate::conversion::TryIntoValue;
use crate::object::Object;
use crate::realm::{Intrinsic, Realm};
use crate::utils::{coerce_object_strict, ArrayLike, ProtoDefault, ValueIterator};
use crate::value::property_key::InternalPropertyKey;
use crate::value::{
    Attributes, BoxedObj, Constructor, CustomName, DefinePropertyResult, Func, IntoValue, MutObj,
    Obj, ObjectImpl, ObjectOrNull, Property,
};
use crate::MutObject;
use crate::{Error, ObjectHandle, Res, Value, ValueResult, Variable};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, properties, properties_new};
use yavashark_string::YSString;

#[derive(Debug)]
pub struct Array {
    inner: RefCell<MutObject>,
    length: Cell<usize>,
}

impl ObjectImpl for Array {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }

    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            let length = value.as_number() as usize;

            self.set_len(length)?;

            return Ok(DefinePropertyResult::Handled);
        }

        self.get_wrapped_object()
            .define_property(name, value, realm)?;

        let new_len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        let current_len = self.length.get();
        if new_len > current_len {
            self.length.set(new_len);
        }

        Ok(DefinePropertyResult::Handled)
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            let length = value.value.as_number() as usize;

            self.set_len(length)?;

            return Ok(DefinePropertyResult::Handled);
        }

        if self
            .get_wrapped_object()
            .define_property_attributes(name, value, realm)?
            == DefinePropertyResult::ReadOnly
        {
            return Ok(DefinePropertyResult::ReadOnly);
        }

        let new_len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        let current_len = self.length.get();
        if new_len > current_len {
            self.length.set(new_len);
        }

        Ok(DefinePropertyResult::Handled)
    }

    fn define_getter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_getter_attributes(name, callback, attributes, realm)?;

        let new_len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        let current_len = self.length.get();
        if new_len > current_len {
            self.length.set(new_len);
        }

        Ok(())
    }

    fn define_setter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_setter_attributes(name, callback, attributes, realm)?;

        let new_len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        let current_len = self.length.get();
        if new_len > current_len {
            self.length.set(new_len);
        }

        Ok(())
    }

    fn define_empty_accessor(
        &self,
        name: InternalPropertyKey,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        self.get_wrapped_object()
            .define_empty_accessor(name, attributes, realm)?;

        let new_len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        let current_len = self.length.get();
        if new_len > current_len {
            self.length.set(new_len);
        }

        Ok(())
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            return Ok(Some(Property::Value(
                self.length.get().into(),
                Attributes::write(),
            )));
        }

        self.get_wrapped_object().resolve_property(name, realm)
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            return Ok(Some(Property::Value(
                self.length.get().into(),
                Attributes::write(),
            )));
        }

        self.get_wrapped_object().get_own_property(name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            return Ok(true);
        }

        self.get_wrapped_object().contains_key(name, realm)
    }

    fn name(&self) -> String {
        "Array".to_string()
    }

    // fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
    //     let mut buf = String::new();
    //
    //     let inner = self.inner.try_borrow()?;
    //
    //     for (_, value) in &inner.array {
    //         let Some(value) = inner.values.get(*value) else {
    //             continue;
    //         };
    //
    //         buf.push_str(value.value.to_string(realm)?.as_str());
    //         buf.push_str(", ");
    //     }
    //
    //     buf.pop();
    //     buf.pop();
    //
    //     Ok(buf.into())
    // }
    //
    // fn to_string_internal(&self) -> Res<YSString> {
    //     use std::fmt::Write as _;
    //
    //     let mut buf = String::new();
    //
    //     let inner = self.inner.try_borrow()?;
    //
    //     for (_, value) in &inner.array {
    //         let Some(value) = inner.values.get(*value) else {
    //             continue;
    //         };
    //
    //         let _ = write!(buf, "{}", value.value);
    //
    //         buf.push_str(", ");
    //     }
    //
    //     buf.pop();
    //     buf.pop();
    //
    //     Ok(buf.into())
    // }
}

impl ProtoDefault for Array {
    fn proto_default(realm: &mut Realm) -> Res<Self> {
        Ok(Self::new(
            realm.intrinsics.clone_public().array.get(realm)?.clone(),
        ))
    }

    fn null_proto_default() -> Self {
        Self::new(ObjectOrNull::Null)
    }
}

impl Array {
    pub fn with_elements(realm: &mut Realm, elements: Vec<Value>) -> Res<Self> {
        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements.into_iter());

        drop(inner);

        Ok(array)
    }

    pub fn with_elements_sparse(realm: &mut Realm, elements: Vec<Option<Value>>) -> Res<Self> {
        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let len = elements.len();
        let mut inner = array.inner.try_borrow_mut()?;
        inner.set_array_sparse(elements.into_iter());
        drop(inner);

        array.length.set(len);

        Ok(array)
    }

    pub fn with_elements_this(
        realm: &mut Realm,
        elements: Vec<Value>,
        this: Value,
    ) -> Res<ObjectHandle> {
        if let Value::Object(this) = this {
            if this.is_constructable()
                && &this != realm.intrinsics.clone_public().array.get(realm)?
            {
                let array = this.construct(Vec::new(), realm)?;

                for (i, element) in elements.into_iter().enumerate() {
                    array.define_property(i.into(), element, realm)?;
                }

                return Ok(array);
            }
        }

        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements.into_iter());

        drop(inner);

        Ok(array.into_object())
    }

    pub fn with_elements_and_proto(proto: ObjectHandle, elements: Vec<Value>) -> Res<Self> {
        let array = Self::new(proto);

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements.into_iter());

        drop(inner);

        Ok(array)
    }

    pub fn from_iter(
        realm: &mut Realm,
        elements: impl ExactSizeIterator<Item = Value>,
    ) -> Res<Self> {
        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements);

        drop(inner);

        Ok(array)
    }

    pub fn from_iter_res(
        realm: &mut Realm,
        elements: impl ExactSizeIterator<Item = ValueResult>,
    ) -> Res<Self> {
        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array_res(elements)?;

        drop(inner);

        Ok(array)
    }

    pub fn from_iter_and_proto(
        proto: ObjectHandle,
        elements: impl ExactSizeIterator<Item = Value>,
    ) -> Res<Self> {
        let array = Self::new(proto);

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements);

        drop(inner);

        Ok(array)
    }

    pub fn from_iter_res_and_proto(
        proto: ObjectHandle,
        elements: impl ExactSizeIterator<Item = ValueResult>,
    ) -> Res<Self> {
        let array = Self::new(proto);

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array_res(elements)?;

        drop(inner);

        Ok(array)
    }

    pub fn with_len(realm: &mut Realm, len: usize) -> Res<Self> {
        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        array.length.set(len);

        Ok(array)
    }

    pub fn from_string(realm: &mut Realm, string: &str) -> Res<Self> {
        let elements = string
            .chars()
            .map(|c| c.to_string().into())
            .collect::<Vec<Value>>();

        let array = Self::with_elements(realm, elements)?;

        Ok(array)
    }

    pub fn from_string_this(realm: &mut Realm, string: &str, this: Value) -> Res<ObjectHandle> {
        let elements = string
            .chars()
            .map(|c| c.to_string().into())
            .collect::<Vec<Value>>();

        let array = Self::with_elements_this(realm, elements, this)?;

        Ok(array)
    }

    pub fn from_array_like(realm: &mut Realm, array_like: Value) -> Res<Self> {
        let array_like = match array_like {
            Value::Object(obj) => obj,
            Value::String(s) => {
                return Self::from_string(realm, &s);
            }
            _ => {
                return Err(Error::ty_error(format!(
                    "Expected object or string, found {array_like:?}"
                )));
            }
        };

        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let mut inner = array.inner.try_borrow_mut()?;

        let len = array_like
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        for idx in 0..len {
            let (_, val) = array_like.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let len = inner.values.len();
                inner.values.push(Variable::new(val.clone()).into());

                inner.array.push((idx, len));
            }
        }

        array.length.set(len);

        drop(inner);

        Ok(array)
    }

    #[must_use]
    pub fn new(proto: impl Into<ObjectOrNull>) -> Self {
        Self {
            inner: RefCell::new(MutObject::with_proto(proto)),
            length: Cell::new(0),
        }
    }

    pub fn from_realm(realm: &mut Realm) -> Res<Self> {
        Ok(Self::new(
            realm.intrinsics.clone_public().array.get(realm)?.clone(),
        ))
    }

    pub fn insert_array(&self, val: Value, idx: usize) -> Res {
        let mut inner = self.inner.try_borrow_mut()?;

        if inner.insert_array(idx, val) == DefinePropertyResult::ReadOnly {
            return Err(Error::ty("Cannot assign to read only property"));
        }
        let len = self.length.get();

        if idx >= len {
            self.length.set(idx + 1);
        }

        Ok(())
    }

    pub fn as_vec(&self) -> Res<Vec<Value>> {
        let inner = self.inner.try_borrow()?;

        Ok(inner
            .array
            .iter()
            .filter_map(|(_, v)| inner.values.get(*v).map(|p| p.value.clone()))
            .collect())
    }

    pub fn push(&self, value: Value) -> ValueResult {
        let mut inner = self.inner.try_borrow_mut()?;

        let index = self.length.get();

        let len = inner.values.len();
        inner.values.push(Variable::new(value).into());

        inner.array.push((index, len));
        self.length.set(index + 1);

        Ok(Value::Undefined)
    }

    pub fn increment_length(&self) {
        let current_len = self.length.get();
        self.length.set(current_len + 1);
    }

    pub fn to_vec(&self) -> Res<Vec<Value>> {
        let inner = self.inner.try_borrow()?;

        let len = inner.array.last().map_or(0, |(i, _)| *i + 1);

        let mut vec = vec![Value::Undefined; len];

        for (idx, value) in &inner.array {
            let Some(value) = inner.values.get(*value) else {
                continue;
            };

            vec[*idx] = value.value.clone();
        }

        Ok(vec)
    }

    pub fn set_len(&self, len: usize) -> Res {
        self.length.set(len);

        self.inner.try_borrow_mut()?.resize_array(len);

        Ok(())
    }

    pub fn shallow_clone(&self, realm: &mut Realm) -> Res<Self> {
        let array = Self::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let other_array = &self.inner.try_borrow()?;

        let mut inner = array.inner.try_borrow_mut()?;

        for (idx, value) in &other_array.array {
            let Some(value) = other_array.values.get(*value) else {
                continue;
            };

            let len = inner.values.len();
            inner.values.push(value.clone());

            inner.array.push((*idx, len));
        }

        drop(inner);

        array
            .inner
            .try_borrow_mut()?
            .array
            .clone_from(&self.inner.try_borrow()?.array);

        array.length.set(self.length.get());

        Ok(array)
    }
}

/// IsConcatSpreadable ( O )
/// Returns true if the object should be spread during Array.prototype.concat
fn is_concat_spreadable(obj: &ObjectHandle, realm: &mut Realm) -> Res<bool> {
    // 1. If O is not an Object, return false.

    // 2. Let spreadable be ? Get(O, @@isConcatSpreadable).
    let spreadable = obj.get_opt(crate::Symbol::IS_CONCAT_SPREADABLE, realm)?;

    // 3. If spreadable is not undefined, return ToBoolean(spreadable).
    if let Some(spreadable) = spreadable {
        if !matches!(spreadable, Value::Undefined) {
            return Ok(spreadable.is_truthy());
        }
    }

    // 4. Return ? IsArray(O).
    // Check if it's an Array instance
    let is_array = obj.downcast::<Array>().is_some();

    Ok(is_array)
}

#[must_use]
pub fn convert_index(idx: isize, len: usize) -> usize {
    if idx < 0 {
        (len as isize + idx).max(0) as usize
    } else {
        idx as usize
    }
}
#[properties_new(
    intrinsic_name(array),
    default_null(array),
    constructor(ArrayConstructor::new)
)]
impl Array {
    #[prop("length")]
    #[writable]
    pub const LENGTH: usize = 0;

    fn at(#[this] this: Value, idx: isize, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let length = this.get("length", realm)?.to_int_or_null(realm)? as usize;

        let idx = convert_index(idx, length);

        let (_, val) = this.get_array_or_done(idx, realm)?;

        Ok(val.map_or(Value::Undefined, |v| v))
    }

    fn concat(#[this] this: Value, #[realm] realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let A be ? ArraySpeciesCreate(O, 0).
        // TODO: Proper ArraySpeciesCreate with Symbol.species support
        let a = Self::from_realm(realm)?;

        // 3. Let n be 0.
        let mut n: u64 = 0;

        // 4. Prepend O to items.
        let mut items = Vec::with_capacity(args.len() + 1);
        items.push(o.clone().into()); // Use the ToObject result, not the raw this value
        items.extend(args);

        // 5. For each element E of items, do
        for e in items {
            match e {
                // 5.a. Let spreadable be ? IsConcatSpreadable(E).
                Value::Object(e_obj) if is_concat_spreadable(&e_obj, realm)? => {
                    // 5.b.i. Let len be ? LengthOfArrayLike(E).
                    let len_raw = e_obj
                        .resolve_property("length", realm)?
                        .unwrap_or(Value::Undefined)
                        .to_number(realm)?;

                    // ToLength conversion
                    let len = if len_raw.is_nan() || len_raw <= 0.0 {
                        0u64
                    } else if len_raw >= NumberConstructor::MAX_SAFE_INTEGER {
                        NumberConstructor::MAX_SAFE_INTEGER as u64
                    } else {
                        len_raw.trunc() as u64
                    };

                    // 5.b.ii. If n + len > 2^53 - 1, throw a TypeError exception.
                    if n.saturating_add(len) > NumberConstructor::MAX_SAFE_INTEGER as u64 {
                        return Err(Error::ty("Array length exceeds maximum safe integer"));
                    }

                    // 5.b.iii-iv. Repeat, while k < len
                    for k in 0..len {
                        // 5.b.iv.a. Let Pk be ! ToString(𝔽(k)).
                        // 5.b.iv.b. Let exists be ? HasProperty(E, Pk).
                        let exists = e_obj.contains_key((k as usize).into(), realm)?;

                        // 5.b.iv.c. If exists is true, then
                        if exists {
                            // 5.b.iv.c.i. Let subElement be ? Get(E, Pk).
                            let sub_element = e_obj.get(k as usize, realm)?;
                            // 5.b.iv.c.ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), subElement).
                            a.insert_array(sub_element, n as usize)?;
                        }
                        // 5.b.iv.d. Set n to n + 1. (always increment, preserving holes)
                        n += 1;
                    }
                }
                _ => {
                    // 5.c. Else (spreadable is false)
                    // 5.c.i. If n >= 2^53 - 1, throw a TypeError exception.
                    if n >= NumberConstructor::MAX_SAFE_INTEGER as u64 {
                        return Err(Error::ty("Array length exceeds maximum safe integer"));
                    }
                    // 5.c.ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), E).
                    a.insert_array(e, n as usize)?;
                    // 5.c.iii. Set n to n + 1.
                    n += 1;
                }
            }
        }

        // 6. Perform ? Set(A, "length", 𝔽(n), true).
        a.set_len(n as usize)?;

        // 7. Return A.
        Ok(a.into_value())
    }

    #[prop("copyWithin")]
    fn copy_within(
        #[this] this_val: Value,
        target: isize,
        start: isize,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this_val, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as i64;

        // 3-6. Compute to (target)
        let relative_target = target as i64;
        let mut to = if relative_target < 0 {
            (len + relative_target).max(0)
        } else {
            relative_target.min(len)
        };

        // 7-10. Compute from (start)
        let relative_start = start as i64;
        let mut from = if relative_start < 0 {
            (len + relative_start).max(0)
        } else {
            relative_start.min(len)
        };

        // 11-14. Compute final (end)
        let relative_end = end.map(|e| e as i64).unwrap_or(len);
        let final_val = if relative_end < 0 {
            (len + relative_end).max(0)
        } else {
            relative_end.min(len)
        };

        // 15. Let count be min(final - from, len - to).
        let mut count = (final_val - from).min(len - to);

        // 16-18. Determine direction
        let direction: i64 = if from < to && to < from + count {
            // Copy backwards to avoid overwriting source before it's read
            from = from + count - 1;
            to = to + count - 1;
            -1
        } else {
            1
        };

        // 19. Repeat, while count > 0,
        while count > 0 {
            let from_key = from as usize;
            let to_key = to as usize;

            // 19.c. Let fromPresent be ? HasProperty(O, fromKey).
            let from_present = o.contains_key(from_key.into(), realm)?;

            if from_present {
                // 19.d. If fromPresent is true, then
                // 19.d.i. Let fromValue be ? Get(O, fromKey).
                let from_value = o.get(from_key, realm)?;
                // 19.d.ii. Perform ? Set(O, toKey, fromValue, true).
                o.set(to_key.to_string(), from_value, realm)?;
            } else {
                // 19.e. Else, Perform ? DeletePropertyOrThrow(O, toKey).
                o.delete_property(to_key.to_string().into(), realm)?;
            }

            from = (from as i64 + direction) as i64;
            to = (to as i64 + direction) as i64;
            count -= 1;
        }

        // 20. Return O.
        Ok(o.into())
    }

    fn entries(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .array_iter
                        .get(realm)?
                        .clone(),
                ),
            }),
            array: this,
            next: Cell::new(0),
            done: Cell::new(false),
            kind: ArrayIteratorKind::Entries,
        };

        Ok(iter.into_value())
    }

    fn every(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter)

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(k.into(), realm)?;

            // 5.c. If kPresent is true, then
            if k_present {
                // 5.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, realm)?;

                // 5.c.ii. Let testResult be ToBoolean(? Call(callback, thisArg, « kValue, 𝔽(k), O »)).
                let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
                let test_result =
                    func.call(vec![k_value, k.into(), o.clone().into()], this_arg, realm)?;

                // 5.c.iii. If testResult is false, return false.
                if test_result.is_falsey() {
                    return Ok(Value::Boolean(false));
                }
            }
            // 5.d. Set k to k + 1.
        }

        // 6. Return true.
        Ok(Value::Boolean(true))
    }

    fn fill(
        #[this] this_val: Value,
        value: &Value,
        start: Option<isize>,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this_val, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        let start = start.map_or(0, |start| convert_index(start, len));
        let end = end.map_or(len, |end| convert_index(end, len));

        for idx in start..end {
            this.define_property(idx.into(), value.clone(), realm)?;
        }

        Ok(this.into())
    }

    fn filter(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter, but we should verify)

        // 4. Let A be ? ArraySpeciesCreate(O, 0).
        // TODO: Implement proper ArraySpeciesCreate with Symbol.species support
        let a = Self::from_realm(realm)?;

        // 5. Let k be 0.
        // 6. Let to be 0.
        // (to is handled implicitly by a.push())

        // 7. Repeat, while k < len,
        for k in 0..len {
            // 7.a. Let Pk be ! ToString(𝔽(k)).
            // 7.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(k.into(), realm)?;

            // 7.c. If kPresent is true, then
            if k_present {
                // 7.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, realm)?;

                // 7.c.ii. Let selected be ToBoolean(? Call(callback, thisArg, « kValue, 𝔽(k), O »)).
                let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
                let selected = func.call(
                    vec![k_value.clone(), k.into(), o.clone().into()],
                    this_arg,
                    realm,
                )?;

                // 7.c.iii. If selected is true, then
                if selected.is_truthy() {
                    // 7.c.iii.1. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(to)), kValue).
                    // 7.c.iii.2. Set to to to + 1.
                    a.push(k_value)?;
                }
            }
            // 7.d. Set k to k + 1.
        }

        // 8. Return A.
        Ok(a.into_value())
    }

    fn find(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(predicate) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter)

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kValue be ? Get(O, Pk).
            // Note: find/findIndex always call Get, even for holes (returns undefined)
            let k_value = o.get(k, realm)?;

            // 5.c. Let testResult be ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
            let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
            let test_result = func.call(
                vec![k_value.clone(), k.into(), o.clone().into()],
                this_arg,
                realm,
            )?;

            // 5.d. If testResult is true, return kValue.
            if test_result.is_truthy() {
                return Ok(k_value);
            }
            // 5.e. Set k to k + 1.
        }

        // 6. Return undefined.
        Ok(Value::Undefined)
    }

    #[prop("findIndex")]
    fn find_index(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(predicate) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter)

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kValue be ? Get(O, Pk).
            // Note: find/findIndex always call Get, even for holes (returns undefined)
            let k_value = o.get(k, realm)?;

            // 5.c. Let testResult be ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
            let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
            let test_result =
                func.call(vec![k_value, k.into(), o.clone().into()], this_arg, realm)?;

            // 5.d. If testResult is true, return 𝔽(k).
            if test_result.is_truthy() {
                return Ok(k.into());
            }
            // 5.e. Set k to k + 1.
        }

        // 6. Return -1𝔽.
        Ok(Value::Number(-1.0))
    }

    #[prop("findLast")]
    fn find_last(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(predicate) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter)

        // 4. Let k be len - 1.
        // 5. Repeat, while k >= 0,
        for k in (0..len).rev() {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kValue be ? Get(O, Pk).
            // Note: findLast/findLastIndex always call Get, even for holes (returns undefined)
            let k_value = o.get(k, realm)?;

            // 5.c. Let testResult be ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
            let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
            let test_result = func.call(
                vec![k_value.clone(), k.into(), o.clone().into()],
                this_arg,
                realm,
            )?;

            // 5.d. If testResult is true, return kValue.
            if test_result.is_truthy() {
                return Ok(k_value);
            }
            // 5.e. Set k to k - 1.
        }

        // 6. Return undefined.
        Ok(Value::Undefined)
    }

    #[prop("findLastIndex")]
    fn find_last_index(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(predicate) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter)

        // 4. Let k be len - 1.
        // 5. Repeat, while k >= 0,
        for k in (0..len).rev() {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kValue be ? Get(O, Pk).
            // Note: findLast/findLastIndex always call Get, even for holes (returns undefined)
            let k_value = o.get(k, realm)?;

            // 5.c. Let testResult be ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
            let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
            let test_result =
                func.call(vec![k_value, k.into(), o.clone().into()], this_arg, realm)?;

            // 5.d. If testResult is true, return 𝔽(k).
            if test_result.is_truthy() {
                return Ok(k.into());
            }
            // 5.e. Set k to k - 1.
        }

        // 6. Return -1𝔽.
        Ok(Value::Number(-1.0))
    }

    fn flat(#[this] this: Value, #[realm] realm: &mut Realm, depth: Option<isize>) -> ValueResult {
        fn flatten(array: &Array, realm: &mut Realm, val: Value, depth: isize) -> Res {
            // Per spec: If depth > 0, check if element is array and flatten
            // If depth <= 0, don't flatten, just push the element
            if depth <= 0 {
                array.push(val)?;
                return Ok(());
            }

            if let Value::Object(obj) = &val {
                if obj.contains_key("length".into(), realm)? {
                    let iter = ValueIterator::new(&val, realm)?;

                    while let Some(val) = iter.next(realm)? {
                        // Use saturating_sub to prevent overflow
                        flatten(array, realm, val, depth.saturating_sub(1))?;
                    }
                    return Ok(());
                }
            }

            array.push(val)?;

            Ok(())
        }
        let this = coerce_object_strict(this, realm)?;

        let array = Self::from_realm(realm)?;

        // Per spec step 4.2: If depthNum < 0, set depthNum to 0
        let depth = depth.unwrap_or(1).max(0);

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in 0..len {
            // Use HasProperty+Get pattern to properly handle holes
            let k_present = this.contains_key(idx.into(), realm)?;
            if k_present {
                let val = this.get(idx, realm)?;
                flatten(&array, realm, val, depth)?;
            }
        }

        Ok(array.into_value())
    }

    #[prop("flatMap")]
    fn flat_map(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let array = Self::from_realm(realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;
        let this_arg = this_arg.unwrap_or(Value::Undefined);

        for idx in 0..len {
            // Use HasProperty+Get pattern to properly handle holes
            let k_present = this.contains_key(idx.into(), realm)?;

            if k_present {
                let val = this.get(idx, realm)?;
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    this_arg.clone(),
                    realm,
                )?;

                if let Value::Object(obj) = &x {
                    if obj.contains_key("length".into(), realm)? {
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
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        if !func.is_callable() {
            return Err(Error::ty("Callback is not callable"));
        }

        let this_arg = this_arg.unwrap_or(Value::Undefined);

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(k.into(), realm)?;

            // 5.c. If kPresent is true, then
            if k_present {
                // 5.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, realm)?;

                // 5.c.ii. Perform ? Call(callback, thisArg, « kValue, 𝔽(k), O »).
                func.call(
                    vec![k_value, k.into(), o.clone().into()],
                    this_arg.clone(),
                    realm,
                )?;
            }
            // 5.d. Set k to k + 1.
        }

        // 6. Return undefined.
        Ok(Value::Undefined)
    }

    fn includes(
        #[this] this: Value,
        search_element: &Value,
        from_index: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If len = 0, return false.
        if len == 0 {
            return Ok(Value::Boolean(false));
        }

        // 4. Let n be ? ToIntegerOrInfinity(fromIndex).
        // 5. Assert: If fromIndex is undefined, then n is 0.
        let n = from_index.unwrap_or(0);

        // 6. If n = +∞, return false.
        // (isize can't be infinity, so skip this)

        // 7. Else if n = -∞, set n to 0.
        // 8-9. Compute k based on n
        let k = convert_index(n, len);

        // 10. Repeat, while k < len,
        for idx in k..len {
            // a. Let elementK be ? Get(O, ! ToString(𝔽(k))).
            // Note: Get returns undefined for missing properties (holes)
            let element_k = o.get(idx.to_string(), realm)?;

            // b. If SameValueZero(searchElement, elementK) is true, return true.
            if search_element.same_value_zero(&element_k) {
                return Ok(Value::Boolean(true));
            }
            // c. Set k to k + 1.
        }

        // 11. Return false.
        Ok(Value::Boolean(false))
    }

    #[prop("indexOf")]
    fn index_of(
        #[this] this: Value,
        search_element: &Value,
        from_index: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If len = 0, return -1.
        if len == 0 {
            return Ok(Value::Number(-1.0));
        }

        // 4-9. Compute k
        let from_index = from_index.unwrap_or(0);
        let k = convert_index(from_index, len);

        // 10. Repeat, while k < len,
        for idx in k..len {
            let pk = idx.to_string();

            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(pk.clone().into(), realm)?;

            // c. If kPresent is true, then
            if k_present {
                // i. Let elementK be ? Get(O, Pk).
                let element_k = o.get(pk, realm)?;

                // ii. If IsStrictlyEqual(searchElement, elementK) is true, return 𝔽(k).
                if search_element == &element_k {
                    return Ok(idx.into());
                }
            }
            // d. Set k to k + 1.
        }

        // 11. Return -1.
        Ok(Value::Number(-1.0))
    }

    fn join(#[this] this: Value, #[realm] realm: &mut Realm, separator: &Value) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToLength conversion
        let len = if len.is_nan() || len <= 0.0 {
            0usize
        } else {
            (len.trunc() as usize).min(isize::MAX as usize)
        };

        // 3. If separator is undefined, let sep be ",".
        // 4. Else, let sep be ? ToString(separator).
        let sep = if separator.is_undefined() {
            YSString::new_static(",")
        } else {
            separator.to_string(realm)?
        };

        // 5. Let R be the empty String.
        let mut r = String::new();

        // 6. Let k be 0.
        // 7. Repeat, while k < len,
        for k in 0..len {
            // 7.a. If k > 0, set R to the string-concatenation of R and sep.
            if k > 0 {
                r.push_str(&sep);
            }

            // 7.b. Let element be ? Get(O, ! ToString(𝔽(k))).
            let element = o.get(k, realm)?;

            // 7.c. If element is neither undefined nor null, then
            if !element.is_undefined() && !element.is_null() {
                // 7.c.i. Let S be ? ToString(element).
                // 7.c.ii. Set R to the string-concatenation of R and S.
                r.push_str(&element.to_string(realm)?);
            }
            // 7.d. Set k to k + 1.
        }

        // 8. Return R.
        Ok(r.into())
    }

    #[prop("toLocaleString")]
    fn to_locale_string(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let array be ? ToObject(this value).
        let array = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(array).
        let len = array
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToLength conversion
        let len = if len.is_nan() || len <= 0.0 {
            0usize
        } else {
            (len.trunc() as usize).min(isize::MAX as usize)
        };

        // 3. Let separator be the implementation-defined list-separator String value appropriate for the host environment's current locale.
        // For simplicity, use ", " as the separator
        let sep = ",";

        // 4. Let R be the empty String.
        let mut r = String::new();

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // 6.a. If k > 0, set R to the string-concatenation of R and separator.
            if k > 0 {
                r.push_str(sep);
            }

            // 6.b. Let element be ? Get(array, ! ToString(𝔽(k))).
            let element = array.get(k, realm)?;

            // 6.c. If element is neither undefined nor null, then
            if !element.is_undefined() && !element.is_null() {
                // 6.c.i. Let S be ? ToString(? Invoke(element, "toLocaleString")).
                let element_obj = element.clone().to_object()?;
                let to_locale_string = element_obj.get("toLocaleString", realm)?;
                let s = if to_locale_string.is_callable() {
                    let func = to_locale_string.as_object()?;
                    func.call(vec![], element, realm)?
                } else {
                    element
                };
                let s_str = s.to_string(realm)?;

                // 6.c.ii. Set R to the string-concatenation of R and S.
                r.push_str(&s_str);
            }
            // 6.d. Set k to k + 1.
        }

        // 7. Return R.
        Ok(r.into())
    }

    fn keys(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .array_iter
                        .get(realm)?
                        .clone(),
                ),
            }),
            array: this,
            next: Cell::new(0),
            done: Cell::new(false),
            kind: ArrayIteratorKind::Keys,
        };

        Ok(iter.into_value())
    }

    #[prop("lastIndexOf")]
    fn last_index_of(
        #[this] this: Value,
        args: Vec<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Get search_element (first argument)
        let search_element = args.first().cloned().unwrap_or(Value::Undefined);

        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // Handle NaN -> 0
        let len = if len.is_nan() { 0.0 } else { len };
        let len = len as usize;

        // 3. If len = 0, return -1𝔽.
        if len == 0 {
            return Ok(Value::Number(-1.0));
        }

        // 4. If fromIndex is present, let n be ? ToIntegerOrInfinity(fromIndex); else let n be len - 1.
        // Note: We check args.len() >= 2 to distinguish between "not provided" and "provided as undefined"
        let n: f64 = if args.len() >= 2 {
            let from_index = &args[1];
            let n = from_index.to_number(realm)?;
            if n.is_nan() {
                0.0
            } else if n == 0.0 || n.is_infinite() {
                n
            } else {
                n.trunc()
            }
        } else {
            (len - 1) as f64
        };

        // 5. If n = -∞, return -1𝔽.
        if n == f64::NEG_INFINITY {
            return Ok(Value::Number(-1.0));
        }

        // 6. If n ≥ 0, then
        //    a. Let k be min(n, len - 1).
        // 7. Else,
        //    a. Let k be len + n.
        let k: isize = if n >= 0.0 {
            n.min((len - 1) as f64) as isize
        } else {
            (len as f64 + n) as isize
        };

        // If k < 0, return -1
        if k < 0 {
            return Ok(Value::Number(-1.0));
        }

        let mut k = k as usize;

        // 8. Repeat, while k ≥ 0,
        loop {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kPresent be ? HasProperty(O, Pk).
            if o.contains_key(k.into(), realm)? {
                // c. If kPresent is true, then
                //    i. Let elementK be ? Get(O, Pk).
                let element_k = o.get(k.to_string(), realm)?;
                //    ii. If IsStrictlyEqual(searchElement, elementK) is true, return 𝔽(k).
                if element_k == search_element {
                    return Ok(Value::Number(k as f64));
                }
            }
            // d. Set k to k - 1.
            if k == 0 {
                break;
            }
            k -= 1;
        }

        // 9. Return -1𝔽.
        Ok(Value::Number(-1.0))
    }

    fn map(
        #[this] this: Value,
        callback: Value,
        this_arg: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        if !callback.is_callable() {
            return Err(Error::ty("callback is not a function"));
        }

        // 4. Let A be ? ArraySpeciesCreate(O, len).
        // TODO: Implement proper ArraySpeciesCreate with Symbol.species support
        let a = Self::with_len(realm, len)?;

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // 6.a. Let Pk be ! ToString(𝔽(k)).
            let pk = k.into();

            // 6.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(pk, realm)?;

            // 6.c. If kPresent is true, then
            if k_present {
                // 6.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, realm)?;

                // 6.c.ii. Let mappedValue be ? Call(callback, thisArg, « kValue, 𝔽(k), O »).
                let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
                let mapped_value =
                    callback.call(realm, vec![k_value, k.into(), o.clone().into()], this_arg)?;

                // 6.c.iii. Perform ? CreateDataPropertyOrThrow(A, Pk, mappedValue).
                a.insert_array(mapped_value, k)?;
            }
            // 6.d. Set k to k + 1.
        }

        // 7. Return A.
        Ok(a.into_value())
    }

    fn pop(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        // LengthOfArrayLike calls ToLength(Get(O, "length"))
        const MAX_SAFE_INTEGER: u64 = 9007199254740991; // 2^53 - 1
        let len_raw = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToLength conversion
        let len = if len_raw.is_nan() || len_raw <= 0.0 {
            0u64
        } else if len_raw >= MAX_SAFE_INTEGER as f64 {
            MAX_SAFE_INTEGER
        } else {
            len_raw.trunc() as u64
        };

        // 3. If len = 0, then
        if len == 0 {
            // 3.a. Perform ? Set(O, "length", +0, true).
            o.set("length", Value::Number(0.0), realm)?;
            // 3.b. Return undefined.
            return Ok(Value::Undefined);
        }

        // 4. Else,
        // 4.a. Assert: len > 0.
        // 4.b. Let newLen be 𝔽(len - 1).
        let new_len = len - 1;

        // 4.c. Let index be ! ToString(newLen).
        let index = new_len.to_string();

        // 4.d. Let element be ? Get(O, index).
        let element = o.get(index.clone(), realm)?;

        // 4.e. Perform ? DeletePropertyOrThrow(O, index).
        o.delete_property(index.into(), realm)?;

        // 4.f. Perform ? Set(O, "length", newLen, true).
        o.set("length", Value::Number(new_len as f64), realm)?;

        // 4.g. Return element.
        Ok(element)
    }

    #[prop("push")]
    fn push_js(
        #[this] this: Value,
        #[variadic] args: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        // LengthOfArrayLike calls ToLength(Get(O, "length"))
        // ToLength: 1. let len = ToIntegerOrInfinity(argument)
        //           2. If len <= 0, return 0
        //           3. Return min(len, 2^53 - 1)
        const MAX_SAFE_INTEGER: u64 = 9007199254740991; // 2^53 - 1
        let len_raw = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToIntegerOrInfinity: NaN, +0, -0 -> 0; +inf -> +inf; -inf -> -inf; else truncate
        let len = if len_raw.is_nan() || len_raw == 0.0 {
            0u64
        } else if len_raw == f64::INFINITY {
            MAX_SAFE_INTEGER
        } else if len_raw == f64::NEG_INFINITY || len_raw < 0.0 {
            0u64
        } else {
            (len_raw.trunc() as u64).min(MAX_SAFE_INTEGER)
        };

        // 3. Let argCount be the number of elements in items.
        let arg_count = args.len() as u64;

        // 4. If len + argCount > 2^53 - 1, throw a TypeError exception.
        if len.saturating_add(arg_count) > MAX_SAFE_INTEGER {
            return Err(Error::ty("Array length exceeds maximum safe integer"));
        }

        // 5. For each element E of items, do
        let mut idx = len;
        for arg in args {
            // 5.a. Perform ? Set(O, ! ToString(𝔽(len)), E, true).
            o.set(idx.to_string(), arg.clone(), realm)?;
            // 5.b. Set len to len + 1.
            idx += 1;
        }

        // 6. Perform ? Set(O, "length", 𝔽(len), true).
        o.set("length", Value::Number(idx as f64), realm)?;

        // 7. Return 𝔽(len).
        Ok(Value::Number(idx as f64))
    }

    fn reduce(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        #[variadic] args: &[Value],
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // Get callback (first argument)
        let callback = args.first().cloned().unwrap_or(Value::Undefined);

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        if !callback.is_callable() {
            return Err(Error::ty("Reduce callback is not a function"));
        }

        // Check if initial value was provided (args.len() > 1 means it was passed, even if undefined)
        let initial_value_present = args.len() > 1;

        // 4. If len = 0 and initialValue is not present, throw a TypeError exception.
        if len == 0 && !initial_value_present {
            return Err(Error::ty("Reduce of empty array with no initial value"));
        }

        // 5. Let k be 0.
        let mut k = 0usize;

        // 6. Let accumulator be undefined.
        let mut accumulator = Value::Undefined;

        // 7. If initialValue is present, then
        if initial_value_present {
            // 7.a. Set accumulator to initialValue.
            accumulator = args.get(1).cloned().unwrap_or(Value::Undefined);
        } else {
            // 8. Else,
            let mut k_present = false;

            while !k_present && k < len {
                let pk = k.into();

                k_present = o.contains_key(pk, realm)?;

                if k_present {
                    accumulator = o.get(k, realm)?;
                }

                k += 1;
            }

            if !k_present {
                return Err(Error::ty("Reduce of empty array with no initial value"));
            }
        }

        while k < len {
            let pk = k.into();

            let k_present = o.contains_key(pk, realm)?;

            if k_present {
                let k_value = o.get(k, realm)?;

                accumulator = callback.call(
                    realm,
                    vec![accumulator, k_value, k.into(), o.clone().into()],
                    Value::Undefined,
                )?;
            }

            k += 1;
        }

        Ok(accumulator)
    }

    #[prop("reduceRight")]
    fn reduce_right(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        #[variadic] args: &[Value],
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // Get callback (first argument)
        let callback = args.first().cloned().unwrap_or(Value::Undefined);

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        if !callback.is_callable() {
            return Err(Error::ty("ReduceRight callback is not a function"));
        }

        // Check if initial value was provided (args.len() > 1 means it was passed, even if undefined)
        let initial_value_present = args.len() > 1;

        // 4. If len = 0 and initialValue is not present, throw a TypeError exception.
        if len == 0 && !initial_value_present {
            return Err(Error::ty("Reduce of empty array with no initial value"));
        }

        // 5. Let k be len - 1.
        // Using isize to handle the k >= 0 check properly
        let mut k = len as isize - 1;

        // 6. Let accumulator be undefined.
        let mut accumulator = Value::Undefined;

        // 7. If initialValue is present, then
        if initial_value_present {
            // 7.a. Set accumulator to initialValue.
            accumulator = args.get(1).cloned().unwrap_or(Value::Undefined);
        } else {
            // 8. Else,
            // 8.a. Let kPresent be false.
            let mut k_present = false;

            // 8.b. Repeat, while kPresent is false and k >= 0,
            while !k_present && k >= 0 {
                // 8.b.i. Let Pk be ! ToString(𝔽(k)).
                let pk = (k as usize).into();

                // 8.b.ii. Set kPresent to ? HasProperty(O, Pk).
                k_present = o.contains_key(pk, realm)?;

                // 8.b.iii. If kPresent is true, then
                if k_present {
                    // 8.b.iii.1. Set accumulator to ? Get(O, Pk).
                    accumulator = o.get(k as usize, realm)?;
                }

                // 8.b.iv. Set k to k - 1.
                k -= 1;
            }

            // 8.c. If kPresent is false, throw a TypeError exception.
            if !k_present {
                return Err(Error::ty("Reduce of empty array with no initial value"));
            }
        }

        // 9. Repeat, while k >= 0,
        while k >= 0 {
            // 9.a. Let Pk be ! ToString(𝔽(k)).
            let pk = (k as usize).into();

            // 9.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(pk, realm)?;

            // 9.c. If kPresent is true, then
            if k_present {
                // 9.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k as usize, realm)?;

                // 9.c.ii. Set accumulator to ? Call(callback, undefined, « accumulator, kValue, 𝔽(k), O »).
                accumulator = callback.call(
                    realm,
                    vec![accumulator, k_value, (k as usize).into(), o.clone().into()],
                    Value::Undefined,
                )?;
            }

            // 9.d. Set k to k - 1.
            k -= 1;
        }

        // 10. Return accumulator.
        Ok(accumulator)
    }

    fn reverse(#[this] this_val: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this_val, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToLength conversion
        let len = if len.is_nan() || len <= 0.0 {
            0u64
        } else if len >= 9007199254740991.0 {
            9007199254740991u64
        } else {
            len.trunc() as u64
        };

        // 3. Let middle be floor(len / 2).
        let middle = len / 2;

        // 4. Let lower be 0.
        let mut lower = 0u64;

        // 5. Repeat, while lower ≠ middle,
        while lower != middle {
            // 5.a. Let upper be len - lower - 1.
            let upper = len - lower - 1;

            // 5.b-c. Let upperP and lowerP be the string representations.
            let upper_p = upper.to_string();
            let lower_p = lower.to_string();

            // 5.d. Let lowerExists be ? HasProperty(O, lowerP).
            let lower_exists = o.contains_key(lower_p.clone().into(), realm)?;

            // 5.e. If lowerExists is true, then let lowerValue be ? Get(O, lowerP).
            let lower_value = if lower_exists {
                Some(o.get(lower_p.clone(), realm)?)
            } else {
                None
            };

            // 5.f. Let upperExists be ? HasProperty(O, upperP).
            let upper_exists = o.contains_key(upper_p.clone().into(), realm)?;

            // 5.g. If upperExists is true, then let upperValue be ? Get(O, upperP).
            let upper_value = if upper_exists {
                Some(o.get(upper_p.clone(), realm)?)
            } else {
                None
            };

            // 5.h. If lowerExists is true and upperExists is true, then
            if lower_exists && upper_exists {
                // 5.h.i. Perform ? Set(O, lowerP, upperValue, true).
                o.set(lower_p, upper_value.unwrap(), realm)?;
                // 5.h.ii. Perform ? Set(O, upperP, lowerValue, true).
                o.set(upper_p, lower_value.unwrap(), realm)?;
            }
            // 5.i. Else if lowerExists is false and upperExists is true, then
            else if !lower_exists && upper_exists {
                // 5.i.i. Perform ? Set(O, lowerP, upperValue, true).
                o.set(lower_p, upper_value.unwrap(), realm)?;
                // 5.i.ii. Perform ? DeletePropertyOrThrow(O, upperP).
                o.delete_property(upper_p.into(), realm)?;
            }
            // 5.j. Else if lowerExists is true and upperExists is false, then
            else if lower_exists && !upper_exists {
                // 5.j.i. Perform ? DeletePropertyOrThrow(O, lowerP).
                o.delete_property(lower_p.into(), realm)?;
                // 5.j.ii. Perform ? Set(O, upperP, lowerValue, true).
                o.set(upper_p, lower_value.unwrap(), realm)?;
            }
            // 5.k. Else (both don't exist - no action needed)

            // 5.l. Set lower to lower + 1.
            lower += 1;
        }

        // 6. Return O.
        Ok(o.into())
    }

    fn shift(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len_raw = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToLength conversion
        let len = if len_raw.is_nan() || len_raw <= 0.0 {
            0u64
        } else if len_raw >= 9007199254740991.0 {
            9007199254740991u64
        } else {
            len_raw.trunc() as u64
        };

        // 3. If len = 0, then
        if len == 0 {
            // 3.a. Perform ? Set(O, "length", +0, true).
            o.set("length", Value::Number(0.0), realm)?;
            // 3.b. Return undefined.
            return Ok(Value::Undefined);
        }

        // 4. Let first be ? Get(O, "0").
        let first = o.get(0usize, realm)?;

        // 5. Let k be 1.
        // 6. Repeat, while k < len,
        for k in 1..len {
            // 6.a. Let from be ! ToString(𝔽(k)).
            let from = k.to_string();
            // 6.b. Let to be ! ToString(𝔽(k - 1)).
            let to = (k - 1).to_string();

            // 6.c. Let fromPresent be ? HasProperty(O, from).
            let from_present = o.contains_key(from.clone().into(), realm)?;

            // 6.d. If fromPresent is true, then
            if from_present {
                // 6.d.i. Let fromValue be ? Get(O, from).
                let from_value = o.get(from, realm)?;
                // 6.d.ii. Perform ? Set(O, to, fromValue, true).
                o.set(to, from_value, realm)?;
            } else {
                // 6.e. Else,
                // 6.e.i. Assert: fromPresent is false.
                // 6.e.ii. Perform ? DeletePropertyOrThrow(O, to).
                o.delete_property(to.into(), realm)?;
            }
            // 6.f. Set k to k + 1.
        }

        // 7. Perform ? DeletePropertyOrThrow(O, ! ToString(𝔽(len - 1))).
        o.delete_property((len - 1).to_string().into(), realm)?;

        // 8. Perform ? Set(O, "length", 𝔽(len - 1), true).
        o.set("length", Value::Number((len - 1) as f64), realm)?;

        // 9. Return first.
        Ok(first)
    }

    #[length(2)]
    fn slice(
        #[this] this: Value,
        start: Option<Value>,
        end: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // Convert to i64 for proper handling of large values
        let len = if len.is_nan() || len <= 0.0 {
            0i64
        } else if len >= 9007199254740991.0 {
            9007199254740991i64 // 2^53 - 1
        } else {
            len.trunc() as i64
        };

        // 3. Let relativeStart be ? ToIntegerOrInfinity(start).
        let relative_start = match &start {
            None => 0i64,
            Some(v) if v.is_undefined() => 0i64,
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n == 0.0 {
                    0i64
                } else if n == f64::INFINITY {
                    i64::MAX
                } else if n == f64::NEG_INFINITY {
                    i64::MIN
                } else {
                    n.trunc() as i64
                }
            }
        };

        // 4. If relativeStart = -∞, let k be 0.
        // 5. Else if relativeStart < 0, let k be max(len + relativeStart, 0).
        // 6. Else, let k be min(relativeStart, len).
        let k = if relative_start == i64::MIN {
            0i64
        } else if relative_start < 0 {
            (len + relative_start).max(0)
        } else {
            relative_start.min(len)
        };

        // 7. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let relative_end = match &end {
            None => len,
            Some(v) if v.is_undefined() => len,
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n == 0.0 {
                    0i64
                } else if n == f64::INFINITY {
                    i64::MAX
                } else if n == f64::NEG_INFINITY {
                    i64::MIN
                } else {
                    n.trunc() as i64
                }
            }
        };

        // 8. If relativeEnd = -∞, let final be 0.
        // 9. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
        // 10. Else, let final be min(relativeEnd, len).
        let final_idx = if relative_end == i64::MIN {
            0i64
        } else if relative_end < 0 {
            (len + relative_end).max(0)
        } else {
            relative_end.min(len)
        };

        // 11. Let count be max(final - k, 0).
        let count = (final_idx - k).max(0) as u64;

        // 12. Let A be ? ArraySpeciesCreate(O, count).
        // For now, just create a regular array. Throw RangeError if count > 2^32 - 1.
        const MAX_ARRAY_LENGTH: u64 = 4294967295; // 2^32 - 1
        if count > MAX_ARRAY_LENGTH {
            return Err(Error::range("Invalid array length"));
        }

        // TODO: Implement proper ArraySpeciesCreate with Symbol.species support
        let array = Self::from_realm(realm)?;

        // 13. Let n be 0.
        let mut n = 0u64;

        // 14. Repeat, while k < final,
        let mut k = k as u64;
        let final_idx = final_idx as u64;
        while k < final_idx {
            // 14.a. Let Pk be ! ToString(𝔽(k)).
            // 14.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key((k as usize).into(), realm)?;

            // 14.c. If kPresent is true, then
            if k_present {
                // 14.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k as usize, realm)?;
                // 14.c.ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), kValue).
                array.insert_array(k_value, n as usize)?;
            }
            // 14.d. Set k to k + 1.
            k += 1;
            // 14.e. Set n to n + 1.
            n += 1;
        }

        // 15. Perform ? Set(A, "length", 𝔽(n), true).
        array.set_len(n as usize)?;

        // 16. Return A.
        Ok(array.into_value())
    }

    fn some(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. If IsCallable(callback) is false, throw a TypeError exception.
        // (Already ensured by func: &ObjectHandle parameter)

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // 5.a. Let Pk be ! ToString(𝔽(k)).
            // 5.b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.contains_key(k.into(), realm)?;

            // 5.c. If kPresent is true, then
            if k_present {
                // 5.c.i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, realm)?;

                // 5.c.ii. Let testResult be ToBoolean(? Call(callback, thisArg, « kValue, 𝔽(k), O »)).
                let this_arg = this_arg.clone().unwrap_or(Value::Undefined);
                let test_result =
                    func.call(vec![k_value, k.into(), o.clone().into()], this_arg, realm)?;

                // 5.c.iii. If testResult is true, return true.
                if test_result.is_truthy() {
                    return Ok(Value::Boolean(true));
                }
            }
            // 5.d. Set k to k + 1.
        }

        // 6. Return false.
        Ok(Value::Boolean(false))
    }

    #[length(1)]
    fn sort(
        #[this] this_val: Value,
        compare: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. If comparator is not undefined and IsCallable(comparator) is false, throw TypeError.
        // (Already ensured by Option<ObjectHandle>)

        // 2. Let obj be ? ToObject(this value).
        let obj = coerce_object_strict(this_val, realm)?;

        // 3. Let len be ? LengthOfArrayLike(obj).
        let len = obj
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 5. Let sortedList be ? SortIndexedProperties(obj, len, SortCompare, skip-holes).
        // SortIndexedProperties with skip-holes: only include elements that exist
        let mut items = Vec::new();

        for k in 0..len {
            // Check HasProperty (skip holes)
            if obj.contains_key(k.into(), realm)? {
                let k_value = obj.get(k, realm)?;
                items.push(k_value);
            }
        }

        // Sort the items
        if let Some(compare) = compare {
            items.sort_by(|a, b| {
                // CompareArrayElements with comparator
                // If x and y are both undefined, return +0
                if matches!(a, Value::Undefined) && matches!(b, Value::Undefined) {
                    return Ordering::Equal;
                }
                // If x is undefined, return 1 (x > y)
                if matches!(a, Value::Undefined) {
                    return Ordering::Greater;
                }
                // If y is undefined, return -1 (x < y)
                if matches!(b, Value::Undefined) {
                    return Ordering::Less;
                }

                let x = compare
                    .call(vec![a.clone(), b.clone()], Value::Undefined, realm)
                    .unwrap_or(Value::Number(0.0));

                let n = x.as_number();
                if n.is_nan() {
                    return Ordering::Equal;
                }
                n.partial_cmp(&0.0).unwrap_or(Ordering::Equal)
            });
        } else {
            // Default sort: compare as strings, undefined goes to end
            items.sort_by(|a, b| {
                // If x and y are both undefined, return +0
                if matches!(a, Value::Undefined) && matches!(b, Value::Undefined) {
                    return Ordering::Equal;
                }
                // If x is undefined, return 1 (x > y)
                if matches!(a, Value::Undefined) {
                    return Ordering::Greater;
                }
                // If y is undefined, return -1 (x < y)
                if matches!(b, Value::Undefined) {
                    return Ordering::Less;
                }

                let x_str = a.to_string(realm).unwrap_or_default();
                let y_str = b.to_string(realm).unwrap_or_default();
                x_str.cmp(&y_str)
            });
        }

        // 6. Let itemCount be the number of elements in sortedList.
        let item_count = items.len();

        // 7-9. Write sorted items back
        for (j, item) in items.into_iter().enumerate() {
            obj.set(j.to_string(), item, realm)?;
        }

        // 10-11. Delete remaining properties to preserve holes
        for j in item_count..len {
            obj.delete_property(j.to_string().into(), realm)?;
        }

        // 12. Return obj.
        Ok(obj.into())
    }

    #[length(2)]
    fn splice(#[this] this: Value, args: Vec<Value>, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as i64;

        // Extract start (args[0]) and deleteCount (args[1]) if present
        // Note: We check args.len() to distinguish between "not provided" and "provided as undefined"
        let start_present = !args.is_empty();
        let delete_count_present = args.len() >= 2;

        // Helper to apply ToIntegerOrInfinity
        fn to_integer_or_infinity(n: f64) -> f64 {
            if n.is_nan() {
                0.0
            } else if n == 0.0 || n.is_infinite() {
                n
            } else {
                n.trunc()
            }
        }

        // 3-6. Compute actualStart
        let actual_start = if start_present {
            let relative_start = to_integer_or_infinity(args[0].to_number(realm)?);
            if relative_start == f64::NEG_INFINITY {
                0
            } else if relative_start < 0.0 {
                (len as f64 + relative_start).max(0.0) as usize
            } else {
                (relative_start.min(len as f64)) as usize
            }
        } else {
            0
        };

        // 7. Let itemCount be the number of elements in items (args after first two).
        let items: Vec<Value> = if args.len() > 2 {
            args[2..].to_vec()
        } else {
            Vec::new()
        };
        let item_count = items.len();

        // 8-11. Compute actualDeleteCount
        let actual_delete_count = if !start_present {
            // If start is not present, actualDeleteCount = 0
            0
        } else if !delete_count_present {
            // If deleteCount is not present, actualDeleteCount = len - actualStart
            (len as usize).saturating_sub(actual_start)
        } else {
            // deleteCount is present - use ToIntegerOrInfinity (undefined -> NaN -> 0)
            let dc = to_integer_or_infinity(args[1].to_number(realm)?);
            let dc = dc.max(0.0) as usize;
            dc.min((len as usize).saturating_sub(actual_start))
        };

        // 12. If len + itemCount - actualDeleteCount > 2^53 - 1, throw TypeError.
        let new_len = len as usize + item_count - actual_delete_count;

        // 13. Let A be ? ArraySpeciesCreate(O, actualDeleteCount).
        // TODO: Proper ArraySpeciesCreate with Symbol.species support
        let a = Self::from_realm(realm)?;

        // 14-16. Copy deleted elements to A
        for k in 0..actual_delete_count {
            let from = actual_start + k;
            // If HasProperty(O, from) is true
            if o.contains_key(from.into(), realm)? {
                let from_value = o.get(from, realm)?;
                a.push(from_value)?;
            }
        }

        // 17. Set A.length (implicitly done by push)

        // 18. If itemCount < actualDeleteCount (shrinking)
        if item_count < actual_delete_count {
            // 18.a. Set k to actualStart
            // 18.b. Repeat, while k < (len - actualDeleteCount)
            let mut k = actual_start;
            while k < (len as usize - actual_delete_count) {
                let from = k + actual_delete_count;
                let to = k + item_count;

                if o.contains_key(from.into(), realm)? {
                    let from_value = o.get(from, realm)?;
                    o.set(to.to_string(), from_value, realm)?;
                } else {
                    o.delete_property(to.to_string().into(), realm)?;
                }
                k += 1;
            }

            // 18.d-e. Delete trailing elements
            let mut k = len as usize;
            while k > new_len {
                o.delete_property((k - 1).to_string().into(), realm)?;
                k -= 1;
            }
        }
        // 19. Else if itemCount > actualDeleteCount (growing)
        else if item_count > actual_delete_count {
            // 19.a. Set k to (len - actualDeleteCount)
            // 19.b. Repeat, while k > actualStart (shift elements right, from end)
            let mut k = len as usize - actual_delete_count;
            while k > actual_start {
                let from = k + actual_delete_count - 1;
                let to = k + item_count - 1;

                if o.contains_key(from.into(), realm)? {
                    let from_value = o.get(from, realm)?;
                    o.set(to.to_string(), from_value, realm)?;
                } else {
                    o.delete_property(to.to_string().into(), realm)?;
                }
                k -= 1;
            }
        }

        // 20-22. Insert new items
        let mut k = actual_start;
        for item in items {
            o.set(k.to_string(), item, realm)?;
            k += 1;
        }

        // 23. Set O.length
        o.set("length", Value::Number(new_len as f64), realm)?;

        // 24. Return A.
        Ok(a.into_value())
    }

    #[prop("toReversed")]
    fn js_to_reversed(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let this = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        // 3. Let A be ? ArrayCreate(len).
        let array = Self::with_len(realm, len)?;

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // 5.a. Let from be ! ToString(𝔽(len - k - 1)).
            let from = len - k - 1;
            // 5.b. Let Pk be ! ToString(𝔽(k)).
            // 5.c. Let fromValue be ? Get(O, from).
            // Note: No HasProperty check - read through holes (returns undefined for holes)
            let from_value = this.get(from, realm)?;
            // 5.d. Perform ! CreateDataPropertyOrThrow(A, Pk, fromValue).
            array.insert_array(from_value, k)?;
            // 5.e. Set k to k + 1.
        }

        // 6. Return A.
        Ok(array.into_value())
    }

    #[prop("toSorted")]
    fn js_to_sorted(
        #[this] this: Value,
        func: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. If comparator is not undefined and IsCallable(comparator) is false, throw a TypeError exception.
        if let Some(ref f) = func {
            if !f.is_callable() {
                return Err(Error::ty("comparator is not a function"));
            }
        }

        // 2. Let O be ? ToObject(this value).
        let this = coerce_object_strict(this, realm)?;

        // 3. Let len be ? LengthOfArrayLike(O).
        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        // 6. Let sortedList be ? SortIndexedProperties(O, len, SortCompare, read-through-holes).
        // With read-through-holes, we always read (no HasProperty check)
        let mut values = Vec::with_capacity(len);

        for idx in 0..len {
            // No HasProperty check - read through holes (returns undefined for holes)
            let val = this.get(idx, realm)?;
            values.push(val);
        }

        // Sort using CompareArrayElements logic
        if let Some(func) = func {
            values.sort_by(|a, b| {
                // CompareArrayElements with comparator
                // 1. If x and y are both undefined, return +0.
                if matches!(a, Value::Undefined) && matches!(b, Value::Undefined) {
                    return Ordering::Equal;
                }
                // 2. If x is undefined, return 1.
                if matches!(a, Value::Undefined) {
                    return Ordering::Greater;
                }
                // 3. If y is undefined, return -1.
                if matches!(b, Value::Undefined) {
                    return Ordering::Less;
                }
                // 4. Call comparator
                let Ok(x) = func.call(realm, vec![a.clone(), b.clone()], Value::Undefined) else {
                    //TODO: this is NOT good, we can't throw the error here
                    return Ordering::Equal;
                };

                // If v is NaN, return +0.
                let num = x.as_number();
                if num.is_nan() {
                    return Ordering::Equal;
                }
                num.partial_cmp(&0.0).unwrap_or(Ordering::Equal)
            });
        } else {
            // Sort by string comparison, but undefined goes to end
            values.sort_by(|a, b| {
                // 1. If x and y are both undefined, return +0.
                if matches!(a, Value::Undefined) && matches!(b, Value::Undefined) {
                    return Ordering::Equal;
                }
                // 2. If x is undefined, return 1.
                if matches!(a, Value::Undefined) {
                    return Ordering::Greater;
                }
                // 3. If y is undefined, return -1.
                if matches!(b, Value::Undefined) {
                    return Ordering::Less;
                }
                // String comparison
                let a_str = a.to_string(realm).unwrap_or_default();
                let b_str = b.to_string(realm).unwrap_or_default();
                a_str.cmp(&b_str)
            });
        }

        // 4. Let A be ? ArrayCreate(len).
        // 7-8. Create array from sorted list
        Ok(Self::with_elements(realm, values)?.into_value())
    }

    #[prop("toSpliced")]
    fn js_to_spliced(
        #[this] this: Value,
        start: Option<isize>,
        skip_count: Option<isize>,
        items: Vec<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len_num = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        // ToLength: If len is NaN, negative, or not a finite number, treat as 0
        let len = if len_num.is_nan() || len_num < 0.0 || !len_num.is_finite() {
            0usize
        } else {
            len_num as usize
        };

        // 3-6. Compute actualStart
        let actual_start = if let Some(start) = start {
            let relative_start = start as i64;
            if relative_start < 0 {
                ((len as i64) + relative_start).max(0) as usize
            } else {
                (relative_start as usize).min(len)
            }
        } else {
            0
        };

        // 7. Let insertCount be the number of elements in items.
        let insert_count = items.len();

        // 8-11. Compute actualSkipCount
        let actual_skip_count = if start.is_none() {
            // If start is not present, actualSkipCount = 0
            0
        } else if skip_count.is_none() {
            // If skipCount is not present, actualSkipCount = len - actualStart
            len.saturating_sub(actual_start)
        } else {
            // Clamp sc between 0 and len - actualStart
            let sc = skip_count.unwrap().max(0) as usize;
            sc.min(len.saturating_sub(actual_start))
        };

        // 12. Let newLen be len + insertCount - actualSkipCount.
        let new_len = len + insert_count - actual_skip_count;

        // 13. If newLen > 2^53 - 1, throw TypeError.
        // (For practical purposes, this is unlikely to happen with usize)

        // 14. Let A be ? ArrayCreate(newLen).
        let a = Self::from_realm(realm)?;

        // 15-16. Let i be 0, r be actualStart + actualSkipCount.
        let mut i = 0usize;
        let r_start = actual_start + actual_skip_count;

        // 17. Repeat, while i < actualStart
        while i < actual_start {
            // Get from O at index i
            let i_value = o.get(i, realm)?;
            a.push(i_value)?;
            i += 1;
        }

        // 18. For each element E of items
        for item in items {
            a.push(item)?;
            i += 1;
        }

        // 19. Repeat, while i < newLen
        let mut r = r_start;
        while i < new_len {
            let from_value = o.get(r, realm)?;
            a.push(from_value)?;
            i += 1;
            r += 1;
        }

        // 20. Return A.
        Ok(a.into_value())
    }

    fn unshift(#[this] this: Value, args: Vec<Value>, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        // 3. Let argCount be the number of elements in items.
        let arg_count = args.len();

        // 4. If argCount > 0, then
        if arg_count > 0 {
            // 4.a. If len + argCount > 2^53 - 1, throw a TypeError exception.
            // (skipping overflow check for now)

            // 4.b. Let k be len.
            let mut k = len;

            // 4.c. Repeat, while k > 0,
            while k > 0 {
                // 4.c.i. Let from be ! ToString(𝔽(k - 1)).
                let from = k - 1;
                // 4.c.ii. Let to be ! ToString(𝔽(k + argCount - 1)).
                let to = k + arg_count - 1;

                // 4.c.iii. Let fromPresent be ? HasProperty(O, from).
                let from_present = o.contains_key(from.into(), realm)?;

                // 4.c.iv. If fromPresent is true, then
                if from_present {
                    // 4.c.iv.1. Let fromValue be ? Get(O, from).
                    let from_value = o.get(from, realm)?;
                    // 4.c.iv.2. Perform ? Set(O, to, fromValue, true).
                    o.define_property(to.into(), from_value, realm)?;
                } else {
                    // 4.c.v. Else,
                    // 4.c.v.1. Assert: fromPresent is false.
                    // 4.c.v.2. Perform ? DeletePropertyOrThrow(O, to).
                    o.delete_property(to.into(), realm)?;
                }

                // 4.c.vi. Set k to k - 1.
                k -= 1;
            }

            // 4.d. Let j be +0𝔽.
            // 4.e. For each element E of items, do
            for (j, e) in args.into_iter().enumerate() {
                // 4.e.i. Perform ? Set(O, ! ToString(j), E, true).
                o.define_property(j.into(), e, realm)?;
                // 4.e.ii. Set j to j + 1𝔽.
            }
        }

        // 5. Perform ? Set(O, "length", 𝔽(len + argCount), true).
        let new_len = len + arg_count;
        o.define_property("length".into(), new_len.into(), realm)?;

        // 6. Return 𝔽(len + argCount).
        Ok(new_len.into())
    }

    fn values(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .array_iter
                        .get(realm)?
                        .clone(),
                ),
            }),
            array: this,
            next: Cell::new(0),
            done: Cell::new(false),
            kind: ArrayIteratorKind::Values,
        };

        Ok(iter.into_value())
    }

    fn with(
        #[this] this: Value,
        index: Value,
        value: Value,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // 1. Let O be ? ToObject(this value).
        let o = coerce_object_strict(this, realm)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let len_raw = o
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)?;

        let len = if len_raw.is_nan() || len_raw <= 0.0 {
            0i64
        } else if len_raw >= 9007199254740991.0 {
            9007199254740991i64
        } else {
            len_raw.trunc() as i64
        };

        // 3. Let relativeIndex be ? ToIntegerOrInfinity(index).
        let relative_index = {
            let n = index.to_number(realm)?;
            if n.is_nan() || n == 0.0 {
                0i64
            } else if n == f64::INFINITY {
                i64::MAX
            } else if n == f64::NEG_INFINITY {
                i64::MIN
            } else {
                n.trunc() as i64
            }
        };

        // 4. If relativeIndex >= 0, let actualIndex be relativeIndex.
        // 5. Else, let actualIndex be len + relativeIndex.
        let actual_index = if relative_index >= 0 {
            relative_index
        } else {
            len + relative_index
        };

        // 6. If actualIndex >= len or actualIndex < 0, throw a RangeError exception.
        if actual_index >= len || actual_index < 0 {
            return Err(Error::range("Invalid index"));
        }

        // Check if we exceed array length limits
        if len as u64 > 4294967295 {
            return Err(Error::range("Invalid array length"));
        }

        // 7. Let A be ? ArrayCreate(len).
        let array = Self::with_len(realm, len as usize)?;

        // 8. Let k be 0.
        // 9. Repeat, while k < len,
        for k in 0..len as usize {
            // 9.a. Let Pk be ! ToString(𝔽(k)).
            // 9.b. If k = actualIndex, let fromValue be value.
            // 9.c. Else, let fromValue be ? Get(O, Pk).
            let from_value = if k == actual_index as usize {
                value.clone()
            } else {
                o.get(k, realm)?
            };

            // 9.d. Perform ! CreateDataPropertyOrThrow(A, Pk, fromValue).
            array.insert_array(from_value, k)?;

            // 9.e. Set k to k + 1.
        }

        // 10. Return A.
        Ok(array.into_value())
    }

    #[prop(crate::Symbol::ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator(&self, #[realm] realm: &mut Realm, #[this] this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .array_iter
                        .get(realm)?
                        .clone(),
                ),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
            kind: ArrayIteratorKind::Values,
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }

    #[prop(crate::Symbol::ASYNC_ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator_async(&self, #[realm] realm: &mut Realm, #[this] this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .array_iter
                        .get(realm)?
                        .clone(),
                ),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
            kind: ArrayIteratorKind::Values,
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }

    #[prop("toString")]
    fn to_string_js(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // 1. Let array be ? ToObject(this value).
        let array = coerce_object_strict(this.clone(), realm)?;

        // 2. Let func be ? Get(array, "join").
        let func = array.get("join", realm)?;

        // 3. If IsCallable(func) is false, set func to the intrinsic function %Object.prototype.toString%.
        // 4. Return ? Call(func, array).
        if func.is_callable() {
            let func_obj = func.as_object()?;
            func_obj.call(vec![], array.into(), realm)
        } else {
            // Fall back to %Object.prototype.toString%
            // Call the intrinsic directly to avoid issues with deleted/modified Object.prototype.toString
            crate::object::prototype::common::to_string(vec![], array.into(), realm)
        }
    }
}

impl PrettyObjectOverride for Array {
    fn pretty_inline(
        &self,
        _obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let Ok(inner) = self.inner.try_borrow() else {
            return None;
        };
        let mut s = String::new();
        s.push('[');
        for (i, (_, idx)) in inner.array.iter().enumerate() {
            if let Some(v) = inner.values.get(*idx) {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&v.value.pretty_print_circular(not, realm));
            }
        }
        s.push(']');
        Some(s)
    }

    fn pretty_multiline(
        &self,
        _obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let Ok(inner) = self.inner.try_borrow() else {
            return None;
        };
        let mut s = String::new();
        s.push_str("[\n");
        for (i, (_, idx)) in inner.array.iter().enumerate() {
            if let Some(v) = inner.values.get(*idx) {
                s.push_str("  ");
                s.push_str(&v.value.pretty_print_circular_nl(not, realm));
                if i + 1 < inner.array.len() {
                    s.push_str(",\n");
                }
            }
        }
        s.push_str("\n]");
        Some(s)
    }
}

#[object(constructor, function, name)]
#[derive(Debug)]
pub struct ArrayConstructor {}

impl CustomName for ArrayConstructor {
    fn custom_name(&self) -> String {
        "Function".to_owned()
    }
}

impl Constructor for ArrayConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        // 3. Let numberOfArgs be the number of elements in values.
        let number_of_args = args.len();

        // 4. If numberOfArgs = 0, return ArrayCreate(0, proto).
        if number_of_args == 0 {
            return Ok(Obj::into_object(Array::from_realm(realm)?));
        }

        // 5. Else if numberOfArgs = 1, then
        if number_of_args == 1 {
            let len = &args[0];

            // 5.c. If len is not a Number, then
            if !matches!(len, Value::Number(_)) {
                // 5.c.i. Perform ! CreateDataPropertyOrThrow(array, "0", len).
                // 5.c.ii. Let intLen be 1.
                let array = Array::with_elements(realm, args)?;
                return Ok(Obj::into_object(array));
            }

            // 5.d. Else (len is a Number)
            let len_num = len.as_number();

            // 5.d.i. Let intLen be ! ToUint32(len).
            let int_len = len_num as u32;

            // 5.d.ii. If SameValueZero(intLen, len) is false, throw a RangeError exception.
            // SameValueZero: intLen as f64 should equal len_num
            if (int_len as f64) != len_num {
                return Err(Error::range("Invalid array length"));
            }

            // 5.e. Perform ! Set(array, "length", intLen, true).
            return Ok(Obj::into_object(Array::with_len(realm, int_len as usize)?));
        }

        // 6. Else (numberOfArgs >= 2)
        let this = Array::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let mut inner = this.inner.try_borrow_mut()?;

        inner.set_array(args.into_iter());
        this.length.set(inner.array.len());

        drop(inner);

        Ok(Obj::into_object(this))
    }
}

impl Func for ArrayConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _: Value) -> ValueResult {
        Ok(Constructor::construct(self, realm, args)?.into())
    }
}

impl ArrayConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, proto: ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableArrayConstructor {
                object: MutObject::with_proto(proto.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl ArrayConstructor {
    #[prop("length")]
    const LENGTH: usize = 1;

    #[prop("isArray")]
    fn is_array(test: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let is_proto = if let Value::Object(o) = &test {
            o == realm.intrinsics.clone_public().array.get(realm)?
        } else {
            false
        };

        if is_proto {
            return Ok(true);
        }

        if let Some(proxy) = test.downcast::<Proxy>()? {
            if proxy.revoke.get() {
                return Err(Error::ty("Cannot perform 'isArray' on a revoked proxy"));
            }
        }

        let this: Res<OwningGcGuard<BoxedObj, Array>, _> =
            crate::value::FromValue::from_value(test);

        Ok(this.is_ok())
    }

    fn of(#[realm] realm: &mut Realm, args: Vec<Value>, #[this] this: Value) -> Res<ObjectHandle> {
        if let Ok(this) = this.as_object() {
            if this.is_constructable() {
                return this.construct(vec![Value::Number(args.len() as f64)], realm);
            }
        }

        let array = Array::with_elements(realm, args)?;

        Ok(Obj::into_object(array))
    }

    fn from(
        items: Value,
        mapper: Option<ObjectHandle>,
        this_arg: Option<Value>,
        #[realm] realm: &mut Realm,
        #[this] this: Value,
    ) -> Res<ObjectHandle> {
        if let Value::String(str) = &items {
            return Ok(Array::from_string_this(realm, str, this)?);
        }

        if let Value::Object(obj) = &items {
            if let Some(set) = obj.downcast::<Set>() {
                let inner = set.inner.try_borrow()?;

                let mut values = Vec::with_capacity(inner.set.len());

                let iter = inner.set.iter();

                for value in iter {
                    values.push(value.clone());
                }

                let array = if let Some(mapper) = mapper {
                    let mut res = Vec::with_capacity(values.len());

                    let this_arg = this_arg.unwrap_or(realm.global.clone().into());

                    for val in values {
                        let val = mapper.call(vec![val], this_arg.clone(), realm)?;

                        res.push(val);
                    }

                    res
                } else {
                    values
                };

                return Ok(Array::with_elements_this(realm, array, this)?);
            }

            if let Some(map) = obj.downcast::<Map>() {
                let inner = map.inner.try_borrow()?;

                let mut values = Vec::with_capacity(inner.map.len());

                let iter = inner.map.iter();

                for (key, value) in iter {
                    values.push(vec![key.clone(), value.clone()].try_into_value(realm)?);
                }

                let array = if let Some(mapper) = mapper {
                    let mut res = Vec::with_capacity(values.len());

                    let this_arg = this_arg.unwrap_or(realm.global.clone().into());

                    for val in values {
                        let val = mapper.call(vec![val], this_arg.clone(), realm)?;

                        res.push(val);
                    }

                    res
                } else {
                    values
                };

                return Ok(Array::with_elements_this(realm, array, this)?);
            }
        }

        let mut it = ArrayLike::new(items, realm)?;

        let array = if let Some(mapper) = mapper {
            let mut res = Vec::with_capacity(it.len());

            let this_arg = this_arg.unwrap_or(realm.global.clone().into());

            while let Some(val) = it.next(realm)? {
                let val = mapper.call(vec![val], this_arg.clone(), realm)?;

                res.push(val);
            }

            res
        } else {
            it.to_vec_no_close(realm)?
        };

        it.close(realm)?;

        Ok(Array::with_elements_this(realm, array, this)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayIteratorKind {
    Keys,
    Values,
    Entries,
}

#[object(name)]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ArrayIterator {
    pub array: ObjectHandle,
    pub next: Cell<usize>,
    pub done: Cell<bool>,
    pub kind: ArrayIteratorKind,
}

impl CustomName for ArrayIterator {
    fn custom_name(&self) -> String {
        "Array Iterator".to_owned()
    }
}

#[properties]
impl ArrayIterator {
    #[prop]
    pub fn next(&self, _args: Vec<Value>, realm: &mut Realm) -> ValueResult {
        if self.done.get() {
            let obj = Object::new(realm);
            obj.define_property("value".into(), Value::Undefined, realm)?;
            obj.define_property("done".into(), Value::Boolean(true), realm)?;
            return Ok(obj.into());
        }

        let index = self.next.get();
        let (done, value) = self.array.get_array_or_done(index, realm)?;

        self.next.set(index + 1);

        if done {
            self.done.set(true);
            let obj = Object::new(realm);
            obj.define_property("value".into(), Value::Undefined, realm)?;
            obj.define_property("done".into(), Value::Boolean(true), realm)?;
            return Ok(obj.into());
        }

        let value = value.map_or_else(
            || {
                self.done.set(true);
                Value::Undefined
            },
            |value| value,
        );

        // Create the result value based on the iterator kind
        let result_value = match self.kind {
            ArrayIteratorKind::Keys => Value::Number(index as f64),
            ArrayIteratorKind::Values => value,
            ArrayIteratorKind::Entries => {
                // Create [index, value] array
                let entry = Array::with_elements(realm, vec![Value::Number(index as f64), value])?;
                Obj::into_object(entry).into()
            }
        };

        let obj = Object::new(realm);
        obj.define_property("value".into(), result_value, realm)?;
        obj.define_property("done".into(), Value::Boolean(self.done.get()), realm)?;

        Ok(obj.into())
    }
}

impl Intrinsic for ArrayIterator {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Self::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone()),
            realm.intrinsics.func.clone(),
            realm,
        )
    }

    fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(realm
            .intrinsics
            .clone_public()
            .array_iter
            .get(realm)?
            .clone())
    }
}
