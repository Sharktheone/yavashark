use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm};
use std::cell::RefCell;
use yavashark_macro::{object, props};

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
    fn construct(
        locales: Option<String>,
        options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> ObjectHandle {
        Self::new(realm).into_object()
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: String,
        _options: Option<ObjectHandle>,
        realm: &Realm,
    ) -> ObjectHandle {
        Array::from_realm(realm).into_object()
    }

    fn segment(&self, _duration: ObjectHandle, realm: &Realm) -> ObjectHandle {
        Array::from_realm(realm).into_object()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
