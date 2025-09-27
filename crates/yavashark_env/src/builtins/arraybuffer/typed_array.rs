use crate::array::{convert_index, Array, ArrayIterator, MutableArrayIterator};
use crate::builtins::ArrayBuffer;
use crate::conversion::downcast_obj;
use crate::utils::ValueIterator;
use crate::{Error, GCd, MutObject, ObjectHandle, ObjectProperty, Realm, Res, Value, ValueResult, Variable};
use bytemuck::{AnyBitPattern, NoUninit, Zeroable};
use half::f16;
use num_traits::FromPrimitive;
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Range};
use yavashark_macro::{props, typed_array_run, typed_array_run_mut};
use yavashark_value::{MutObj, Obj};
use yavashark_value::property_key::InternalPropertyKey;

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

    pub inner: RefCell<MutObject>
}

impl yavashark_value::ObjectImpl<Realm> for TypedArray {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl DerefMut<Target=impl MutObj<Realm>> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target=Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target=Self::Inner> {
        self.inner.borrow_mut()
    }

    fn define_property(&self, name: Value, value: Value) -> Res {
        if self.is_detached() {
            return self.get_wrapped_object().define_property(name, value)
        }

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
        if self.is_detached() {
            return self.get_wrapped_object().resolve_property(name);
        }


        let key = InternalPropertyKey::from(name.copy());

        if let InternalPropertyKey::Index(idx) = key {
            typed_array_run!({
                return Ok(slice.get(idx).map(|x| x.0.into()));
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
                return Ok(slice.get(idx).map(|x| x.0.into()));
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
            return Ok(())
        }

        self.get_wrapped_object().define_getter(key.into(), value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        if self.is_detached() {
            return self.get_wrapped_object().define_setter(name, value);
        }

        let key = InternalPropertyKey::from(name);
        if matches!(key, InternalPropertyKey::Index(_)) {
            return Ok(())
        }

        self.get_wrapped_object().define_setter(key.into(), value)
    }

    fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
        if self.is_detached() {
            return self.get_wrapped_object().delete_property(name);
        }

        let key = InternalPropertyKey::from(name.copy());
        if matches!(key, InternalPropertyKey::Index(_)) {
            return Ok(None)
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

    fn properties(&self) -> Res<Vec<(Value, Value)>> {
        if self.is_detached() {
            return self.get_wrapped_object().properties();
        }

        let mut props = typed_array_run!({
            slice.iter().enumerate().map(|(i, x)| (i.into(), x.0.into())).collect::<Vec<_>>()
        });

        props.append(&mut self.get_wrapped_object().properties()?);

        Ok(props)
    }

    fn keys(&self) -> Res<Vec<Value>> {
        if self.is_detached() {
            return self.get_wrapped_object().keys();
        }

        let mut keys = typed_array_run!({
            slice.iter().enumerate().map(|(i, _)| i.into()).collect::<Vec<_>>()
        });

        keys.append(&mut self.get_wrapped_object().keys()?);

        Ok(keys)
    }

    fn values(&self) -> Res<Vec<Value>> {
        if self.is_detached() {
            return self.get_wrapped_object().values();
        }

        let mut values = typed_array_run!({
            slice.iter().map(|x| x.0.into()).collect::<Vec<_>>()
        });

        values.append(&mut self.get_wrapped_object().values()?);

        Ok(values)
    }

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
        if self.is_detached() {
            return self.get_wrapped_object().get_array_or_done(index);
        }

        typed_array_run!({
            Ok((index < slice.len(), slice.get(index).map(|x| x.0.into())))
        })
    }
}

impl TypedArray {
    pub fn new(
        realm: &mut Realm,
        mut buffer: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
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
        let byte_length = byte_length.map_or_else(
            || usize::MAX,
            |len| {
                // if len + byte_offset > buf_len {
                //     return Err(Error::range("byteLength is out of bounds"));
                // } //TODO
                len
            },
        );

        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(realm.intrinsics.typed_array.clone().into())),
            buffer: buf,
            byte_offset,
            opt_byte_length: byte_length,
            ty,
        })
    }

    pub fn apply_offsets<'a>(&self, slice: &'a [u8]) -> Res<&'a [u8]> {
        let start = self.byte_offset;



        let mut end = start + self.opt_byte_length.min(slice.len() - start);
        end -= end % self.ty.size();

        if end > slice.len() {
            return Err(Error::range("TypedArray is out of bounds"));
        }

        slice
            .get(start..end)
            .ok_or_else(|| Error::range("TypedArray is out of bounds"))
    }

    pub fn apply_offsets_mut<'a>(&self, slice: &'a mut [u8]) -> Res<&'a mut [u8]> {
        let start = self.byte_offset;
        let mut end = start + self.opt_byte_length.min(slice.len() - start);
        end -= end % self.ty.size();

        if end > slice.len() {
            return Err(Error::range("TypedArray is out of bounds"));
        }

        slice
            .get_mut(start..end)
            .ok_or_else(|| Error::range("TypedArray is out of bounds"))
    }

    pub fn to_value_vec(&self) -> Res<Vec<Value>> {
        Ok(typed_array_run!({
            slice.iter().map(|x| x.0.into()).collect()
        }))
    }

    pub fn is_attached(&self) -> bool {
        self.buffer.inner.borrow().buffer.is_some()
    }

    pub fn is_detached(&self) -> bool {
        self.buffer.inner.borrow().buffer.is_none()
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
                buffer.extend_from_slice(&(item.to_number(realm)? as u64).to_le_bytes());
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
                buffer.extend_from_slice(&(item.to_number(realm)? as i64).to_le_bytes());
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

    #[get("buffer")]
    pub fn get_buffer(&self) -> ObjectHandle {
        self.buffer.gc().into()
    }

    #[get("byteLength")]
    pub fn get_byte_length(&self) -> usize {
        if self.opt_byte_length == usize::MAX {
            let buf_len = self.buffer.get_slice().map_or(0, |s| s.len());
            let len = buf_len.saturating_sub(self.byte_offset);
            len - (len % self.ty.size())
        } else {
            self.opt_byte_length - (self.opt_byte_length % self.ty.size())
        }
    }

    #[get("byteOffset")]
    pub const fn get_byte_offset(&self) -> usize {
        self.byte_offset
    }

    #[get("length")]
    pub fn get_length(&self) -> usize {
        if self.opt_byte_length == usize::MAX {
            let buf_len = self.buffer.get_slice().map_or(0, |s| s.len());
            let len = buf_len.saturating_sub(self.byte_offset);
            len / self.ty.size()
        } else {
            self.opt_byte_length / self.ty.size()
        }
    }

    pub fn at(&self, idx: usize) -> Res<Value> {
        Ok(typed_array_run!({
            slice
                .get(idx)
                .map_or(Value::Undefined, |x| Value::from(x.0))
        }))
    }

    #[prop("copyWithin")]
    pub fn copy_within(&self, target: usize, start: usize, end: Option<usize>, this: Value) -> ValueResult {
        fn oob(target: usize, start: usize, end: Option<usize>, len: usize) -> Option<(Range<usize>, usize)> {
            if target >= len {
                return None;
            }

            if start >= len {
                return None;
            }

            let end = end.unwrap_or(usize::MAX).min(start + (len - target));

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

    fn entries(&self, #[realm] realm: &Realm) -> ValueResult {
        let array = Array::with_elements(realm, self.to_value_vec()?)?.into_object();

        let iter = ArrayIterator {
            inner: RefCell::new(MutableArrayIterator {
                object: MutObject::with_proto(realm.intrinsics.array_iter.clone().into()),
            }),
            array,
            next: Cell::new(0),
            done: Cell::new(false),
        };

        Ok(iter.into_value())
    }

    fn every(
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
            // drop(slice0);
            for (idx, x) in owned.into_iter().enumerate() {
                let args = vec![x.0.into(), idx.into(), array.copy()];

                let res = callback.call(realm, args, Value::Undefined)?;

                if !res.is_truthy() {
                    return Ok(false);
                }
            }
        });

        Ok(true)
    }

    fn fill(
        &self,
        #[this] array: Value,
        #[realm] realm: &mut Realm,
        value: &Value,
        start: Option<isize>,
        end: Option<isize>,
    ) -> ValueResult {
        typed_array_run_mut!({
            let len = slice.len();

            let start = start.map_or(0, |start| convert_index(start, len));
            let end = end.map_or(len, |end| convert_index(end, len));

            let value: TY = FromPrimitive::from_f64(value.to_number(realm)?)
                .ok_or(Error::ty("Failed to convert to value"))?;

            for val in slice
                .get_mut(start..end)
                .ok_or(Error::range("TypedArray is out of bounds"))?
            {
                val.0 = value;
            }
        });

        Ok(array)
    }
}

fn bytemuck_err(err: bytemuck::PodCastError) -> Error {
    Error::new_error(err.to_string())
}
