use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};
use crate::value::Obj;

#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}

#[data_object]
pub enum Granularity {
    Grapheme,
    Word,
    Sentence,
}

#[data_object]
pub struct SegmenterOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    pub granularity: Option<Granularity>,
}

#[object]
#[derive(Debug)]
pub struct Segmenter {}

impl Segmenter {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableSegmenter {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_segmenter
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_segmenter, to_string_tag = "Intl.Segmenter")]
impl Segmenter {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<SegmenterOptions>,
        realm: &mut Realm,
    ) -> Res<Self> {
        Self::new(realm)
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(_locales: String, _options: Option<ObjectHandle>) -> Vec<String> {
        Vec::new()
    }

    fn segment(&self, _duration: ObjectHandle) -> Vec<String> {
        Vec::new()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
