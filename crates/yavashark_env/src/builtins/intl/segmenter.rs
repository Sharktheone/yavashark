use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Segmenter {}

impl Segmenter {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableSegmenter {
                object: MutObject::with_proto(realm.intrinsics.clone_public().intl_segmenter.get(realm)?.clone()),
            }),
        })
    }
}

#[props(intrinsic_name = intl_segmenter, to_string_tag = "Intl.Segmenter")]
impl Segmenter {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: String,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    fn segment(&self, _duration: ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
