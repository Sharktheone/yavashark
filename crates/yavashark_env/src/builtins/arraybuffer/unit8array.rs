use crate::builtins::typed_array::{Type, TypedArray};
use crate::{Error, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Uint8Array {}

impl Uint8Array {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.int8array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableUint8Array {}),
            extends: ty,
        })
    }
}

#[props]
impl Uint8Array {
    const BYTES_PER_ELEMENT: usize = size_of::<u8>();

    #[constructor]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::U8)?;

        Ok(Self::new(realm, ty)?.into_object())
    }

    #[prop("fromBase64")]
    fn from_base_64(_base64: &str, _options: Option<ObjectHandle>) -> Res<ObjectHandle> {
        Err(Error::new("Not implemented"))
    }

    #[prop("fromHex")]
    fn from_hex(_hex: &str) -> Res<ObjectHandle> {
        Err(Error::new("Not implemented"))
    }

    #[prop("setFromBase64")]
    fn set_from_base_64(&self, _base64: &str, _options: Option<ObjectHandle>) -> Res<()> {
        Err(Error::new("Not implemented"))
    }

    #[prop("toBase64")]
    fn to_base_64(&self, _options: Option<ObjectHandle>) -> Res<String> {
        Err(Error::new("Not implemented"))
    }

    #[prop("toHex")]
    fn to_hex(&self) -> Res<String> {
        Err(Error::new("Not implemented"))
    }

    #[prop("setFromHex")]
    fn set_from_hex(&self, _hex: &str) -> Res<()> {
        Err(Error::new("Not implemented"))
    }
}
