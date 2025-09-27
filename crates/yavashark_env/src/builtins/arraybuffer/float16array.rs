use crate::builtins::typed_array::{Type, TypedArray};
use crate::{ObjectHandle, Realm, Res, Value};
use half::f16;
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Float16Array {}

impl Float16Array {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.int8array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableFloat16Array {}),
            extends: ty,
        })
    }
}

#[props(extends = TypedArray)]
impl Float16Array {
    #[both]
    const BYTES_PER_ELEMENT: usize = size_of::<f16>();

    #[constructor]
    #[length(3)]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::F16)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
