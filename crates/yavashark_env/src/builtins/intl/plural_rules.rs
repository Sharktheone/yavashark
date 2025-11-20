use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};
use crate::builtins::intl::utils::{LocaleMatcher, LocaleMatcherOptions};

#[data_object]
pub enum Type {
    Cardinal,
    Ordinal,
}

#[data_object]
pub enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

#[data_object]
pub enum RoundingPriority {
    Auto,
    MorePrecision,
    LessPrecision,
}

#[data_object]
pub enum TrailingZeroDisplay {
    Auto,
    StripIfInteger,
}

#[data_object]
pub struct PluralRulesOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    #[prop("type")]
    pub type_: Option<Type>,
    #[prop("minimumIntegerDigits")]
    pub minimum_integer_digits: Option<u8>,
    #[prop("minimumFractionDigits")]
    pub minimum_fraction_digits: Option<u8>,
    #[prop("maximumFractionDigits")]
    pub maximum_fraction_digits: Option<u8>,
    #[prop("minimumSignificantDigits")]
    pub minimum_significant_digits: Option<u8>,
    #[prop("maximumSignificantDigits")]
    pub maximum_significant_digits: Option<u8>,
    #[prop("roundingMode")]
    pub rounding_mode: Option<RoundingMode>,
    #[prop("roundingPriority")]
    pub rounding_priority: Option<RoundingPriority>,
    #[prop("roundingIncrement")]
    pub rounding_increment: Option<u64>,
    #[prop("trailingZeroDisplay")]
    pub trailing_zero_display: Option<TrailingZeroDisplay>,
}

#[object]
#[derive(Debug)]
pub struct PluralRules {}

impl PluralRules {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutablePluralRules {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_plural_rules
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_plural_rules, to_string_tag = "Intl.PluralRules")]
impl PluralRules {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<PluralRulesOptions>,
        realm: &mut Realm,
    ) -> Res<Self> {
        Self::new(realm)
    }

    #[prop("supportedLocalesOf")]
    pub fn supported_locales_of(_locales: String, _options: Option<LocaleMatcherOptions>) -> Vec<String> {
        Vec::new()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }

    #[prop("select")]
    fn select(&self, _number: f32) -> String {
        String::new()
    }

    #[prop("selectRange")]
    fn select_range(&self, _start: f32, _end: f32) -> String {
        String::new()
    }
}
