#![allow(dead_code)]
use crate::object::property_map::PropertyMap;
use crate::value::nan_boxed::ValueInner;

pub(crate) type Value = ValueInner;

#[repr(C)]
pub struct Object<T: ?Sized> {
    pub props: PropertyMap<T>,
}

pub struct ButterFly {
    //TODO
}
