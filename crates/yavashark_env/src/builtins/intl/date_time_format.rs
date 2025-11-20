use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};
use crate::builtins::intl::utils::{HourCycle, LocaleMatcher, LocaleMatcherOptions, Style};

#[data_object]
pub enum FormatMatcher {
    Basic,
    #[name("best fit")]
    BestFit,
}

#[data_object]
pub enum DateTimeStyle {
    Full,
    Long,
    Medium,
    Short,
}


#[data_object]
pub enum NumberDigit {
    Numeric,
    #[name("2-digit")]
    TwoDigit,
}

#[data_object]
pub enum Month {
    Numeric,
    #[name("2-digit")]
    TwoDigit,
    Narrow,
    Short,
    Long,
}

#[data_object]
pub enum TimeZoneName {
    Short,
    Long,
    ShortOffset,
    LongOffset,
    ShortGeneric,
    LongGeneric,
}

#[data_object]
pub struct DateTimeFormatOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    pub calendar: Option<String>,
    #[prop("numberingSystem")]
    pub numbering_system: Option<String>,
    pub hour12: Option<bool>,
    #[prop("hourCycle")]
    pub hour_cycle: Option<HourCycle>,
    #[prop("timeZone")]
    pub time_zone: Option<String>,
    #[prop("formatMatcher")]
    pub format_matcher: Option<FormatMatcher>,
    pub weekday: Option<Style>,
    pub era: Option<Style>,
    pub year: Option<NumberDigit>,
    pub month: Option<Month>,
    pub day: Option<NumberDigit>,
    pub hour: Option<NumberDigit>,
    pub minute: Option<NumberDigit>,
    pub second: Option<NumberDigit>,
    #[prop("fractionalSecondDigits")]
    pub fractional_second_digits: Option<u8>,
    #[prop("timeZoneName")]
    pub time_zone_name: Option<TimeZoneName>,
    #[prop("dateStyle")]
    pub date_style: Option<DateTimeStyle>,
    #[prop("timeStyle")]
    pub time_style: Option<DateTimeStyle>,
}

#[object]
#[derive(Debug)]
pub struct DateTimeFormat {}

impl DateTimeFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableDateTimeFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_date_time_format
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_date_time_format, to_string_tag = "Intl.DateTimeFormat")]
impl DateTimeFormat {
    #[call_constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<DateTimeFormatOptions>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(_locales: String, _options: Option<LocaleMatcherOptions>) -> Vec<String> {
        Vec::new()
    }

    fn format(&self) -> String {
        String::new()
    }

    #[prop("formatRange")]
    fn format_range(&self, _start: String, _end: String) -> String {
        String::new()
    }

    #[prop("formatRangeToParts")]
    fn format_range_to_parts(&self, _start: String, _end: String) -> Vec<String> {
        Vec::new()
    }

    #[prop("formatToParts")]
    fn format_to_parts(&self, _date: String, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
