use crate::builtins::typed_array::{Type, TypedArray};
use crate::value::Obj;
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Int16Array {}

impl Int16Array {
    pub fn new(realm: &mut Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(
            realm
                .intrinsics
                .clone_public()
                .int16array
                .get(realm)?
                .clone()
                .into(),
            realm,
        )?;

        Ok(Self {
            inner: RefCell::new(MutableInt16Array {}),
            extends: ty,
        })
    }
}

#[props(intrinsic_name = int16array, extends = TypedArray, extends_constructor)]
impl Int16Array {
    #[both]
    const BYTES_PER_ELEMENT: usize = size_of::<i16>();

    #[constructor]
    #[length(3)]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::I16)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
