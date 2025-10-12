use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct Segmenter {}

impl Segmenter {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableSegmenter {
                object: MutObject::with_proto(realm.intrinsics.intl_segmenter.clone()),
            }),
        }
    }
}


#[props]
impl Segmenter {
    #[constructor]
    fn construct() {}

}
