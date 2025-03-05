use std::cell::RefCell;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, properties_new};
use yavashark_value::{BoxedObj};
use crate::{MutObject, Realm, Value, Result, Error};
use crate::builtins::ArrayBuffer;
use crate::conversion::FromValueOutput;

#[object(direct(buffer, byte_offset, byte_length))]
#[derive(Debug)]
pub struct TypedArray {
    #[allow(unused)]
    byte_offset: usize,
    
}


impl TypedArray {
    pub fn new(
        realm: &mut Realm,
        buffer: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
    ) -> Result<Self> {
        let buf = <&ArrayBuffer>::from_value_out(buffer.copy())?;
        let buf_len = buf.inner.borrow().buffer.len();
        let byte_offset = byte_offset.unwrap_or(0);

        if byte_offset > buf_len {
            return Err(Error::range("byteOffset is out of bounds"));
        }

        let byte_length = match byte_length {
            Some(len) => {
                if len + byte_offset > buf_len {
                    return Err(Error::range("byteLength is out of bounds"));
                }

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
        })
    }
    
    pub fn get_buffer(&self) -> Result<OwningGcGuard<BoxedObj<Realm>, ArrayBuffer>> {
        let inner = self.inner.borrow();

        let buf = inner.buffer.value.clone();

        <&ArrayBuffer>::from_value_out(buf)
    }
}


#[properties_new]
impl TypedArray {
    
}