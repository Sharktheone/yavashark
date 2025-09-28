use std::cell::{Ref, RefCell, RefMut};
use yavashark_macro::{object, props};
use yavashark_value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use crate::array::convert_index;

#[object]
#[derive(Debug)]
pub struct ArrayBuffer {
    #[mutable]
    pub buffer: Option<Vec<u8>>,
    max_byte_length: Option<usize>,
    resizable: bool,
}

impl ArrayBuffer {
    pub fn new(realm: &mut Realm, len: usize) -> Res<Self> {
        if len > Self::ALLOC_MAX {
            return Err(Error::range("length too large"));
        }


        let buffer = vec![0; len];

        Ok(Self {
            inner: RefCell::new(MutableArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.arraybuffer.clone().into()),
                buffer: Some(buffer),
            }),
            max_byte_length: Some(len),
            resizable: true,
        })
    }

    pub fn from_buffer(realm: &mut Realm, buffer: Vec<u8>) -> Self {
        let len = buffer.len();

        Self {
            inner: RefCell::new(MutableArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.arraybuffer.clone().into()),
                buffer: Some(buffer),
            }),
            max_byte_length: Some(len),
            resizable: true,
        }
    }

    pub fn get_slice(&self) -> Res<Ref<'_, [u8]>> {
        let inner = self.inner.borrow();

        Ref::filter_map(inner, |x| x.buffer.as_deref())
            .map_err(|_| Error::ty("ArrayBuffer is detached"))
    }

    pub fn get_slice_mut(&self) -> Res<RefMut<'_, [u8]>> {
        let inner = self.inner.borrow_mut();

        RefMut::filter_map(inner, |x| x.buffer.as_deref_mut())
            .map_err(|_| Error::ty("ArrayBuffer is detached"))
    }

    pub fn detach(&self) -> Option<Vec<u8>> {
        let mut inner = self.inner.borrow_mut();
        inner.buffer.take()
    }

    const ALLOC_MAX: usize = 0xFFFFFFFF;
}

#[props]
impl ArrayBuffer {
    #[constructor]
    fn construct(realm: &mut Realm, len: usize, opts: Option<ObjectHandle>) -> ValueResult {
        let max_len = match opts.map(|v| {
            let x = v.get("maxByteLength", realm);

            x.and_then(|x| x.to_int_or_null(realm))
        }) {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        if max_len.is_some_and(i64::is_negative) {
            return Err(Error::range("maxByteLength must be positive"));
        }

        if len > max_len.unwrap_or(i64::MAX) as usize {
            return Err(Error::range(
                "length must be less than or equal to maxByteLength",
            ));
        }

        let max_len = max_len.map_or(len, |x| x as usize);

        if len > Self::ALLOC_MAX || max_len > Self::ALLOC_MAX {
            return Err(Error::range("length or maxByteLength too large"));
        }

        let buffer = vec![0; len];

        let buffer = ArrayBuffer {
            inner: RefCell::new(MutableArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.arraybuffer.clone().into()),
                buffer: Some(buffer),
            }),
            max_byte_length: Some(max_len),
            resizable: true,
        };

        Ok(buffer.into_value())
    }

    fn resize(&self, len: usize) -> Res {
        if !self.resizable {
            return Err(Error::ty("ArrayBuffer is not resizable"));
        }

        let mut inner = self.inner.borrow_mut();

        if let Some(buf) = inner.buffer.as_mut() {
            buf.resize(len, 0);
        } else {
            return Err(Error::ty("ArrayBuffer is detached"));
        }

        Ok(())
    }

    fn slice(
        &self,
        start: Option<isize>,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let inner = self.inner.borrow();

        let Some(buf) = &inner.buffer else {
            return Err(Error::ty("ArrayBuffer is detached"));
        };

        let start = start.unwrap_or(0);
        let end = end.unwrap_or(buf.len() as isize);

        let start = convert_index(start, buf.len());
        let end = convert_index(end, buf.len());

        let Some(buffer) = buf.get(start..end) else {
            return Ok(Self::new(realm, 0)?.into_value());
        };

        Ok(Self::from_buffer(realm, buffer.to_vec()).into_value())
    }


    fn transfer(&self, realm: &mut Realm) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        let Some(buf) = inner.buffer.take() else {
            return Err(Error::ty("ArrayBuffer is detached"));
        };

        Ok(Self::from_buffer(realm, buf).into_value())
    }


    #[prop("transferToFixedLength")]
    fn transfer_to_fixed_length(&self, new_len: Option<usize>, realm: &mut Realm) -> ValueResult {
        let mut inner = self.inner.borrow_mut();

        let Some(mut buf) = inner.buffer.take() else {
            return Err(Error::ty("ArrayBuffer is detached"));
        };

        if let Some(new_len) = new_len {
            buf.resize(new_len, 0);
        }



        Ok(Self::from_buffer(realm, buf).into_value())
    }

    #[get("resizable")]
    const fn resizable(&self) -> bool {
        self.resizable
    }

    #[get("byteLength")]
    fn byte_length(&self) -> usize {
        let inner = self.inner.borrow();

        let Some(buf) = &inner.buffer else {
            return 0;
        };

        buf.len()
    }

    #[get("maxByteLength")]
    fn max_byte_length(&self) -> usize {
        self.max_byte_length.unwrap_or(0)
    }

    #[get("detached")]
    fn detached(&self) -> bool {
        let inner = self.inner.borrow();

        inner.buffer.is_none()
    }

    #[prop("isView")]
    pub fn is_view(view: &Value, #[realm] realm: &mut Realm) -> Res<bool> {
        Ok(
            view.instance_of(&realm.intrinsics.typed_array_constructor().value, realm)?
                || view.instance_of(&realm.intrinsics.data_view_constructor().value, realm)?,
        )
    }
}
