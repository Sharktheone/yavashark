use crate::builtins::buf::ArrayBuffer;
use crate::builtins::typed_array::{Type, TypedArray};
use crate::value::Obj;
use crate::{Error, Object, ObjectHandle, Realm, Res, Value};
use base64::alphabet::{STANDARD, URL_SAFE};
use base64::{engine, Engine};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Uint8Array {}

impl Uint8Array {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.uint8array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableUint8Array {}),
            extends: ty,
        })
    }
}

#[props(extends = TypedArray)]
impl Uint8Array {
    #[both]
    const BYTES_PER_ELEMENT: usize = size_of::<u8>();

    #[constructor]
    #[length(3)]
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
    fn from_base_64(
        base64: &str,
        options: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let standard = if let Some(options) = options {
            options
                .resolve_property(&"alphabet".into(), realm)?
                .map_or(Ok(false), |x| x.normal_eq(&"base64url".into(), realm))?
        } else {
            false
        };

        let engine = if standard { &URL_SAFE } else { &STANDARD };

        let engine = engine::GeneralPurpose::new(engine, engine::GeneralPurposeConfig::default());

        let bytes = engine
            .decode(base64.as_bytes())
            .map_err(|e| Error::syn_error(e.to_string()))?;

        let array = ArrayBuffer::from_buffer(realm, bytes);

        let ty = TypedArray::new(realm, array.into_value(), None, None, Type::U8)?;

        Ok(Self::new(realm, ty)?.into_object())
    }

    #[prop("fromHex")]
    fn from_hex(hex: &str, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let bytes = hex::decode(hex).map_err(|e| Error::syn_error(e.to_string()))?;

        let array = ArrayBuffer::from_buffer(realm, bytes);

        let ty = TypedArray::new(realm, array.into_value(), None, None, Type::U8)?;

        Ok(Self::new(realm, ty)?.into_object())
    }

    #[prop("setFromBase64")]
    fn set_from_base_64(
        &self,
        base64: &str,
        options: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let standard = if let Some(options) = options {
            options
                .resolve_property(&"alphabet".into(), realm)?
                .map_or(Ok(false), |x| x.normal_eq(&"base64url".into(), realm))?
        } else {
            false
        };

        let engine = if standard { &URL_SAFE } else { &STANDARD };

        let engine = engine::GeneralPurpose::new(engine, engine::GeneralPurposeConfig::default());

        let buf = &self.extends.buffer;

        let mut inner = buf.inner.borrow_mut();

        let Some(mut inner_buf) = inner.buffer.as_mut() else {
            return Err(Error::ty("ArrayBuffer is detached"));
        };

        engine.decode_vec(base64.as_bytes(), &mut inner_buf)?;

        let written = inner_buf.len();
        let read = base64.len();

        let obj = Object::new(realm);

        obj.define_property("written".into(), written.into())?;
        obj.define_property("read".into(), read.into())?;

        Ok(obj)
    }

    #[prop("toBase64")]
    fn to_base_64(&self, options: Option<ObjectHandle>, #[realm] realm: &mut Realm) -> Res<String> {
        let standard = if let Some(options) = options {
            options
                .resolve_property(&"alphabet".into(), realm)?
                .map_or(Ok(false), |x| x.normal_eq(&"base64url".into(), realm))?
        } else {
            false
        };

        let engine = if standard { &URL_SAFE } else { &STANDARD };

        let engine = engine::GeneralPurpose::new(engine, engine::GeneralPurposeConfig::default());

        let buf = &self.extends.buffer;
        let slice = buf.get_slice()?;

        Ok(engine.encode(slice.as_ref()))
    }

    #[prop("toHex")]
    fn to_hex(&self) -> Res<String> {
        let buf = &self.extends.buffer;
        let slice = buf.get_slice()?;

        Ok(hex::encode(slice.as_ref()))
    }

    #[prop("setFromHex")]
    fn set_from_hex(&self, hex: &str) -> Res<()> {
        let buf = &self.extends.buffer;

        let mut inner = buf.inner.borrow_mut();

        let Some(inner_buf) = inner.buffer.as_mut() else {
            return Err(Error::ty("ArrayBuffer is detached"));
        };

        if inner_buf.len() < hex.len() * 2 {
            inner_buf.resize(hex.len() * 2, 0);
        }

        hex::encode_to_slice(hex, inner_buf)?;

        Ok(())
    }
}
