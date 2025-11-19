use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};

#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}

#[data_object]
pub enum Style {
    Long,
    Short,
    Narrow,
    Digital,
}

#[data_object]
pub enum Display {
    Auto,
    Always,
}

#[data_object]
pub enum UnitStyle {
    Long,
    Short,
    Narrow,
    Numeric,
    #[name("2-digit")]
    TwoDigit,
}

#[data_object]
pub struct DurationFormatOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    pub calendar: Option<String>,
    #[prop("numberingSystem")]
    pub numbering_system: Option<String>,
    pub style: Option<Style>,
    pub years: Option<UnitStyle>,
    #[prop("yearsDisplay")]
    pub years_display: Option<Display>,
    pub months: Option<UnitStyle>,
    #[prop("monthsDisplay")]
    pub months_display: Option<Display>,
    pub weeks: Option<UnitStyle>,
    #[prop("weeksDisplay")]
    pub weeks_display: Option<Display>,
    pub days: Option<UnitStyle>,
    #[prop("daysDisplay")]
    pub days_display: Option<Display>,
    pub hours: Option<UnitStyle>,
    #[prop("hoursDisplay")]
    pub hours_display: Option<Display>,
    pub minutes: Option<UnitStyle>,
    #[prop("minutesDisplay")]
    pub minutes_display: Option<Display>,
    pub seconds: Option<UnitStyle>,
    #[prop("secondsDisplay")]
    pub seconds_display: Option<Display>,
    pub milliseconds: Option<UnitStyle>,
    #[prop("millisecondsDisplay")]
    pub milliseconds_display: Option<Display>,
    pub microseconds: Option<UnitStyle>,
    #[prop("microsecondsDisplay")]
    pub microseconds_display: Option<Display>,
    pub nanoseconds: Option<UnitStyle>,
    #[prop("nanosecondsDisplay")]
    pub nanoseconds_display: Option<Display>,
    #[prop("fractionalDigits")]
    pub fractional_digits: Option<u8>,
}

#[object]
#[derive(Debug)]
pub struct DurationFormat {}

impl DurationFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableDurationFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_duration_format
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_duration_format, to_string_tag = "Intl.DurationFormat")]
impl DurationFormat {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<DurationFormatOptions>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(DurationFormat::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(_locales: String, _options: Option<ObjectHandle>) -> Vec<String> {
        Vec::new()
    }

    fn format(&self, _duration: ObjectHandle) -> String {
        String::new()
    }

    #[prop("formatToParts")]
    fn format_to_parts(&self, _duration: ObjectHandle) -> Vec<String> {
        Vec::new()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
