use crate::builtins::typed_array::{Type, TypedArray};
use crate::value::Obj;
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Uint8ClampedArray {}

impl Uint8ClampedArray {
    pub fn new(realm: &mut Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(
            realm
                .intrinsics
                .clone_public()
                .uint8clampedarray
                .get(realm)?
                .clone()
                .into(),
            realm,
        )?;

        Ok(Self {
            inner: RefCell::new(MutableUint8ClampedArray {}),
            extends: ty,
        })
    }
}

#[props(intrinsic_name = uint8clampedarray, extends = TypedArray)]
impl Uint8ClampedArray {
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
}
