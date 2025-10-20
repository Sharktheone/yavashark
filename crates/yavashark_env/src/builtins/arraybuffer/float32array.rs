use crate::builtins::typed_array::{Type, TypedArray};
use crate::value::Obj;
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Float32Array {}

impl Float32Array {
    pub fn new(realm: &mut Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.clone_public().float32array.get(realm)?.clone().into(), realm)?;

        Ok(Self {
            inner: RefCell::new(MutableFloat32Array {}),
            extends: ty,
        })
    }
}

#[props(intrinsic_name = float32array, extends = TypedArray)]
impl Float32Array {
    #[both]
    const BYTES_PER_ELEMENT: usize = size_of::<f32>();

    #[constructor]
    #[length(3)]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::F32)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
