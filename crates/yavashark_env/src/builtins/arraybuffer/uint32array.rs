use crate::builtins::typed_array::{Type, TypedArray};
use crate::value::Obj;
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Uint32Array {}

impl Uint32Array {
    pub fn new(realm: &mut Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.uint32array.clone().into(), realm)?;

        Ok(Self {
            inner: RefCell::new(MutableUint32Array {}),
            extends: ty,
        })
    }
}

#[props(intrinsic_name = uint32array, extends = TypedArray)]
impl Uint32Array {
    #[both]
    const BYTES_PER_ELEMENT: usize = size_of::<u32>();

    #[constructor]
    #[length(3)]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::U32)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
