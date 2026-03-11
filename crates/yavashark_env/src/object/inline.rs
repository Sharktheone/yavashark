use crate::object::property_map::PropertyMap;
use crate::value::nan_boxed::ValueInner;


pub(crate) type Value = ValueInner;

#[repr(C)]
pub struct Object<T: ?Sized> {
    pub extensible: bool,
    pub props: PropertyMap<T>,
}




pub struct ButterFly {
    //TODO
}
