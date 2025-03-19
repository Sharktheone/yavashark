use crate::builtins::typed_array::{Type, TypedArray};
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Uint8ClampedArray {}

impl Uint8ClampedArray {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.int8array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableUint8ClampedArray {}),
            extends: ty,
        })
    }
}

#[props(extends = TypedArray)]
impl Uint8ClampedArray {
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
}
