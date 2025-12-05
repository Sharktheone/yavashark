use crate::array::Array;
use crate::builtins::intl::utils::{HourCycle, LocaleMatcher, LocaleMatcherOptions, Style};
use crate::value::{IntoValue, Obj};
use crate::{Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value};
use icu::datetime::fieldsets;
use icu::datetime::input::{Date, DateTime, Time};
use icu::datetime::options::Length;
use icu::datetime::{DateTimeFormatter, DateTimeFormatterPreferences};
use icu::locale::Locale;
use std::cell::RefCell;
use std::sync::Arc;
use yavashark_macro::{data_object, object, props};

#[derive(Clone, Copy, Debug)]
#[data_object(error = "range")]
pub enum FormatMatcher {
    Basic,
    #[name("best fit")]
    BestFit,
}

#[derive(Clone, Copy, Debug)]
#[data_object(error = "range")]
pub enum DateTimeStyle {
    Full,
    Long,
    Medium,
    Short,
}

impl DateTimeStyle {
    fn to_length(&self) -> Length {
        match self {
            DateTimeStyle::Full => Length::Long,
            DateTimeStyle::Long => Length::Long,
            DateTimeStyle::Medium => Length::Medium,
            DateTimeStyle::Short => Length::Short,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[data_object(error = "range")]
pub enum NumberDigit {
    Numeric,
    #[name("2-digit")]
    TwoDigit,
}

#[derive(Clone, Copy, Debug)]
#[data_object(error = "range")]
pub enum Month {
    Numeric,
    #[name("2-digit")]
    TwoDigit,
    Narrow,
    Short,
    Long,
}

#[derive(Clone, Copy, Debug)]
#[data_object(error = "range")]
pub enum TimeZoneName {
    Short,
    Long,
    ShortOffset,
    LongOffset,
    ShortGeneric,
    LongGeneric,
}

/// FractionalSecondDigits must be 1, 2, or 3
#[derive(Clone, Copy, Debug)]
pub enum FractionalSecondDigits {
    One = 1,
    Two = 2,
    Three = 3,
}

impl std::str::FromStr for FractionalSecondDigits {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(FractionalSecondDigits::One),
            "2" => Ok(FractionalSecondDigits::Two),
            "3" => Ok(FractionalSecondDigits::Three),
            _ => Err(Error::range_error(
                "fractionalSecondDigits must be 1, 2, or 3".to_string(),
            )),
        }
    }
}

impl crate::conversion::FromValueOutput for FractionalSecondDigits {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        let n = value.to_number(realm)?;

        // Check if it's NaN or not an integer
        if n.is_nan() || n.is_infinite() {
            return Err(Error::range_error(
                "fractionalSecondDigits must be 1, 2, or 3".to_string(),
            ));
        }

        // Check if it's an integer
        if n.fract() != 0.0 {
            return Err(Error::range_error(
                "fractionalSecondDigits must be 1, 2, or 3".to_string(),
            ));
        }

        let n = n as i64;
        match n {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            _ => Err(Error::range_error(
                "fractionalSecondDigits must be 1, 2, or 3".to_string(),
            )),
        }
    }
}

impl crate::value::IntoValue for FractionalSecondDigits {
    fn into_value(self) -> Value {
        (self as u8).into()
    }
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
    pub fractional_second_digits: Option<FractionalSecondDigits>,
    #[prop("timeZoneName")]
    pub time_zone_name: Option<TimeZoneName>,
    #[prop("dateStyle")]
    pub date_style: Option<DateTimeStyle>,
    #[prop("timeStyle")]
    pub time_style: Option<DateTimeStyle>,
}

#[derive(Clone, Debug)]
struct DateTimeFormatConfig {
    locale: Locale,
    calendar: String,
    numbering_system: Option<String>,
    time_zone: String,
    hour_cycle: Option<HourCycle>,
    weekday: Option<Style>,
    era: Option<Style>,
    year: Option<NumberDigit>,
    month: Option<Month>,
    day: Option<NumberDigit>,
    hour: Option<NumberDigit>,
    minute: Option<NumberDigit>,
    second: Option<NumberDigit>,
    fractional_second_digits: Option<FractionalSecondDigits>,
    time_zone_name: Option<TimeZoneName>,
    date_style: Option<DateTimeStyle>,
    time_style: Option<DateTimeStyle>,
}

impl DateTimeFormatConfig {
    fn format_date_time(&self, timestamp_ms: f64) -> String {
        if timestamp_ms.is_nan() || timestamp_ms.is_infinite() {
            return "Invalid Date".to_string();
        }

        let timestamp_secs = (timestamp_ms / 1000.0) as i64;
        let subsec_nanos = ((timestamp_ms % 1000.0) * 1_000_000.0) as u32;

        let chrono_dt = chrono::DateTime::from_timestamp(timestamp_secs, subsec_nanos)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default());

        let (year, month, day, hour, minute, second) = if self.time_zone == "UTC" {
            let dt = chrono_dt.naive_utc();
            (
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second(),
            )
        } else {
            let dt = chrono_dt.naive_local();
            (
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second(),
            )
        };

        let Ok(date) = Date::try_new_iso(year, month as u8, day as u8) else {
            return "Invalid Date".to_string();
        };
        let Ok(time) = Time::try_new(hour as u8, minute as u8, second as u8, 0) else {
            return "Invalid Date".to_string();
        };
        let datetime = DateTime { date, time };

        let prefs: DateTimeFormatterPreferences = (&self.locale).into();

        if let (Some(date_style), Some(time_style)) = (&self.date_style, &self.time_style) {
            return match (date_style.to_length(), time_style.to_length()) {
                (Length::Long, Length::Long) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMDE::long().with_time_hms())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Long, Length::Medium) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMDE::long().with_time_hms())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Long, Length::Short) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMDE::long().with_time_hm())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Medium, Length::Long) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium().with_time_hms())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Medium, Length::Medium) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium().with_time_hms())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Medium, Length::Short) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium().with_time_hm())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Short, Length::Long) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMD::short().with_time_hms())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Short, Length::Medium) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMD::short().with_time_hms())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                (Length::Short, Length::Short) => {
                    if let Ok(dtf) =
                        DateTimeFormatter::try_new(prefs, fieldsets::YMD::short().with_time_hm())
                    {
                        dtf.format(&datetime).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                _ => self.fallback_format(year, month, day, hour, minute, second),
            };
        } else if let Some(date_style) = &self.date_style {
            return match date_style.to_length() {
                Length::Long => {
                    if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::YMDE::long()) {
                        dtf.format(&date).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                Length::Medium => {
                    if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium()) {
                        dtf.format(&date).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                Length::Short => {
                    if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::YMD::short()) {
                        dtf.format(&date).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                _ => self.fallback_format(year, month, day, hour, minute, second),
            };
        } else if let Some(time_style) = &self.time_style {
            return match time_style.to_length() {
                Length::Long | Length::Medium => {
                    if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::T::medium()) {
                        dtf.format(&time).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                Length::Short => {
                    if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::T::short()) {
                        dtf.format(&time).to_string()
                    } else {
                        self.fallback_format(year, month, day, hour, minute, second)
                    }
                }
                _ => self.fallback_format(year, month, day, hour, minute, second),
            };
        }

        let has_date = self.year.is_some()
            || self.month.is_some()
            || self.day.is_some()
            || self.weekday.is_some();
        let has_time = self.hour.is_some() || self.minute.is_some() || self.second.is_some();

        if has_date && has_time {
            if let Ok(dtf) =
                DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium().with_time_hms())
            {
                return dtf.format(&datetime).to_string();
            }
        } else if has_date {
            if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium()) {
                return dtf.format(&date).to_string();
            }
        } else if has_time {
            if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::T::medium()) {
                return dtf.format(&time).to_string();
            }
        }

        if let Ok(dtf) = DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium()) {
            return dtf.format(&date).to_string();
        }

        self.fallback_format(year, month, day, hour, minute, second)
    }

    fn fallback_format(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    }
}

use chrono::{Datelike, Timelike};

#[object]
#[derive(Debug)]
pub struct DateTimeFormat {
    #[mutable]
    config: Arc<DateTimeFormatConfig>,
    #[mutable]
    bound_format: Option<ObjectHandle>,
}

impl DateTimeFormat {
    fn create(realm: &mut Realm, config: Arc<DateTimeFormatConfig>) -> Res<Self> {
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
                config,
                bound_format: None,
            }),
        })
    }
}

// https://tc39.es/ecma402/#sec-intl-datetimeformat-constructor
#[props(intrinsic_name = intl_date_time_format, to_string_tag = "Intl.DateTimeFormat")]
impl DateTimeFormat {
    // https://tc39.es/ecma402/#sec-intl.datetimeformat
    #[constructor]
    fn construct(
        locales: Option<String>,
        options: Option<DateTimeFormatOptions>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = options.unwrap_or(DateTimeFormatOptions {
            locale_matcher: None,
            calendar: None,
            numbering_system: None,
            hour12: None,
            hour_cycle: None,
            time_zone: None,
            format_matcher: None,
            weekday: None,
            era: None,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            fractional_second_digits: None,
            time_zone_name: None,
            date_style: None,
            time_style: None,
        });

        let locale_str = locales.unwrap_or_else(|| "en".to_string());
        let locale: Locale = locale_str
            .parse()
            .unwrap_or_else(|_| "en".parse().expect("en is a valid locale"));

        let has_date_components = opts.weekday.is_some()
            || opts.year.is_some()
            || opts.month.is_some()
            || opts.day.is_some();
        let has_time_components = opts.hour.is_some()
            || opts.minute.is_some()
            || opts.second.is_some()
            || opts.fractional_second_digits.is_some();
        let has_style = opts.date_style.is_some() || opts.time_style.is_some();

        if has_style && (has_date_components || has_time_components) {
            return Err(Error::ty_error(
                "Can't set option weekday, year, month, day, hour, minute, second, fractionalSecondDigits when dateStyle or timeStyle is used".to_string()
            ));
        }

        let time_zone = opts.time_zone.clone().unwrap_or_else(|| {
            iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".to_string())
        });

        let calendar = opts
            .calendar
            .clone()
            .unwrap_or_else(|| "gregory".to_string());

        let hour_cycle = if let Some(h12) = opts.hour12 {
            if h12 {
                Some(HourCycle::H12)
            } else {
                Some(HourCycle::H23)
            }
        } else {
            opts.hour_cycle
        };

        let (year, month, day) = if !has_style && !has_date_components && !has_time_components {
            (
                Some(NumberDigit::Numeric),
                Some(Month::Numeric),
                Some(NumberDigit::Numeric),
            )
        } else {
            (opts.year, opts.month, opts.day)
        };

        let config = Arc::new(DateTimeFormatConfig {
            locale,
            calendar,
            numbering_system: opts.numbering_system.clone(),
            time_zone,
            hour_cycle,
            weekday: opts.weekday,
            era: opts.era,
            year,
            month,
            day,
            hour: opts.hour,
            minute: opts.minute,
            second: opts.second,
            fractional_second_digits: opts.fractional_second_digits,
            time_zone_name: opts.time_zone_name,
            date_style: opts.date_style,
            time_style: opts.time_style,
        });

        Ok(Self::create(realm, config)?.into_object())
    }

    // https://tc39.es/ecma402/#sec-intl.datetimeformat.prototype.format
    #[get("format")]
    fn format(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut inner = self.inner.borrow_mut();

        if let Some(ref bound_format) = inner.bound_format {
            return Ok(bound_format.clone());
        }

        let config = Arc::clone(&inner.config);

        let format_fn = NativeFunction::with_proto_and_len(
            "get format",
            move |args, _this, realm| {
                let timestamp_ms = if let Some(date) = args.first() {
                    if date.is_undefined() {
                        get_current_time_ms()
                    } else {
                        let n = date.to_number(realm)?;
                        if n.is_nan() || n.is_infinite() {
                            return Err(Error::range_error("Invalid time value".to_string()));
                        }
                        n
                    }
                } else {
                    get_current_time_ms()
                };

                let result = config.format_date_time(timestamp_ms);
                Ok(Value::from(result))
            },
            realm.intrinsics.func.clone(),
            1,
            realm,
        );

        inner.bound_format = Some(format_fn.clone());
        Ok(format_fn)
    }

    // https://tc39.es/ecma402/#sec-intl.datetimeformat.prototype.formatRange
    #[prop("formatRange")]
    fn format_range(
        &self,
        start: Option<Value>,
        end: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<String> {
        // Step 4: If startDate is undefined or endDate is undefined, throw a TypeError
        let start = start.ok_or_else(|| Error::ty_error("startDate is undefined".to_string()))?;
        let end = end.ok_or_else(|| Error::ty_error("endDate is undefined".to_string()))?;

        if start.is_undefined() {
            return Err(Error::ty_error("startDate is undefined".to_string()));
        }
        if end.is_undefined() {
            return Err(Error::ty_error("endDate is undefined".to_string()));
        }

        // Step 5-6: Let x be ? ToNumber(startDate), let y be ? ToNumber(endDate)
        let start = start.to_number(realm)?;
        let end = end.to_number(realm)?;

        // Step 7-8: If x is NaN or y is NaN, throw a RangeError
        if start.is_nan() || start.is_infinite() {
            return Err(Error::range_error("Invalid time value".to_string()));
        }
        if end.is_nan() || end.is_infinite() {
            return Err(Error::range_error("Invalid time value".to_string()));
        }

        let inner = self.inner.borrow();
        let start_str = inner.config.format_date_time(start);
        let end_str = inner.config.format_date_time(end);

        Ok(format!("{} – {}", start_str, end_str))
    }

    // https://tc39.es/ecma402/#sec-Intl.DateTimeFormat.prototype.formatRangeToParts
    #[prop("formatRangeToParts")]
    fn format_range_to_parts(
        &self,
        start: Option<f64>,
        end: Option<f64>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        // Step 4: If startDate is undefined or endDate is undefined, throw a TypeError
        let start = start.ok_or_else(|| Error::ty_error("startDate is undefined".to_string()))?;
        let end = end.ok_or_else(|| Error::ty_error("endDate is undefined".to_string()))?;

        // Step 7-8: If x is NaN or y is NaN, throw a RangeError
        if start.is_nan() || start.is_infinite() {
            return Err(Error::range_error("Invalid time value".to_string()));
        }
        if end.is_nan() || end.is_infinite() {
            return Err(Error::range_error("Invalid time value".to_string()));
        }

        let inner = self.inner.borrow();
        let start_str = inner.config.format_date_time(start);
        let end_str = inner.config.format_date_time(end);

        let parts = Array::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let start_part = Object::new(realm);
        start_part.define_property("type".into(), "literal".into(), realm)?;
        start_part.define_property("value".into(), start_str.into(), realm)?;
        start_part.define_property("source".into(), "startRange".into(), realm)?;
        parts.push(start_part.into())?;

        let separator_part = Object::new(realm);
        separator_part.define_property("type".into(), "literal".into(), realm)?;
        separator_part.define_property("value".into(), " – ".into(), realm)?;
        separator_part.define_property("source".into(), "shared".into(), realm)?;
        parts.push(separator_part.into())?;

        let end_part = Object::new(realm);
        end_part.define_property("type".into(), "literal".into(), realm)?;
        end_part.define_property("value".into(), end_str.into(), realm)?;
        end_part.define_property("source".into(), "endRange".into(), realm)?;
        parts.push(end_part.into())?;

        Ok(parts.into_object())
    }

    // https://tc39.es/ecma402/#sec-Intl.DateTimeFormat.prototype.formatToParts
    #[prop("formatToParts")]
    #[length(1)]
    fn format_to_parts(&self, date: Option<f64>, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let timestamp_ms = date.unwrap_or_else(get_current_time_ms);

        if timestamp_ms.is_nan() || timestamp_ms.is_infinite() {
            return Err(Error::range_error("Invalid time value".to_string()));
        }

        let inner = self.inner.borrow();
        let formatted = inner.config.format_date_time(timestamp_ms);

        let parts = Array::new(realm.intrinsics.clone_public().array.get(realm)?.clone());

        let part = Object::new(realm);
        part.define_property("type".into(), "literal".into(), realm)?;
        part.define_property("value".into(), formatted.into(), realm)?;
        parts.push(part.into())?;

        Ok(parts.into_object())
    }

    // https://tc39.es/ecma402/#sec-intl.datetimeformat.prototype.resolvedoptions
    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let inner = self.inner.borrow();
        let config = &inner.config;

        let options = Object::new(realm);

        options.define_property("locale".into(), config.locale.to_string().into(), realm)?;
        options.define_property("calendar".into(), config.calendar.clone().into(), realm)?;

        if let Some(ref ns) = config.numbering_system {
            options.define_property("numberingSystem".into(), ns.clone().into(), realm)?;
        }

        options.define_property("timeZone".into(), config.time_zone.clone().into(), realm)?;

        if let Some(ref hc) = config.hour_cycle {
            let hc_str = hc.as_str();
            options.define_property("hourCycle".into(), hc_str.into(), realm)?;

            let hour12 = matches!(hc, HourCycle::H11 | HourCycle::H12);
            options.define_property("hour12".into(), hour12.into(), realm)?;
        }

        if let Some(ref weekday) = config.weekday {
            options.define_property("weekday".into(), weekday.into_value(), realm)?;
        }

        if let Some(ref era) = config.era {
            options.define_property("era".into(), era.into_value(), realm)?;
        }

        if let Some(ref year) = config.year {
            options.define_property("year".into(), year.into_value(), realm)?;
        }

        if let Some(ref month) = config.month {
            options.define_property("month".into(), month.into_value(), realm)?;
        }

        if let Some(ref day) = config.day {
            options.define_property("day".into(), day.into_value(), realm)?;
        }

        if let Some(ref hour) = config.hour {
            options.define_property("hour".into(), hour.into_value(), realm)?;
        }

        if let Some(ref minute) = config.minute {
            options.define_property("minute".into(), minute.into_value(), realm)?;
        }

        if let Some(ref second) = config.second {
            options.define_property("second".into(), second.into_value(), realm)?;
        }

        if let Some(fsd) = config.fractional_second_digits {
            options.define_property("fractionalSecondDigits".into(), fsd.into_value(), realm)?;
        }

        if let Some(ref tzn) = config.time_zone_name {
            options.define_property("timeZoneName".into(), tzn.into_value(), realm)?;
        }

        if let Some(ref ds) = config.date_style {
            options.define_property("dateStyle".into(), ds.into_value(), realm)?;
        }

        if let Some(ref ts) = config.time_style {
            options.define_property("timeStyle".into(), ts.into_value(), realm)?;
        }

        Ok(options)
    }

    // https://tc39.es/ecma402/#sec-intl.datetimeformat.supportedlocalesof
    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        locales: &Value,
        _options: Option<LocaleMatcherOptions>,
        #[realm] realm: &mut Realm,
    ) -> Res<Vec<String>> {
        let locale_list = canonicalize_locale_list(locales, realm)?;

        let mut supported = Vec::new();
        for locale_str in locale_list {
            if let Ok(locale) = locale_str.parse::<Locale>() {
                let prefs: DateTimeFormatterPreferences = (&locale).into();
                let fieldset = fieldsets::YMD::medium();
                if DateTimeFormatter::try_new(prefs, fieldset).is_ok() {
                    supported.push(locale.to_string());
                }
            }
        }
        Ok(supported)
    }
}

fn get_current_time_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

fn canonicalize_locale_list(locales: &Value, realm: &mut Realm) -> Res<Vec<String>> {
    if locales.is_undefined() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();

    if locales.is_string() {
        result.push(locales.to_string(realm)?.to_string());
    } else if let Value::Object(obj) = &locales {
        let length = obj.get("length", realm)?.to_number(realm)? as usize;

        for i in 0..length {
            let locale_value = obj.get(i, realm)?;
            if !locale_value.is_undefined() && !locale_value.is_null() {
                let locale_str = locale_value.to_string(realm)?;
                result.push(locale_str.to_string());
            }
        }
    }

    Ok(result)
}
