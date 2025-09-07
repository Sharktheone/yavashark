mod from_bytes;

use crate::builtins::dataview::from_bytes::FromBytes;
use crate::builtins::ArrayBuffer;
use crate::conversion::downcast_obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, properties_new};
use yavashark_value::{BoxedObj, Constructor, Error, Obj};

#[object(direct(buffer, byte_offset, byte_length))]
#[derive(Debug)]
pub struct DataView {
    byte_offset: usize,
}

impl DataView {
    pub fn new(
        realm: &mut Realm,
        buffer: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
    ) -> Res<Self> {
        let buf = downcast_obj::<ArrayBuffer>(buffer.copy())?;
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
            inner: RefCell::new(MutableDataView {
                object: MutObject::with_proto(realm.intrinsics.data_view.clone().into()),
                buffer: buffer.into(),
                byte_offset: byte_offset.into(),
                byte_length: byte_length.into(),
            }),
            byte_offset,
        })
    }

    pub fn get_buffer(&self) -> Res<OwningGcGuard<'_, BoxedObj<Realm>, ArrayBuffer>> {
        let inner = self.inner.borrow();

        let buf = inner.buffer.value.clone();

        downcast_obj::<ArrayBuffer>(buf)
    }

    pub fn extract<T: FromBytes>(&self, offset: usize, le: bool) -> Res<T> {
        let buffer = self.get_buffer()?;

        let buffer = buffer.inner.borrow();

        let slice = buffer.buffer.as_slice();
        let offset = self.byte_offset + offset;

        let value = &slice
            .get(offset..offset + T::N_BYTES)
            .ok_or(Error::range("Out of bounds"))?;

        let bytes = T::Bytes::try_from(value).map_err(|_| Error::range("Out of bounds"))?;

        Ok(T::from_bytes(bytes, le))
    }

    pub fn set<T: FromBytes>(&self, offset: usize, value: T, le: bool) -> Res {
        let buffer = self.get_buffer()?;
        let mut buffer = buffer.inner.borrow_mut();
        let slice = buffer.buffer.as_mut_slice();

        let offset = self.byte_offset + offset;

        let bytes = T::to_bytes(value, le);
        let Some(slice) = slice.get_mut(offset..offset + T::N_BYTES) else {
            return Err(Error::range("Out of bounds"));
        };

        slice.copy_from_slice(bytes.as_ref());

        Ok(())
    }
}

#[properties_new(constructor(DataViewConstructor::new))]
impl DataView {
    #[prop("getFloat32")]
    pub fn get_float32(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        let value = self.extract::<f32>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getFloat64")]
    pub fn get_float64(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<f64>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getInt8")]
    pub fn get_int8(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<i8>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getInt16")]
    pub fn get_int16(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<i16>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getInt32")]
    pub fn get_int32(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<i32>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getBigInt64")]
    pub fn get_big_int64(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<i64>(offset, le)?;

        Ok(BigInt::from(value).into())
    }

    #[prop("getUint8")]
    pub fn get_uint8(&self, offset: usize, le: Option<bool>) -> ValueResult {
        let le = le.unwrap_or(false);
        let value = self.extract::<u8>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getUint16")]
    pub fn get_uint16(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<u16>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getUint32")]
    pub fn get_uint32(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<u32>(offset, le)?;

        Ok(value.into())
    }

    #[prop("getBigUint64")]
    pub fn get_big_uint64(&self, offset: usize, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);
        let value = self.extract::<u64>(offset, le)?;

        Ok(BigInt::from(value).into())
    }

    #[prop("setFloat32")]
    pub fn set_float32(&self, offset: usize, value: f64, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setFloat64")]
    pub fn set_float64(&self, offset: usize, value: f64, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setInt8")]
    pub fn set_int8(&self, offset: usize, value: i8, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setInt16")]
    pub fn set_int16(&self, offset: usize, value: i16, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setInt32")]
    pub fn set_int32(&self, offset: usize, value: i32, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setBigInt64")]
    pub fn set_big_int64(
        &self,
        offset: usize,
        value: &BigInt,
        little: Option<bool>,
    ) -> ValueResult {
        let le = little.unwrap_or(false);

        let value = value.to_i64().ok_or(Error::range("Out of bounds"))?;

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setUint8")]
    pub fn set_uint8(&self, offset: usize, value: u8, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setUint16")]
    pub fn set_uint16(&self, offset: usize, value: u16, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setUint32")]
    pub fn set_uint32(&self, offset: usize, value: u32, little: Option<bool>) -> ValueResult {
        let le = little.unwrap_or(false);

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }

    #[prop("setBigUint64")]
    pub fn set_big_uint64(
        &self,
        offset: usize,
        value: &BigInt,
        little: Option<bool>,
    ) -> ValueResult {
        let le = little.unwrap_or(false);

        let value = value.to_u64().ok_or(Error::range("Out of bounds"))?;

        self.set(offset, value, le)?;

        Ok(Value::Undefined)
    }
}
#[object(constructor)]
#[derive(Debug)]
pub struct DataViewConstructor {}

impl DataViewConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableDataViewConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        Ok(this.into_object())
    }
}

impl Constructor<Realm> for DataViewConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let buffer = args.first().map_or(
            Err(Error::new("DataView requires a buffer argument")),
            |v| Ok(v.clone()),
        )?;

        let byte_offset = match args.get(1).map(|v| v.to_number(realm).map(|v| v as usize)) {
            Some(Ok(v)) => Some(v),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        let byte_length = match args.get(2).map(|v| v.to_number(realm).map(|v| v as usize)) {
            Some(Ok(v)) => Some(v),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        Ok(DataView::new(realm, buffer, byte_offset, byte_length)?.into_value())
    }
}
