use crate::builtins::{ArrayBuffer, ArrayBufferConstructor};
use crate::conversion::FromValueOutput;
use crate::{Error, MutObject, Realm, Res, Value};
use std::cell::RefCell;
use swc_ecma_ast::Pat::Array;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, properties_new};
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
        } else if buffer.contains_key(&"length".into())? {
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
        let byte_length = match byte_length {
            Some(len) => {
                // if len + byte_offset > buf_len {
                //     return Err(Error::range("byteLength is out of bounds"));
                // } //TODO

                len
            }
            None => buf_len - byte_offset,
        };

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

fn convert_buffer(items: Vec<Value>, ty: Type, realm: &mut Realm) -> Res<ArrayBuffer> {
    let len = items.len()
        * match ty {
            Type::U8 => 1,
            Type::U16 => 2,
            Type::U32 => 4,
            Type::U64 => 8,
            Type::I8 => 1,
            Type::I16 => 2,
            Type::I32 => 4,
            Type::I64 => 8,
            Type::F16 => 2,
            Type::F32 => 4,
            Type::F64 => 8,
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
                buffer.extend_from_slice(&(item.to_number(realm)? as f64).to_le_bytes());
            }
        }
    }

    Ok(ArrayBuffer::from_buffer(realm, buffer))
}

#[properties_new]
impl TypedArray {}
