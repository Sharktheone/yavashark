pub mod bigint64array;
pub mod biguint64array;
pub mod dataview;
pub mod float16array;
pub mod float32array;
pub mod float64array;
pub mod int16array;
pub mod int32array;
pub mod int8array;
pub mod typed_array;
pub mod uint16array;
pub mod uint32array;
pub mod uint8clampedarray;
pub mod unit8array;

use crate::array::convert_index;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::{Ref, RefCell, RefMut};
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Obj};

#[object(direct(
    max_byte_length(maxByteLength),
    byte_length(byteLength),
    resizable,
    detached
))]
#[derive(Debug)]
pub struct ArrayBuffer {
    #[mutable]
    buffer: Vec<u8>,
}

impl ArrayBuffer {
    pub fn new(realm: &mut Realm, len: usize) -> Self {
        let buffer = vec![0; len];

        Self {
            inner: RefCell::new(MutableArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.arraybuffer.clone().into()),
                max_byte_length: len.into(),
                byte_length: len.into(),
                resizable: true.into(),
                detached: false.into(),
                buffer,
            }),
        }
    }

    pub fn from_buffer(realm: &mut Realm, buffer: Vec<u8>) -> Self {
        let len = buffer.len();

        Self {
            inner: RefCell::new(MutableArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.arraybuffer.clone().into()),
                max_byte_length: len.into(),
                byte_length: len.into(),
                resizable: true.into(),
                detached: false.into(),
                buffer,
            }),
        }
    }

    pub fn get_slice(&self) -> Ref<[u8]> {
        let inner = self.inner.borrow();

        Ref::map(inner, |x| x.buffer.as_slice())
    }

    pub fn get_slice_mut(&self) -> RefMut<[u8]> {
        let inner = self.inner.borrow_mut();

        RefMut::map(inner, |x| x.buffer.as_mut())
    }
}

#[properties_new(constructor(ArrayBufferConstructor::new))]
impl ArrayBuffer {
    fn resize(&self, len: usize) {
        let mut inner = self.inner.borrow_mut();

        inner.byte_length = len.into();
        inner.buffer.resize(len, 0);
    }

    fn slice(
        &self,
        start: Option<isize>,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let inner = self.inner.borrow();

        let start = start.unwrap_or(0);
        let end = end.unwrap_or(inner.buffer.len() as isize);

        let start = convert_index(start, inner.buffer.len());
        let end = convert_index(end, inner.buffer.len());

        let Some(buffer) = inner.buffer.get(start..end) else {
            return Ok(Self::new(realm, 0).into_value());
        };

        Ok(Self::from_buffer(realm, buffer.to_vec()).into_value())
    }
}

#[object(constructor)]
#[derive(Debug)]
pub struct ArrayBufferConstructor {}

#[properties_new(raw)]
impl ArrayBufferConstructor {
    #[prop("isView")]
    pub fn is_view(&self, view: &Value, #[realm] realm: &mut Realm) -> Res<bool> {
        view.instance_of(&realm.intrinsics.typed_array_constructor().value, realm)
    }
}

impl ArrayBufferConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableArrayBufferConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

impl Constructor<Realm> for ArrayBufferConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let len = args.first().map_or(Ok(0), |v| v.to_int_or_null(realm))? as usize;
        let max_len = match args.get(1).map(|v| {
            let x = v.get_property(&"maxByteLength".into(), realm);

            x.and_then(|x| x.to_int_or_null(realm))
        }) {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        if max_len.is_some_and(i64::is_negative) {
            return Err(Error::range("maxByteLength must be positive"));
        }

        let max_len = max_len.map_or(len, |x| x as usize);

        let buffer = vec![0; len];

        let buffer = ArrayBuffer {
            inner: RefCell::new(MutableArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.arraybuffer.clone().into()),
                max_byte_length: max_len.into(),
                byte_length: len.into(),
                resizable: true.into(),
                detached: false.into(),
                buffer,
            }),
        };

        Ok(buffer.into_value())
    }
}
