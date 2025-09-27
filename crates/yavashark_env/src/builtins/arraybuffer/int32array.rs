use crate::builtins::typed_array::{Type, TypedArray};
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Int32Array {}

impl Int32Array {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.int32array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableInt32Array {}),
            extends: ty,
        })
    }
}

#[props(extends = TypedArray)]
impl Int32Array {
    #[both]
    const BYTES_PER_ELEMENT: usize = size_of::<i32>();

    #[constructor]
    #[length(3)]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::I32)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
