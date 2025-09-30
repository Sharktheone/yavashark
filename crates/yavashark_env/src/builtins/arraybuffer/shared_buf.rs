use std::cell::{Ref, RefCell, RefMut};
use yavashark_macro::{object, props};
use crate::value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, ValueResult};
use crate::array::convert_index;

#[object]
#[derive(Debug)]
pub struct SharedArrayBuffer {
    #[mutable]
    buffer: Vec<u8>,
    max_byte_length: Option<usize>,
    growable: bool,
}

impl SharedArrayBuffer {
    pub fn new(realm: &mut Realm, len: usize) -> Res<Self> {
        if len > Self::ALLOC_MAX {
            return Err(Error::range("length too large"));
        }


        let buffer = vec![0; len];

        Ok(Self {
            inner: RefCell::new(MutableSharedArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.sharedarraybuffer.clone()),
                buffer,
            }),
            max_byte_length: Some(len),
            growable: true,
        })
    }

    pub fn from_buffer(realm: &mut Realm, buffer: Vec<u8>) -> Self {
        let len = buffer.len();

        Self {
            inner: RefCell::new(MutableSharedArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.sharedarraybuffer.clone()),
                buffer,
            }),
            max_byte_length: Some(len),
            growable: true,
        }
    }

    pub fn get_slice(&self) -> Res<Ref<'_, [u8]>> {
        let inner = self.inner.borrow();

        Ok(
            Ref::map(inner, |x| x.buffer.as_slice())
        )
    }

    pub fn get_slice_mut(&self) -> Res<RefMut<'_, [u8]>> {
        let inner = self.inner.borrow_mut();

        Ok(
            RefMut::map(inner, |x| x.buffer.as_mut_slice())
        )
    }

    const ALLOC_MAX: usize = 0xFFFFFFFF;
}

#[props]
impl SharedArrayBuffer {
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

        let buffer = SharedArrayBuffer {
            inner: RefCell::new(MutableSharedArrayBuffer {
                object: MutObject::with_proto(realm.intrinsics.sharedarraybuffer.clone()),
                buffer,
            }),
            max_byte_length: Some(max_len),
            growable: true,
        };

        Ok(buffer.into_value())
    }

    fn grow(&self, len: usize) -> Res {
        if !self.growable {
            return Err(Error::ty("SharedArrayBuffer is not resizable"));
        }

        let mut inner = self.inner.borrow_mut();

        inner.buffer.resize(len, 0);

        Ok(())
    }

    fn slice(
        &self,
        start: Option<isize>,
        end: Option<isize>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        let inner = self.inner.borrow();

        let buf = &inner.buffer;

        let start = start.unwrap_or(0);
        let end = end.unwrap_or(buf.len() as isize);

        let start = convert_index(start, buf.len());
        let end = convert_index(end, buf.len());

        let Some(buffer) = buf.get(start..end) else {
            return Ok(Self::new(realm, 0)?.into_value());
        };

        Ok(Self::from_buffer(realm, buffer.to_vec()).into_value())
    }

    #[get("growable")]
    const fn growable(&self) -> bool {
        self.growable
    }

    #[get("byteLength")]
    fn byte_length(&self) -> usize {
        let inner = self.inner.borrow();

        inner.buffer.len()
    }

    #[get("maxByteLength")]
    fn max_byte_length(&self) -> usize {
        self.max_byte_length.unwrap_or(0)
    }
}
