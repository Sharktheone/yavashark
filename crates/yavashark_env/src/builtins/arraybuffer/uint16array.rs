use crate::builtins::typed_array::{Type, TypedArray};
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Uint16Array {}

impl Uint16Array {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.int8array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableUint16Array {}),
            extends: ty,
        })
    }
}

#[props]
impl Uint16Array {
    #[constructor]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::U16)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
