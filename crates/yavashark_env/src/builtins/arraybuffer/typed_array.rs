mod conv;

use crate::array::{convert_index, Array, ArrayIterator, MutableArrayIterator};
use crate::builtins::bigint64array::BigInt64Array;
use crate::builtins::biguint64array::BigUint64Array;
use crate::builtins::buf::ArrayBuffer;
use crate::builtins::float16array::Float16Array;
use crate::builtins::float32array::Float32Array;
use crate::builtins::float64array::Float64Array;
use crate::builtins::int16array::Int16Array;
use crate::builtins::int32array::Int32Array;
use crate::builtins::int8array::Int8Array;
use crate::builtins::uint16array::Uint16Array;
use crate::builtins::uint32array::Uint32Array;
use crate::builtins::unit8array::Uint8Array;
use crate::conversion::downcast_obj;
use crate::utils::ValueIterator;
use crate::value::property_key::InternalPropertyKey;
use crate::value::{MutObj, Obj};
use crate::{
    Error, GCd, MutObject, ObjectHandle, ObjectProperty, Realm, Res, Value, ValueResult, Variable,
};
use bytemuck::{try_cast_vec, AnyBitPattern, NoUninit, Zeroable};
use conv::to_value;
use half::f16;
use num_traits::{FromPrimitive, ToPrimitive};
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Range};
use yavashark_macro::{props, typed_array_run, typed_array_run_mut};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    F32,
    F64,
}

impl Type {
    #[must_use]
    pub const fn size(&self) -> usize {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 | Self::F16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 => 8,
        }
    }
}

#[repr(Rust, packed)]
struct Packed<T>(T);

impl<T> From<T> for Packed<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// impl<T> Into<T> for Packed<T> {
//     fn into(self) -> T {
//         self.0
//     }
// }

unsafe impl<T: Zeroable> Zeroable for Packed<T> {}

impl<T: Copy> Copy for Packed<T> {}

impl<T: Clone + Copy> Clone for Packed<T> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T: AnyBitPattern> AnyBitPattern for Packed<T> {}

unsafe impl<T: NoUninit> NoUninit for Packed<T> {}

#[derive(Debug)]
pub struct TypedArray {
    pub byte_offset: usize,
    pub opt_byte_length: usize,
    // TODO: this is a memleak!
    pub buffer: GCd<ArrayBuffer>,
    pub ty: Type,

    pub inner: RefCell<MutObject>,
}

impl crate::value::ObjectImpl for TypedArray {
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

    fn define_property(&self, name: Value, value: Value) -> Res {
        // if self.is_detached() {
        //     return self.get_wrapped_object().define_property(name, value)
        // }

        let key = InternalPropertyKey::from(name);

        if let InternalPropertyKey::Index(idx) = key {
            typed_array_run_mut!({
                let value: TY = FromPrimitive::from_f64(value.to_number_or_null())
                    .ok_or(Error::ty("Failed to convert to value"))?;

                if let Some(slot) = slice.get_mut(idx) {
                    slot.0 = value;
                } else {
                    return Err(Error::range("Index out of bounds"));
                }
            });

            Ok(())
        } else {
            self.get_wrapped_object().define_property(key.into(), value)
        }
    }

    fn define_variable(&self, name: Value, value: Variable) -> Res {
        if self.is_detached() {
            return self.get_wrapped_object().define_variable(name, value);
        }

        let key = InternalPropertyKey::from(name);

        if let InternalPropertyKey::Index(idx) = key {
            typed_array_run_mut!({
                let value: TY = FromPrimitive::from_f64(value.value.to_number_or_null())
                    .ok_or(Error::ty("Failed to convert to value"))?;

                if let Some(slot) = slice.get_mut(idx) {
                    slot.0 = value;
                } else {
                    return Err(Error::range("Index out of bounds"));
                }
            });

            Ok(())
        } else {
            self.get_wrapped_object().define_variable(key.into(), value)
        }
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        let key = InternalPropertyKey::from(name.copy());

        if let InternalPropertyKey::Index(idx) = key {
            if self.is_detached() {
                return self.get_wrapped_object().get_property(name);
            }

            typed_array_run!({
                return Ok(slice.get(idx).map(|x| to_value(x.0).into()));
            });
        }

        self.get_wrapped_object().resolve_property(&key.into())
    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if self.is_detached() {
            return self.get_wrapped_object().get_property(name);
        }

        let key = InternalPropertyKey::from(name.copy());

        if let InternalPropertyKey::Index(idx) = key {
            typed_array_run!({
                return Ok(slice.get(idx).map(|x| to_value(x.0).into()));
            });
        }

        self.get_wrapped_object().get_property(&key.into())
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        if self.is_detached() {
            return self.get_wrapped_object().define_getter(name, value);
        }

        let key = InternalPropertyKey::from(name);
        if matches!(key, InternalPropertyKey::Index(_)) {
            return Ok(());
        }

        self.get_wrapped_object().define_getter(key.into(), value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        if self.is_detached() {
            return self.get_wrapped_object().define_setter(name, value);
        }

        let key = InternalPropertyKey::from(name);
        if matches!(key, InternalPropertyKey::Index(_)) {
            return Ok(());
        }

        self.get_wrapped_object().define_setter(key.into(), value)
    }

    fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
        if self.is_detached() {
            return self.get_wrapped_object().delete_property(name);
        }

        let key = InternalPropertyKey::from(name.copy());
        if matches!(key, InternalPropertyKey::Index(_)) {
            return Ok(None);
        }

        self.get_wrapped_object().delete_property(&key.into())
    }

    fn contains_key(&self, name: &Value) -> Res<bool> {
        if self.is_detached() {
            return self.get_wrapped_object().contains_key(name);
        }

        let key = InternalPropertyKey::from(name.copy());

        if let InternalPropertyKey::Index(idx) = key {
            typed_array_run!({
                return Ok(slice.get(idx).is_some());
            });
        }

        self.get_wrapped_object().contains_key(&key.into())
    }

    fn has_key(&self, name: &Value) -> Res<bool> {
        let key = InternalPropertyKey::from(name.copy());

        if let InternalPropertyKey::Index(idx) = key {
            if self.is_detached() {
                return self.get_wrapped_object().contains_key(name);
            }

            typed_array_run!({
                return Ok(slice.get(idx).is_some());
            });
        }

        self.get_wrapped_object().has_key(&key.into())
    }

    fn properties(&self) -> Res<Vec<(Value, Value)>> {
        if self.is_detached() {
            return self.get_wrapped_object().properties();
        }

        let mut props = typed_array_run!({
            slice
                .iter()
                .enumerate()
                .map(|(i, x)| (i.into(), to_value(x.0)))
                .collect::<Vec<_>>()
        });

        props.append(&mut self.get_wrapped_object().properties()?);

        Ok(props)
    }

    fn keys(&self) -> Res<Vec<Value>> {
        if self.is_detached() {
            return self.get_wrapped_object().keys();
        }

        let mut keys = typed_array_run!({
            slice
                .iter()
                .enumerate()
                .map(|(i, _)| i.into())
                .collect::<Vec<_>>()
        });

        keys.append(&mut self.get_wrapped_object().keys()?);

        Ok(keys)
    }

    fn values(&self) -> Res<Vec<Value>> {
        if self.is_detached() {
            return self.get_wrapped_object().values();
        }

        let mut values =
            typed_array_run!({ slice.iter().map(|x| to_value(x.0)).collect::<Vec<_>>() });

        values.append(&mut self.get_wrapped_object().values()?);

        Ok(values)
    }

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
        if self.is_detached() {
            return self.get_wrapped_object().get_array_or_done(index);
        }

        typed_array_run!({ Ok((index < slice.len(), slice.get(index).map(|x| to_value(x.0)))) })
    }
}

impl TypedArray {
    pub fn new(
        realm: &mut Realm,
        mut buffer: Value,
        byte_offset: Option<usize>,
        length: Option<usize>,
        ty: Type,
    ) -> Res<Self> {
        let buf = if let Ok(buf) = downcast_obj::<ArrayBuffer>(buffer.copy()) {
            buf
        } else if buffer.has_key(&"length".into()).ok().unwrap_or(false) {
            let iter = ValueIterator::new(&buffer, realm)?;

            let mut items = Vec::new();

            while let Some(item) = iter.next(realm)? {
                items.push(item);
            }

            buffer = convert_buffer(items, ty, realm)?.into_value();

            downcast_obj::<ArrayBuffer>(buffer.copy())?
        } else {
            let len = buffer.to_int_or_null(realm)? as usize * ty.size();
            buffer = ArrayBuffer::new(realm, len)?.into_value();

            downcast_obj::<ArrayBuffer>(buffer.copy())?
        };

        let byte_offset = byte_offset.unwrap_or(0);

        // if byte_offset > buf_len { //TODO: re-implement this with BYTES_PER_ELEMENT
        //     return Err(Error::range("byteOffset is out of bounds"));
        // }
        //
        let byte_length = length.map_or_else(
            || usize::MAX,
            |len| {
                // if len + byte_offset > buf_len {
                //     return Err(Error::range("byteLength is out of bounds"));
                // } //TODO
                len * ty.size()
            },
        );

        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(realm.intrinsics.typed_array.clone())),
            buffer: buf,
            byte_offset,
            opt_byte_length: byte_length,
            ty,
        })
    }

    pub fn from_buffer(realm: &Realm, buffer: ArrayBuffer, ty: Type) -> Res<Self> {
        let buffer = buffer.into_value();
        let buffer = downcast_obj::<ArrayBuffer>(buffer)?;

        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(realm.intrinsics.typed_array.clone())),
            buffer,
            byte_offset: 0,
            opt_byte_length: usize::MAX,
            ty,
        })
    }

    pub fn apply_offsets<'a>(&self, slice: &'a [u8]) -> Res<&'a [u8]> {
        let start = self.byte_offset;

        if start > slice.len() {
            return Err(Error::ty("TypedArray detached"));
        }

        let mut end = if self.opt_byte_length == usize::MAX {
            start + self.opt_byte_length.min(slice.len() - start)
        } else if self.opt_byte_length > slice.len() - start {
            return Err(Error::ty("TypedArray detached"));
        } else {
            start + self.opt_byte_length
        };

        end -= end % self.ty.size();

        Ok(slice.get(start..end).unwrap_or_default())
    }

    pub fn apply_offsets_mut<'a>(&self, slice: &'a mut [u8]) -> Res<&'a mut [u8]> {
        let start = self.byte_offset;

        if start > slice.len() {
            return Err(Error::ty("TypedArray detached"));
        }

        let mut end = if self.opt_byte_length == usize::MAX {
            start + self.opt_byte_length.min(slice.len() - start)
        } else if self.opt_byte_length > slice.len() - start {
            return Err(Error::ty("TypedArray detached"));
        } else {
            start + self.opt_byte_length
        };

        end -= end % self.ty.size();

        Ok(slice.get_mut(start..end).unwrap_or_default())
    }

    pub fn to_value_vec(&self) -> Res<Vec<Value>> {
        Ok(typed_array_run!({
            slice.iter().map(|x| to_value(x.0)).collect()
        }))
    }

    pub fn is_attached(&self) -> bool {
        self.buffer.inner.borrow().buffer.is_some()
    }

    pub fn is_detached(&self) -> bool {
        let buffer = self.buffer.inner.borrow();

        let Some(buffer) = &buffer.buffer else {
            return true;
        };

        let buf_len = buffer.len();

        if self.byte_offset > buf_len {
            return true;
        }

        if self.opt_byte_length == usize::MAX {
            return false;
        }

        self.opt_byte_length > buf_len - self.byte_offset
    }
}

fn convert_buffer(items: Vec<Value>, ty: Type, realm: &mut Realm) -> Res<ArrayBuffer> {
    let len = items.len()
        * match ty {
            Type::U8 | Type::I8 => 1,
            Type::U16 | Type::I16 | Type::F16 => 2,
            Type::U32 | Type::I32 | Type::F32 => 4,
            Type::U64 | Type::I64 | Type::F64 => 8,
        };

    let mut buffer = Vec::with_capacity(len);

    for item in items {
        match ty {
            Type::U8 => {
                buffer.push(item.to_number(realm)? as u8);
            }
            Type::U16 => {
                buffer.extend_from_slice(&(item.to_number(realm)? as u16).to_le_bytes());
            }
            Type::U32 => {
                buffer.extend_from_slice(&(item.to_number(realm)? as u32).to_le_bytes());
            }
            Type::U64 => {
                buffer.extend_from_slice(
                    &(item.to_big_int(realm)?.to_u64().unwrap_or_default()).to_le_bytes(),
                );
            }
            Type::I8 => {
                buffer.extend_from_slice(&(item.to_number(realm)? as i8).to_le_bytes());
            }
            Type::I16 => {
                buffer.extend_from_slice(&(item.to_number(realm)? as i16).to_le_bytes());
            }
            Type::I32 => {
                buffer.extend_from_slice(&(item.to_number(realm)? as i32).to_le_bytes());
            }
            Type::I64 => {
                buffer.extend_from_slice(
                    &(item.to_big_int(realm)?.to_i64().unwrap_or_default()).to_le_bytes(),
                );
            }
            Type::F16 => {
                buffer.extend_from_slice(&(f16::from_f64(item.to_number(realm)?)).to_le_bytes());
            }
            Type::F32 => {
                buffer.extend_from_slice(&(item.to_number(realm)? as f32).to_le_bytes());
            }
            Type::F64 => {
                buffer.extend_from_slice(&(item.to_number(realm)?).to_le_bytes());
            }
        }
    }

    Ok(ArrayBuffer::from_buffer(realm, buffer))
}

#[props]
impl TypedArray {
    const BYTES_PER_ELEMENT: u8 = 1;

    #[call_constructor]
    pub fn construct() -> Res {
        Err(Error::ty("Abstract class TypedArray not directly constructable") )
    }


    #[get("buffer")]
    pub fn get_buffer(&self) -> ObjectHandle {
        self.buffer.gc().into()
    }

    #[get("byteLength")]
    pub fn get_byte_length(&self) -> usize {
        let buf_len = self.buffer.get_slice().map_or(0, |s| s.len());
        let len = buf_len.saturating_sub(self.byte_offset);

        if self.opt_byte_length == usize::MAX {
            len - (len % self.ty.size())
        } else if self.opt_byte_length > len {
            0
        } else {
            self.opt_byte_length - (self.opt_byte_length % self.ty.size())
        }
    }

    #[get("byteOffset")]
    pub fn get_byte_offset(&self) -> usize {
        let buf_len = self.buffer.get_slice().map_or(0, |s| s.len());

        if self.byte_offset > buf_len {
            0
        } else if self.opt_byte_length != usize::MAX {
            let len = buf_len.saturating_sub(self.byte_offset);

            if self.opt_byte_length != usize::MAX && self.opt_byte_length > len {
                0
            } else {
                self.byte_offset
            }
        } else {
            self.byte_offset
        }
    }

    #[get("length")]
    pub fn get_length(&self) -> usize {
        let buf_len = self.buffer.get_slice().map_or(0, |s| s.len());
        let len = buf_len.saturating_sub(self.byte_offset);

        if self.opt_byte_length == usize::MAX {
            len / self.ty.size()
        } else if self.opt_byte_length > len {
            0
        } else {
            self.opt_byte_length / self.ty.size()
        }
    }

    pub fn at(&self, idx: isize) -> Res<Value> {
        Ok(typed_array_run!({
            let idx = convert_index(idx, slice.len());

            slice.get(idx).map_or(Value::Undefined, |x| to_value(x.0))
        }))
    }

    #[prop("copyWithin")]
    pub fn copy_within(
        &self,
        target: isize,
        start: isize,
        end: Option<isize>,
        this: Value,
    ) -> ValueResult {
        fn oob(
            target: isize,
            start: isize,
            end: Option<isize>,
            len: usize,
        ) -> Option<(Range<usize>, usize)> {
            let target = convert_index(target, len);
            let start = convert_index(start, len);
            let end = end.map(|end| convert_index(end, len));

            if target >= len {
                return None;
            }

            if start >= len {
                return None;
            }

            let end = end.unwrap_or(usize::MAX).min(len - target);

            Some((start..end, target))
        }

        typed_array_run_mut!({
            let Some((range, target)) = oob(target, start, end, slice.len()) else {
                return Ok(this);
            };

            slice.copy_within(range, target);
        });

        Ok(this)
    }

    pub fn entries(&self, #[realm] realm: &Realm) -> ValueResult {
        let array = Array::with_elements(realm, self.to_value_vec()?)?.into_object();

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_value())
    }

    pub fn every(
        &self,
        #[this] array: &Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
    ) -> Res<bool> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, Value::Undefined)?;

                if !res.is_truthy() {
                    return Ok(false);
                }
            }
        });

        Ok(true)
    }

    pub fn fill(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        value: &Value,
        start: Option<isize>,
        end: Option<isize>,
    ) -> ValueResult {
        let num = value.to_numeric(realm)?;
        let num = num.to_f64().unwrap_or_default();

        typed_array_run_mut!({
            let len = slice.len();

            let start = start.map_or(0, |start| convert_index(start, len));
            let end = end.map_or(len, |end| convert_index(end, len));

            let value: TY =
                FromPrimitive::from_f64(num).ok_or(Error::ty("Failed to convert to value"))?;

            for val in slice
                .get_mut(start..end)
                .ok_or(Error::range("TypedArray is out of bounds"))?
            {
                val.0 = value;
            }
        });

        Ok(array)
    }

    pub fn filter(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<ObjectHandle> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let mut results: Vec<u8> = Vec::new();

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                if res.is_truthy() {
                    results.extend_from_slice(x.0.to_le_bytes().as_slice());
                }
            }
        });

        create_ta(realm, self.ty, results)
    }

    pub fn find(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<Value> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                if res.is_truthy() {
                    return Ok(to_value(x.0));
                }
            }
        });

        Ok(Value::Undefined)
    }

    #[prop("findIndex")]
    pub fn find_index(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<isize> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                if res.is_truthy() {
                    return Ok(idx as isize);
                }
            }
        });

        Ok(-1)
    }

    #[prop("findLast")]
    pub fn find_last(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<Value> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate().rev() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                if res.is_truthy() {
                    return Ok(to_value(x.0));
                }
            }
        });

        Ok(Value::Undefined)
    }

    #[prop("findLastIndex")]
    pub fn find_last_index(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<isize> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate().rev() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                if res.is_truthy() {
                    return Ok(idx as isize);
                }
            }
        });

        Ok(-1)
    }

    #[prop("forEach")]
    pub fn for_each(
        &self,
        #[this] array: &Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<()> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                callback.call(realm, args, this_arg.copy())?;
            }
        });

        Ok(())
    }

    pub fn includes(
        &self,
        search_element: &Value,
        from_index: Option<isize>,
        realm: &mut Realm,
    ) -> Res<bool> {
        let search_element = search_element.to_numeric(realm)?;
        let search_element = search_element.to_f64().unwrap_or_default();

        typed_array_run!({
            let len = slice.len();
            let from_index = from_index.map_or(0, |i| convert_index(i, len));

            let num: TY = FromPrimitive::from_f64(search_element)
                .ok_or(Error::ty("Failed to convert to value"))?;

            for x in slice.get(from_index..).unwrap_or_default() {
                let n = x.0;

                if n == num {
                    return Ok(true);
                }
            }
        });

        Ok(false)
    }

    #[prop("indexOf")]
    pub fn index_of(
        &self,
        search_element: &Value,
        from_index: Option<isize>,
        realm: &mut Realm,
    ) -> Res<isize> {
        let search_element = search_element.to_numeric(realm)?;
        let search_element = search_element.to_f64().unwrap_or_default();

        typed_array_run!({
            let len = slice.len();
            let from_index = from_index.map_or(0, |i| convert_index(i, len));

            let num: TY = FromPrimitive::from_f64(search_element)
                .ok_or(Error::ty("Failed to convert to value"))?;

            for (idx, x) in slice
                .get(from_index..)
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                let n = x.0;
                if n == num {
                    return Ok((idx + from_index) as isize);
                }
            }
        });

        Ok(-1)
    }

    pub fn join(&self, separator: Option<String>) -> Res<String> {
        let sep = separator.as_deref().unwrap_or(",");

        let mut str = String::new();

        typed_array_run!({
            let mut first = true;
            for x in slice.iter() {
                if !str.is_empty() && !first {
                    str.push_str(sep);
                }
                first = false;

                let n = x.0;
                str.push_str(&n.to_string());
            }
        });

        Ok(str)
    }

    #[prop("keys")]
    pub fn keys_js(&self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let array = Array::with_elements(realm, (0..self.get_length()).map(Into::into).collect())?
            .into_object();

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_object())
    }

    #[prop("lastIndexOf")]
    pub fn last_index_of(
        &self,
        search_element: Value,
        from_index: Option<isize>,
        realm: &mut Realm,
    ) -> Res<isize> {
        let search_element = search_element.to_numeric(realm)?;
        let search_element = search_element.to_f64().unwrap_or_default();

        typed_array_run!({
            let len = slice.len();
            let from_index = from_index.map_or(0, |i| convert_index(i, len));

            let num: TY = FromPrimitive::from_f64(search_element)
                .ok_or(Error::ty("Failed to convert to value"))?;

            for (idx, x) in slice
                .get(0..=from_index as usize)
                .unwrap_or_default()
                .iter()
                .enumerate()
                .rev()
            {
                let n = x.0;
                if n == num {
                    return Ok(idx as isize);
                }
            }
        });

        Ok(-1)
    }

    #[prop("map")]
    pub fn map_js(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<ObjectHandle> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let mut results: Vec<u8> = Vec::new();

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                let num: TY = FromPrimitive::from_f64(res.to_number_or_null())
                    .ok_or(Error::ty("Failed to convert to value"))?;

                results.extend_from_slice(num.to_le_bytes().as_slice());
            }
        });

        create_ta(realm, self.ty, results)
    }

    pub fn reduce(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        initial_value: Option<Value>,
    ) -> Res<Value> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let mut acc;

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);

            let iter = owned.into_iter().enumerate();

            if let Some(initial) = initial_value {
                acc = initial;
            } else {
                let Some((_, first_val)) = iter.clone().next() else {
                    return Ok(Value::Undefined);
                };

                acc = to_value(first_val.0);
            }

            for (idx, x) in iter {
                let args = vec![acc, to_value(x.0), idx.into(), array.copy()];

                acc = callback.call(realm, args, Value::Undefined)?;
            }
        });

        Ok(acc)
    }

    #[prop("reduceRight")]
    pub fn reduce_right(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        initial_value: Option<Value>,
    ) -> Res<Value> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let mut acc;

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);

            let iter = owned.into_iter().enumerate().rev();

            if let Some(initial) = initial_value {
                acc = initial;
            } else {
                let Some((_, first_val)) = iter.clone().next() else {
                    return Ok(Value::Undefined);
                };

                acc = to_value(first_val.0);
            }

            for (idx, x) in iter {
                let args = vec![acc, to_value(x.0), idx.into(), array.copy()];

                acc = callback.call(realm, args, Value::Undefined)?;
            }
        });

        Ok(acc)
    }

    pub fn reverse(&self, #[this] array: Value) -> Res<Value> {
        typed_array_run_mut!({
            slice.reverse();
        });

        Ok(array)
    }

    pub fn set(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        source: &Value,
        offset: Option<isize>,
    ) -> Res<Value> {
        let offset = offset.map_or(0, |i| convert_index(i, self.get_length()));

        let bytes = if let Ok(ta) = downcast_obj::<TypedArray>(source.copy()) {
            ta.buffer.get_slice()?.to_vec()
        } else if source.has_key(&"length".into()).ok().unwrap_or(false) {
            let iter = ValueIterator::new(source, realm)?;

            let mut bytes = Vec::new();

            while let Some(item) = iter.next(realm)? {
                extend_as_bytes(&mut bytes, item, self.ty)?;
            }

            bytes
        } else {
            return Err(Error::ty("Source is not a TypedArray or array-like object"));
        };

        let offset = offset * self.ty.size();

        let mut slice = self.buffer.get_slice_mut()?;

        let slice = self.apply_offsets_mut(&mut slice)?;

        if offset > slice.len() {
            return Err(Error::range("Offset is out of bounds"));
        }

        let len = bytes.len().min(slice.len() - offset);

        slice[offset..offset + len].copy_from_slice(&bytes[..len]);

        Ok(array)
    }

    pub fn slice(
        &self,
        #[realm] realm: &mut Realm,
        start: Option<isize>,
        end: Option<isize>,
    ) -> Res<ObjectHandle> {
        typed_array_run!({
            let len = slice.len();

            let start = start.map_or(0, |start| convert_index(start, len));
            let end = end.map_or(len, |end| convert_index(end, len));

            let mut bytes = Vec::new();

            for val in slice
                .get(start..end)
                .ok_or(Error::range("TypedArray is out of bounds"))?
            {
                bytes.extend_from_slice(val.0.to_le_bytes().as_slice());
            }

            create_ta(realm, self.ty, bytes)
        })
    }

    pub fn some(
        &self,
        #[this] array: &Value,
        #[realm] realm: &mut Realm,
        callback: &ObjectHandle,
        this_arg: Option<Value>,
    ) -> Res<bool> {
        if !callback.is_function() {
            return Err(Error::ty("Callback is not a function"));
        }

        let this_arg = this_arg.unwrap_or(realm.global.clone().into());

        typed_array_run!({
            let owned = slice.to_vec();
            drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![to_value(x.0), idx.into(), array.copy()];

                let res = callback.call(realm, args, this_arg.copy())?;

                if res.is_truthy() {
                    return Ok(true);
                }
            }
        });

        Ok(false)
    }

    #[length(1)]
    pub fn sort(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        compare_fn: Option<ObjectHandle>,
    ) -> Res<Value> {
        if let Some(compare_fn) = &compare_fn {
            if !compare_fn.is_function() {
                return Err(Error::ty("Compare function is not a function"));
            }
        }

        typed_array_run_mut!({
            if let Some(compare_fn) = compare_fn {
                let owned = slice.to_vec();
                drop(slice0);

                let mut vec = owned;

                vec.sort_by(|a, b| {
                    let args = vec![to_value(a.0), to_value(b.0)];

                    let res = compare_fn.call(realm, args, Value::Undefined);

                    match res {
                        Ok(v) => {
                            let n = v.to_number_or_null();

                            if n.is_nan() || n == 0.0 {
                                std::cmp::Ordering::Equal
                            } else if n < 0.0 {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Greater
                            }
                        }
                        Err(_) => std::cmp::Ordering::Equal,
                    }
                });

                let mut slice0 = self.buffer.get_slice_mut()?;

                let slice = self.apply_offsets_mut(&mut slice0)?;

                let slice =
                    bytemuck::try_cast_slice_mut::<u8, Packed<TY>>(slice).map_err(bytemuck_err)?;

                slice.copy_from_slice(&vec);
            } else {
                slice.sort_by(|a, b| {
                    let a = a.0;
                    let b = b.0;
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        });

        Ok(array)
    }

    pub fn subarray(
        &self,
        start: isize,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let len = self.get_length();

        let start = convert_index(start, len);
        let end = end.map_or(len, |end| convert_index(end, len));

        let byte_offset = self.byte_offset + start * self.ty.size();
        let length = end.saturating_sub(start);

        TypedArray::new(
            realm,
            self.buffer.gc().into(),
            Some(byte_offset),
            Some(length),
            self.ty,
        )
        .map(|ta| ta.into_object())
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self) -> Res<String> {
        let mut str = String::new();

        typed_array_run!({
            let mut first = true;
            for x in slice.iter() {
                if !str.is_empty() && !first {
                    str.push(',');
                }
                first = false;

                let n = x.0;
                str.push_str(&n.to_string());
            }
        });

        Ok(str)
    }

    #[prop("toReversed")]
    pub fn to_reversed(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let results;

        typed_array_run!({
            let mut owned = slice.to_vec();
            owned.reverse();

            results = try_cast_vec::<Packed<TY>, u8>(owned).map_err(|(e, _)| bytemuck_err(e))?;
        });

        create_ta(realm, self.ty, results)
    }

    #[prop("toSorted")]
    #[length(1)]
    pub fn to_sorted(
        &self,
        #[realm] realm: &mut Realm,
        compare_fn: Option<ObjectHandle>,
    ) -> Res<ObjectHandle> {
        if let Some(compare_fn) = &compare_fn {
            if !compare_fn.is_function() {
                return Err(Error::ty("Compare function is not a function"));
            }
        }

        let results;

        typed_array_run!({
            let mut owned = slice.to_vec();

            if let Some(compare_fn) = compare_fn {
                owned.sort_by(|a, b| {
                    let args = vec![to_value(a.0), to_value(b.0)];

                    let res = compare_fn.call(realm, args, Value::Undefined);

                    match res {
                        Ok(v) => {
                            let n = v.to_number_or_null();

                            if n.is_nan() || n == 0.0 {
                                std::cmp::Ordering::Equal
                            } else if n < 0.0 {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Greater
                            }
                        }
                        Err(_) => std::cmp::Ordering::Equal,
                    }
                });
            } else {
                owned.sort_by(|a, b| {
                    let a = a.0;
                    let b = b.0;
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }

            results = try_cast_vec::<Packed<TY>, u8>(owned).map_err(|(e, _)| bytemuck_err(e))?;
        });

        create_ta(realm, self.ty, results)
    }

    #[prop("toString")]
    pub fn to_string_js(&self) -> Res<String> {
        let mut str = String::new();

        typed_array_run!({
            let mut first = true;
            for x in slice.iter() {
                if !str.is_empty() && !first {
                    str.push(',');
                }
                first = false;

                let n = x.0;
                str.push_str(&n.to_string());
            }
        });

        Ok(str)
    }

    #[prop("values")]
    pub fn values_js(&self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let array = Array::with_elements(realm, self.to_value_vec()?)?.into_object();

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_object())
    }

    pub fn with(&self, #[realm] realm: &mut Realm, index: isize, value: f64) -> Res<ObjectHandle> {
        let mut bytes = Vec::new();

        typed_array_run!({
            let len = slice.len();
            let index = convert_index(index, len);
            let value = Value::from(value);

            for (idx, x) in slice.iter().enumerate() {
                if idx == index {
                    extend_as_bytes(&mut bytes, value.copy(), self.ty)?;
                } else {
                    extend_as_bytes(&mut bytes, to_value(x.0), self.ty)?;
                }
            }
        });

        create_ta(realm, self.ty, bytes)
    }

    #[prop(crate::Symbol::ITERATOR)]
    #[allow(clippy::unused_self)]
    fn iterator(&self, #[realm] realm: &Realm) -> ValueResult {
        let array = Array::with_elements(realm, self.to_value_vec()?)?.into_object();

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone()),
            }),
            array,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        let iter: Box<dyn Obj> = Box::new(iter);

        Ok(iter.into())
    }
}

fn create_ta(realm: &mut Realm, ty: Type, bytes: Vec<u8>) -> Res<ObjectHandle> {
    let buffer = ArrayBuffer::from_buffer(realm, bytes);

    let ta = TypedArray::from_buffer(realm, buffer, ty)?;

    Ok(match ty {
        Type::U8 => Uint8Array::new(realm, ta)?.into_object(),
        Type::U16 => Uint16Array::new(realm, ta)?.into_object(),
        Type::U32 => Uint32Array::new(realm, ta)?.into_object(),
        Type::U64 => BigUint64Array::new(realm, ta)?.into_object(),
        Type::I8 => Int8Array::new(realm, ta)?.into_object(),
        Type::I16 => Int16Array::new(realm, ta)?.into_object(),
        Type::I32 => Int32Array::new(realm, ta)?.into_object(),
        Type::I64 => BigInt64Array::new(realm, ta)?.into_object(),
        Type::F16 => Float16Array::new(realm, ta)?.into_object(),
        Type::F32 => Float32Array::new(realm, ta)?.into_object(),
        Type::F64 => Float64Array::new(realm, ta)?.into_object(),
    })
}

fn extend_as_bytes(bytes: &mut Vec<u8>, value: Value, ty: Type) -> Res<()> {
    let value = value.to_number_or_null();

    match ty {
        Type::U8 => {
            bytes.push(value as u8);
        }
        Type::U16 => {
            bytes.extend_from_slice(&(value as u16).to_le_bytes());
        }
        Type::U32 => {
            bytes.extend_from_slice(&(value as u32).to_le_bytes());
        }
        Type::U64 => {
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        Type::I8 => {
            bytes.extend_from_slice(&(value as i8).to_le_bytes());
        }
        Type::I16 => {
            bytes.extend_from_slice(&(value as i16).to_le_bytes());
        }
        Type::I32 => {
            bytes.extend_from_slice(&(value as i32).to_le_bytes());
        }
        Type::I64 => {
            bytes.extend_from_slice(&(value as i64).to_le_bytes());
        }
        Type::F16 => {
            bytes.extend_from_slice(&(f16::from_f64(value)).to_le_bytes());
        }
        Type::F32 => {
            bytes.extend_from_slice(&(value as f32).to_le_bytes());
        }
        Type::F64 => {
            bytes.extend_from_slice(&(value).to_le_bytes());
        }
    }

    Ok(())
}

fn bytemuck_err(err: bytemuck::PodCastError) -> Error {
    Error::new_error(err.to_string())
}
