use crate::array::{convert_index, Array, ArrayIterator, MutableArrayIterator};
use crate::builtins::ArrayBuffer;
use crate::conversion::FromValueOutput;
use crate::utils::ValueIterator;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use half::f16;
use num_traits::FromPrimitive;
use std::cell::{Cell, RefCell};
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, props, typed_array_run, typed_array_run_mut};
use yavashark_value::{BoxedObj, Obj};

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

#[object(direct(buffer, byte_offset, byte_length))]
#[derive(Debug)]
pub struct TypedArray {
    #[allow(unused)]
    byte_offset: usize,
    byte_length: usize,
    ty: Type,
}

impl TypedArray {
    pub fn new(
        realm: &mut Realm,
        mut buffer: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        ty: Type,
    ) -> Res<Self> {
        let buf = if let Ok(buf) = <&ArrayBuffer>::from_value_out(buffer.copy()) {
            buf
        } else if buffer.contains_key(&"length".into()).ok().unwrap_or(false) {
            let iter = ValueIterator::new(&buffer, realm)?;

            let mut items = Vec::new();

            while let Some(item) = iter.next(realm)? {
                items.push(item);
            }

            buffer = convert_buffer(items, ty, realm)?.into_value();

            <&ArrayBuffer>::from_value_out(buffer.copy())?
        } else {
            let len = buffer.to_int_or_null() as usize;
            buffer = ArrayBuffer::new(realm, len).into_value();

            <&ArrayBuffer>::from_value_out(buffer.copy())?
        };

        let buf_len = buf.inner.borrow().buffer.len();
        let byte_offset = byte_offset.unwrap_or(0);

        // if byte_offset > buf_len { //TODO: re-implement this with BYTES_PER_ELEMENT
        //     return Err(Error::range("byteOffset is out of bounds"));
        // }
        //
        let byte_length = byte_length.map_or_else(|| buf_len - byte_offset, |len| {
            // if len + byte_offset > buf_len {
            //     return Err(Error::range("byteLength is out of bounds"));
            // } //TODO
            len
        });

        Ok(Self {
            inner: RefCell::new(MutableTypedArray {
                object: MutObject::with_proto(realm.intrinsics.typed_array.clone().into()),
                buffer: buffer.into(),
                byte_offset: byte_offset.into(),
                byte_length: byte_length.into(),
            }),
            byte_offset,
            byte_length,
            ty,
        })
    }

    pub fn get_buffer(&self) -> Res<OwningGcGuard<BoxedObj<Realm>, ArrayBuffer>> {
        let inner = self.inner.borrow();

        let buf = inner.buffer.value.clone();

        <&ArrayBuffer>::from_value_out(buf)
    }

    pub fn apply_offsets<'a>(&self, slice: &'a [u8]) -> Res<&'a [u8]> {
        let start = self.byte_offset;
        let end = start + self.byte_length;

        if end > slice.len() {
            return Err(Error::range("TypedArray is out of bounds"));
        }

        slice
            .get(start..end)
            .ok_or_else(|| Error::range("TypedArray is out of bounds"))
    }

    pub fn apply_offsets_mut<'a>(&self, slice: &'a mut [u8]) -> Res<&'a mut [u8]> {
        let start = self.byte_offset;
        let end = start + self.byte_length;

        if end > slice.len() {
            return Err(Error::range("TypedArray is out of bounds"));
        }

        slice
            .get_mut(start..end)
            .ok_or_else(|| Error::range("TypedArray is out of bounds"))
    }

    pub fn to_value_vec(&self) -> Res<Vec<Value>> {
        Ok(typed_array_run!({
            slice.iter().map(|x| (*x).into()).collect()
        }))
    }
}

fn convert_buffer(items: Vec<Value>, ty: Type, realm: &mut Realm) -> Res<ArrayBuffer> {
    let len = items.len()
        * match ty {
            Type::U8 | Type::I8 => 1,
            Type::U16 | Type::I16 | Type::F16 => 2,
            Type::U32 | Type::I32 | Type::F32 => 4,
            Type::U64 | Type::I64 | Type::F64=> 8,
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

    pub fn at(&self, idx: usize) -> Res<Value> {
        Ok(typed_array_run!({
            slice.get(idx).map_or(Value::Undefined, |x| Value::from(*x))
        }))
    }

    #[prop("copyWithin")]
    pub fn copy_within(&self, target: usize, start: usize, end: Option<usize>) -> Res<()> {
        typed_array_run_mut!({
            let end = end.unwrap_or(slice.len());

            slice.copy_within(start..end, target);
        });

        Ok(())
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
        typed_array_run!({
            for (idx, x) in slice.iter().enumerate() {
                let args = vec![(*x).into(), idx.into(), array.copy()];

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
                *val = value;
            }
        });

        Ok(array)
    }
}

// pub trait FromF64
