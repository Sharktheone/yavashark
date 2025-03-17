use crate::builtins::typed_array::{Type, TypedArray};
use crate::{ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object(extends = TypedArray)]
#[derive(Debug)]
pub struct Float64Array {}

impl Float64Array {
    pub fn new(realm: &Realm, ty: TypedArray) -> Res<Self> {
        ty.set_prototype(realm.intrinsics.int8array.clone().into())?;

        Ok(Self {
            inner: RefCell::new(MutableFloat64Array {}),
            extends: ty,
        })
    }
}

#[props]
impl Float64Array {
    const BYTES_PER_ELEMENT: usize = size_of::<f32>();
    
    #[constructor]
    fn construct(
        buf: Value,
        byte_offset: Option<usize>,
        byte_length: Option<usize>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let ty = TypedArray::new(realm, buf, byte_offset, byte_length, Type::F64)?;

        Ok(Self::new(realm, ty)?.into_object())
    }
}
