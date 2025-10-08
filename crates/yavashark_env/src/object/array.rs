use crate::console::print::{PrettyObjectOverride, PrettyPrint};
use crate::object::Object;
use crate::realm::Realm;
use crate::utils::{coerce_object_strict, ArrayLike, ProtoDefault, ValueIterator};
use crate::value::property_key::InternalPropertyKey;
use crate::value::{
    BoxedObj, Constructor, CustomName, DefinePropertyResult, Func, MutObj, Obj, ObjectImpl,
    ObjectOrNull, Property,
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

        let len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        self.length.set(len);

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

        self.get_wrapped_object()
            .define_property_attributes(name, value, realm)?;

        let len = self.get_inner().array.last().map_or(0, |(i, _)| *i + 1);
        self.length.set(len);

        Ok(DefinePropertyResult::Handled)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            return Ok(Some(Property::Value(Variable::write(
                self.length.get().into(),
            ))));
        }

        self.get_wrapped_object().resolve_property(name, realm)
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(&name, InternalPropertyKey::String(s) if s == "length") {
            return Ok(Some(Property::Value(self.length.get().into())));
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
    fn proto_default(realm: &Realm) -> Self {
        Self::new(realm.intrinsics.array.clone())
    }

    fn null_proto_default() -> Self {
        Self::new(ObjectOrNull::Null)
    }
}

impl Array {
    pub fn with_elements(realm: &Realm, elements: Vec<Value>) -> Res<Self> {
        let array = Self::new(realm.intrinsics.array.clone());

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements);

        drop(inner);

        Ok(array)
    }

    pub fn with_elements_and_proto(proto: ObjectHandle, elements: Vec<Value>) -> Res<Self> {
        let array = Self::new(proto);

        let mut inner = array.inner.try_borrow_mut()?;
        array.length.set(elements.len());

        inner.set_array(elements);

        drop(inner);

        Ok(array)
    }

    pub fn with_len(realm: &Realm, len: usize) -> Res<Self> {
        let array = Self::new(realm.intrinsics.array.clone());

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

        let array = Self::new(realm.intrinsics.array.clone());

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
                inner
                    .properties
                    .insert(InternalPropertyKey::Index(idx).into(), len);
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

    #[must_use]
    pub fn from_realm(realm: &Realm) -> Self {
        Self::new(realm.intrinsics.array.clone())
    }

    pub fn insert_array(&self, val: Value, idx: usize) -> Res {
        let mut inner = self.inner.try_borrow_mut()?;

        inner.insert_array(idx, val);
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

        let index = inner.array.last().map_or(0, |(i, _)| *i + 1);

        let len = inner.values.len();
        inner.values.push(Variable::new(value).into());

        inner.array.push((index, len));
        inner
            .properties
            .insert(InternalPropertyKey::Index(index).into(), len);
        self.length.set(index + 1);

        Ok(Value::Undefined)
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

    pub fn shallow_clone(&self, realm: &Realm) -> Res<Self> {
        let array = Self::new(realm.intrinsics.array.clone());

        let other_array = &self.inner.try_borrow()?;

        let mut inner = array.inner.try_borrow_mut()?;

        for (idx, value) in &other_array.array {
            let Some(value) = other_array.values.get(*value) else {
                continue;
            };

            let len = inner.values.len();
            inner.values.push(value.clone());

            inner.array.push((*idx, len));
            inner
                .properties
                .insert(InternalPropertyKey::Index(*idx).into(), len);
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

#[must_use]
pub fn convert_index(idx: isize, len: usize) -> usize {
    if idx < 0 {
        (len as isize + idx).max(0) as usize
    } else {
        idx as usize
    }
}
#[properties_new(default_null(array), constructor(ArrayConstructor::new))]
impl Array {
    #[prop("length")]
    pub const LENGTH: usize = 0;

    fn at(#[this] this: Value, idx: isize, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let length = this.get("length", realm)?.to_int_or_null(realm)? as usize;

        let idx = convert_index(idx, length);

        let (_, val) = this.get_array_or_done(idx, realm)?;

        Ok(val.map_or(Value::Undefined, |v| v))
    }

    fn concat(#[this] this: Value, #[realm] realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let array = if let Some(array) = this.downcast::<Self>()? {
            array.shallow_clone(realm)?
        } else {
            let items = ArrayLike::new(this, realm)?.to_vec(realm)?;

            Self::with_elements(realm, items)?
        };

        for arg in args {
            if ArrayLike::is_array_like(&arg, realm)? {
                let items = ArrayLike::new(arg, realm)?.to_vec(realm)?;

                for item in items {
                    array.push(item)?;
                }
            } else {
                array.push(arg)?;
            }
        }

        Ok(Obj::into_value(array))
    }

    #[prop("copyWithin")]
    fn copy_within(
        #[this] this_val: Value,
        target: isize,
        start: isize,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this_val, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        let target = convert_index(target, len);
        let start = convert_index(start, len);
        let end = end.map_or(len, |end| convert_index(end, len));

        for (count, idx) in (start..end).enumerate() {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                this.define_property((target + count).into(), val, realm)?;
            }
        }

        Ok(this.into())
    }

    fn entries(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array: this,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_value())
    }

    fn every(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        deez: Option<Value>,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let deez = deez.unwrap_or(realm.global.clone().into());

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val, idx.into(), this.clone().into()],
                    deez.clone(),
                    realm,
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
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let array = Self::from_realm(realm);

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    this_arg.clone(),
                    realm,
                )?;

                if x.is_truthy() {
                    array.push(val)?;
                }
            }
        }

        Ok(Obj::into_value(array))
    }

    fn find(#[this] this: Value, #[realm] realm: &mut Realm, func: &ObjectHandle) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                    realm,
                )?;

                if x.is_truthy() {
                    return Ok(val);
                }
            }
        }

        Ok(Value::Undefined)
    }

    #[prop("findIndex")]
    fn find_index(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                    realm,
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
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                    realm,
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
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                    realm,
                )?;

                if x.is_truthy() {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    fn flat(#[this] this: Value, #[realm] realm: &mut Realm, depth: Option<isize>) -> ValueResult {
        fn flatten(array: &Array, realm: &mut Realm, val: Value, depth: isize) -> Res {
            if depth == 0 {
                array.push(val)?;
                return Ok(());
            }

            if let Value::Object(obj) = &val {
                if obj.contains_key("length".into(), realm)? {
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
        let this = coerce_object_strict(this, realm)?;

        let array = Self::from_realm(realm);

        let depth = depth.unwrap_or(1);

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                flatten(&array, realm, val, depth)?;
            }
        }

        Ok(Obj::into_value(array))
    }

    #[prop("flatMap")]
    fn flat_map(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let array = Self::from_realm(realm);

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
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

        Ok(Obj::into_value(array))
    }

    #[prop("forEach")]
    fn for_each(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    this_arg.clone(),
                    realm,
                )?;
            }
        }

        Ok(Value::Undefined)
    }

    fn includes(
        #[this] this: Value,
        search_element: &Value,
        from_index: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let from_index = from_index.unwrap_or(0);

        let from_index = convert_index(from_index, len);

        for idx in from_index..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

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
        #[this] this: Value,
        search_element: &Value,
        from_index: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let from_index = from_index.unwrap_or(0);

        let from_index = convert_index(from_index, len);

        for idx in from_index..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                if val.eq(search_element) {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    fn join(#[this] this: Value, #[realm] realm: &mut Realm, separator: &Value) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let mut buf = String::new();

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        let sep = if separator.is_undefined() {
            YSString::new_static(",")
        } else {
            separator.to_string(realm)?
        };

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                buf.push_str(&val.to_string(realm)?);
            }

            if idx < len - 1 {
                buf.push_str(&sep);
            }
        }

        Ok(buf.into())
    }

    fn keys(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array: this,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_value())
    }

    #[prop("lastIndexOf")]
    fn last_index_of(
        #[this] this: Value,
        search_element: &Value,
        from_index: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let from_index = from_index.unwrap_or(len as isize - 1);

        let from_index = convert_index(from_index, len);

        for idx in (0..=from_index).rev() {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                if val.eq(search_element) {
                    return Ok(idx.into());
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    fn map(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        let len = len.to_number(realm)? as usize;

        let array = Self::from_realm(realm);

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(vec![val], this_arg.copy(), realm)?;

                array.insert_array(x, idx)?;
            }
        }

        Ok(Obj::into_value(array))
    }

    fn pop(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        if len == 0 {
            this.define_property("length".into(), 0.into(), realm)?;
            return Ok(Value::Undefined);
        }

        let idx = len - 1;

        let (_, val) = this.get_array_or_done(idx, realm)?;

        this.define_property("length".into(), idx.into(), realm)?;
        this.define_property(idx.into(), Value::Undefined, realm)?;

        Ok(val.unwrap_or(Value::Undefined))
    }

    #[prop("push")]
    fn push_js(
        #[this] this: Value,
        #[variadic] args: &[Value],
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let mut idx = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined)
            .to_number(realm)? as usize;

        for arg in args {
            this.define_property(idx.into(), arg.clone(), realm)?;
            idx += 1;
        }

        this.define_property("length".into(), idx.into(), realm)?;

        Ok(idx.into())
    }

    fn reduce(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        initial_value: &Value,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let mut acc = initial_value.clone();

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                acc = func.call(
                    vec![acc, val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                    realm,
                )?;
            }
        }

        Ok(acc)
    }

    #[prop("reduceRight")]
    fn reduce_right(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        initial_value: &Value,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let mut acc = initial_value.clone();

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                acc = func.call(
                    vec![acc, val.clone(), idx.into(), this.clone().into()],
                    realm.global.clone().into(),
                    realm,
                )?;
            }
        }

        Ok(acc)
    }

    fn reverse(#[this] this_val: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this_val, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let mut idx = 0;

        while idx < len / 2 {
            let (_, left_val) = this.get_array_or_done(idx, realm)?;
            let (_, right_val) = this.get_array_or_done(len - idx - 1, realm)?;

            if let Some(left_val) = left_val {
                this.define_property((len - idx - 1).into(), left_val, realm)?;
            }

            if let Some(right_val) = right_val {
                this.define_property(idx.into(), right_val, realm)?;
            }

            idx += 1;
        }

        Ok(this.into())
    }

    fn shift(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        if len == 0 {
            this.define_property("length".into(), 0.into(), realm)?;
            return Ok(Value::Undefined);
        }

        let (_, val) = this.get_array_or_done(0, realm)?;

        for idx in 1..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            this.define_property((idx - 1).into(), val.unwrap_or(Value::Undefined), realm)?;
        }

        this.define_property("length".into(), (len - 1).into(), realm)?;

        Ok(val.unwrap_or(Value::Undefined))
    }

    fn slice(
        #[this] this: Value,
        start: isize,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let start = convert_index(start, len);
        let end = end.map_or(len, |end| convert_index(end, len));

        let array = Self::from_realm(realm);

        for idx in start..end {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                array.push(val)?;
            }
        }

        Ok(Obj::into_value(array))
    }

    fn some(
        #[this] this: Value,
        #[realm] realm: &mut Realm,
        func: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                let x = func.call(
                    vec![val.clone(), idx.into(), this.clone().into()],
                    this_arg.clone(),
                    realm,
                )?;

                if x.is_truthy() {
                    return Ok(Value::Boolean(true));
                }
            }
        }

        Ok(Value::Boolean(false))
    }

    fn sort(
        #[this] this_val: Value,
        compare: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this_val, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let mut values = Vec::new();

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                values.push(val);
            }
        }

        if let Some(compare) = compare {
            values.sort_by(|a, b| {
                let x = compare
                    .call(
                        vec![a.clone(), b.clone()],
                        realm.global.clone().into(),
                        realm,
                    )
                    .unwrap_or(Value::Number(0.0));

                x.as_number().partial_cmp(&0.0).unwrap_or(Ordering::Equal)
            });
        } else {
            values.sort_by_key(|a| a.to_string(realm).unwrap_or_default());
        }

        for (idx, value) in values.into_iter().enumerate() {
            this.define_property(idx.into(), value, realm)?;
        }

        this.define_property("length".into(), len.into(), realm)?;

        Ok(this.into())
    }

    fn splice(
        #[this] this: Value,
        start: isize,
        delete_count: Option<isize>,
        items: Vec<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let start = convert_index(start, len);

        let delete_count = delete_count.unwrap_or(len as isize - start as isize);

        let delete_count = delete_count.max(0) as usize;

        let mut deleted = Vec::new();

        for idx in start..start + delete_count {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                deleted.push(val);
            }
        }

        let mut idx = start;

        let item_len = items.len();

        let shift = delete_count as isize + item_len as isize;

        while idx + delete_count < len {
            let rev_idx = len - (idx + delete_count);
            let (_, val) = this.get_array_or_done(rev_idx, realm)?;

            if let Some(val) = val {
                this.define_property(((rev_idx as isize + shift) as usize).into(), val, realm)?;
            } else {
                this.define_property(
                    ((rev_idx as isize + shift) as usize).into(),
                    Value::Undefined,
                    realm,
                )?;
            }

            idx += 1;
        }

        idx = start;

        for item in items {
            this.define_property(idx.into(), item, realm)?;
            idx += 1;
        }

        let new_len = len as isize + item_len as isize - delete_count as isize;

        this.define_property("length".into(), new_len.into(), realm)?;

        Ok(Obj::into_value(Self::with_elements(realm, deleted)?))
    }

    #[prop("toReversed")]
    fn js_to_reversed(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let array = Self::from_realm(realm);

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        for idx in (0..len).rev() {
            let (_, val) = this.get_array_or_done(idx, realm)?;

            if let Some(val) = val {
                array.push(val)?;
            }
        }

        Ok(Obj::into_value(array))
    }

    #[prop("toSorted")]
    fn js_to_sorted(
        #[this] this: Value,
        func: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let mut values = Vec::new();

        for idx in 0..len {
            let (_, val) = this.get_array_or_done(idx, realm)?;

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

        Ok(Obj::into_value(Self::with_elements(realm, values)?))
    }

    fn unshift(#[this] this: Value, args: Vec<Value>, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let len = this
            .resolve_property("length", realm)?
            .unwrap_or(Value::Undefined);

        let len = len.to_number(realm)? as usize;

        let mut idx = args.len() + len;

        while idx > 0 {
            let (_, val) = this.get_array_or_done(idx - args.len(), realm)?;

            this.define_property(idx.into(), val.unwrap_or(Value::Undefined), realm)?;

            idx -= 1;
        }

        for (idx, arg) in args.into_iter().enumerate() {
            this.define_property(idx.into(), arg, realm)?;
        }

        this.define_property("length".into(), idx.into(), realm)?;

        Ok(idx.into())
    }

    fn values(#[this] this: Value, #[realm] realm: &mut Realm) -> ValueResult {
        let this = coerce_object_strict(this, realm)?;

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
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
        let len = this.get_property("length", realm)?.to_number(realm)? as usize;

        let mut vals = Vec::with_capacity(len);

        let idx = convert_index(idx, len);

        for i in 0..len {
            if i == idx {
                vals.push(val.clone());
            } else {
                vals.push(this.get_property(i, realm)?.clone());
            }
        }

        Ok(Obj::into_value(Self::with_elements(realm, vals)?))
    }

    #[prop(crate::Symbol::ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator(&self, #[realm] realm: &Realm, #[this] this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }

    #[prop(crate::Symbol::ASYNC_ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator_async(&self, #[realm] realm: &Realm, #[this] this: Value) -> ValueResult {
        let Value::Object(obj) = this else {
            return Err(Error::ty_error(format!("Expected object, found {this:?}")));
        };

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array: obj,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }

    #[prop("toString")]
    fn to_string_js(&self, #[realm] realm: &mut Realm) -> Res<YSString> {
        let mut buf = String::new();

        let inner = self.inner.try_borrow()?;

        for (_, value) in &inner.array {
            let Some(value) = inner.values.get(*value) else {
                continue;
            };

            buf.push_str(value.value.to_string(realm)?.as_str());
            buf.push_str(", ");
        }

        buf.pop();
        buf.pop();

        Ok(buf.into())
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
        if args.len() == 1 {
            let len = args[0].to_number(realm)?;

            if !len.is_finite() || len.fract() != 0.0 || len < 0.0 {
                return Err(Error::range("Invalid array length"));
            }

            return Ok(Obj::into_object(Array::with_len(realm, len as usize)?));
        }

        let this = Array::new(realm.intrinsics.array.clone());

        let mut inner = this.inner.try_borrow_mut()?;

        inner.set_array(args);
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

        this.initialize(proto, realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl ArrayConstructor {
    #[prop("length")]
    const LENGTH: usize = 1;

    #[prop("isArray")]
    fn is_array(test: Value, #[realm] realm: &Realm) -> bool {
        let is_proto = test.as_object().is_ok_and(|o| *o == realm.intrinsics.array);

        if is_proto {
            return true;
        }

        let this: Res<OwningGcGuard<BoxedObj, Array>, _> =
            crate::value::FromValue::from_value(test);

        this.is_ok()
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
    ) -> Res<ObjectHandle> {
        if let Value::String(str) = &items {
            return Ok(Obj::into_object(Array::from_string(realm, str)?));
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

        Ok(Obj::into_object(Array::with_elements(realm, array)?))
    }
}

#[object(name)]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ArrayIterator {
    pub array: ObjectHandle,
    pub next: Cell<usize>,
    pub done: Cell<bool>,
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

        let (done, value) = self.array.get_array_or_done(self.next.get(), realm)?;

        self.next.set(self.next.get() + 1);

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

        let obj = Object::new(realm);
        obj.define_property("value".into(), value, realm)?;
        obj.define_property("done".into(), Value::Boolean(self.done.get()), realm)?;

        Ok(obj.into())
    }
}
