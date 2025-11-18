use crate::builtins::temporal::duration::{value_to_duration, Duration};
use crate::builtins::temporal::instant::Instant;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::plain_time::{value_to_plain_time, PlainTime};
use crate::builtins::temporal::utils::{
    difference_settings, disambiguation_opt, display_calendar, display_offset, display_timezone,
    offset_disambiguation_opt, overflow_options_opt, rounding_options, string_rounding_mode_opts,
    transition_direction, value_to_zoned_date_time_fields,
};
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::ops::BigIntOrNumber;
use crate::value::{IntoValue, Obj, Object};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::options::OffsetDisambiguation;
use temporal_rs::partial::PartialZonedDateTime;
use temporal_rs::{Calendar, MonthCode, Temporal, TimeZone, TinyAsciiStr, UtcOffset};
use temporal_rs::provider::COMPILED_TZ_PROVIDER;
use yavashark_macro::{object, props};
use yavashark_string::YSString;

#[object]
#[derive(Debug)]
pub struct ZonedDateTime {
    date: temporal_rs::ZonedDateTime,
}

impl ZonedDateTime {
    pub fn new(date: temporal_rs::ZonedDateTime, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableZonedDateTime {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .temporal_zoned_date_time
                        .get(realm)?
                        .clone(),
                ),
            }),
            date,
        })
    }

    pub fn now(tz: Option<TimeZone>) -> Res<temporal_rs::ZonedDateTime> {
        Temporal::now()
            .zoned_date_time_iso(tz)
            .map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &mut Realm, tz: Option<TimeZone>) -> Res<Self> {
        let date = Self::now(tz)?;

        Self::new(date, realm)
    }
}

#[props(intrinsic_name = temporal_zoned_date_time, to_string_tag = "Temporal.ZonedDateTime")]
impl ZonedDateTime {
    #[constructor]
    pub fn construct(
        ns: &BigIntOrNumber,
        tz: &str,
        calendar: Option<YSString>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let nanos = ns
            .to_big_int()
            .and_then(|n| n.to_i128())
            .ok_or_else(|| Error::ty("Invalid nanoseconds value"))?;

        let tz = TimeZone::try_from_str(tz).map_err(Error::from_temporal)?;

        let date = if let Some(cal) = calendar {
            let cal = Calendar::from_str(&cal).map_err(Error::from_temporal)?;

            temporal_rs::ZonedDateTime::try_new(nanos, tz, cal)
        } else {
            temporal_rs::ZonedDateTime::try_new_iso(nanos, tz)
        }
        .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    fn compare(left: &Value, right: &Value, realm: &mut Realm) -> Res<i8> {
        let left = value_to_zoned_date_time(left, None, realm)?;
        let right = value_to_zoned_date_time(right, None, realm)?;

        Ok(left.compare_instant(&right) as i8)
    }

    fn from(value: &Value, options: Option<ObjectHandle>, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = value_to_zoned_date_time(value, options, realm)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    fn add(
        &self,
        duration: Value,
        options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let options = overflow_options_opt(options.as_ref(), realm)?;

        let duration = value_to_duration(duration, realm)?;

        let date = self
            .date
            .add(&duration, options)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    fn equals(&self, other: Value, realm: &mut Realm) -> Res<bool> {
        let other = value_to_zoned_date_time(&other, None, realm)?;

        self.date
            .equals(&other)
            .map_err(Error::from_temporal)
    }

    #[prop("getTimeZoneTransition")]
    pub fn get_time_zone_transition(&self, options: &Value, realm: &mut Realm) -> Res<Value> {
        let direction = transition_direction(options, realm)?;

        let Some(transition) = self
            .date
            .get_time_zone_transition(direction)
            .map_err(Error::from_temporal)?
        else {
            return Ok(Value::Null);
        };

        Ok(Self::new(transition, realm)?.into_value())
    }

    pub fn round(&self, unit: Value, realm: &mut Realm) -> Res<ObjectHandle> {
        let (opts, _) = rounding_options(unit, realm)?;

        let date = self
            .date
            .round(opts)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    pub fn since(
        &self,
        other: &Value,
        options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other = value_to_zoned_date_time(other, None, realm)?;

        let settings = options
            .map(|opts| difference_settings(opts, realm))
            .transpose()?
            .unwrap_or_default();

        let dur = self
            .date
            .since(&other, settings)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, dur)?.into_object())
    }

    #[prop("startOfDay")]
    pub fn start_of_day(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .start_of_day()
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    pub fn subtract(
        &self,
        duration: Value,
        options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let options = overflow_options_opt(options.as_ref(), realm)?;

        let duration = value_to_duration(duration, realm)?;

        let date = self
            .date
            .subtract(&duration, options)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    #[prop("toInstant")]
    pub fn to_instant(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let instant = self.date.to_instant();

        Ok(Instant::from_stamp(instant, realm)?.into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> Res<String> {
        self.date
            .to_string_with_provider(&*COMPILED_TZ_PROVIDER)
            .map_err(Error::from_temporal)
    }

    #[prop("toPlainDate")]
    pub fn to_plain_date(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self.date.to_plain_date();

        Ok(PlainDate::new(date, realm)?.into_object())
    }

    #[prop("toPlainDateTime")]
    pub fn to_plain_date_time(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self.date.to_plain_date_time();

        Ok(PlainDateTime::new(date, realm)?.into_object())
    }

    #[prop("toPlainTime")]
    pub fn to_plain_time(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self.date.to_plain_time();

        Ok(PlainTime::new(date, realm)?.into_object())
    }

    #[prop("toString")]
    pub fn to_js_string(&self, options: Option<ObjectHandle>, realm: &mut Realm) -> Res<String> {
        let display_offset = display_offset(options.as_ref(), realm)?;
        let display_timezone = display_timezone(options.as_ref(), realm)?;
        let display_calendar = display_calendar(options.as_ref(), realm)?;

        let options = string_rounding_mode_opts(options, realm)?;

        self.date
            .to_ixdtf_string(
                display_offset,
                display_timezone,
                display_calendar,
                options,
            )
            .map_err(Error::from_temporal)
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self) -> Res<String> {
        self.date
            .to_string_with_provider(&*COMPILED_TZ_PROVIDER)
            .map_err(Error::from_temporal)
    }

    pub fn until(
        &self,
        other: &Value,
        options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other = value_to_zoned_date_time(other, None, realm)?;

        let settings = options
            .map(|opts| difference_settings(opts, realm))
            .transpose()?
            .unwrap_or_default();

        let dur = self
            .date
            .until(&other, settings)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, dur)?.into_object())
    }

    #[prop("valueOf")]
    #[nonstatic]
    pub const fn value_of() -> Res<()> {
        Err(Error::ty("ZonedDateTime does not support valueOf"))
    }

    pub fn with(&self, other: &ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        let overflow = overflow_options_opt(Some(other), realm)?;
        let disambiguation = disambiguation_opt(Some(other), realm)?;
        let offset_disambiguation = offset_disambiguation_opt(Some(other), realm)?;

        let fields = value_to_zoned_date_time_fields(other, false, realm)?;

        let date = self
            .date
            .with(fields, disambiguation, offset_disambiguation, overflow)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    #[prop("withCalendar")]
    pub fn with_calendar(&self, calendar: &str, realm: &mut Realm) -> Res<ObjectHandle> {
        let calendar = Calendar::from_str(calendar).map_err(Error::from_temporal)?;

        let date = self.date.with_calendar(calendar);

        Ok(Self::new(date, realm)?.into_object())
    }

    #[prop("withPlainTime")]
    pub fn with_plain_time(&self, time: Value, realm: &mut Realm) -> Res<ObjectHandle> {
        let time = value_to_plain_time(time, realm)?;

        let date = self
            .date
            .with_plain_time(Some(time))
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
    }

    #[prop("withTimeZone")]
    pub fn with_time_zone(&self, time_zone: &str, realm: &mut Realm) -> Res<Value> {
        let tz = TimeZone::try_from_str(time_zone).map_err(Error::from_temporal)?;

        let date = self.date.with_timezone(tz).map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_value())
    }

    #[get("calendarId")]
    pub fn calendar_id(&self) -> &'static str {
        self.date.calendar().identifier()
    }

    #[get("day")]
    pub fn day(&self) -> u8 {
        self.date.day()
    }

    #[get("dayOfWeek")]
    pub fn day_of_week(&self) -> u16 {
        self.date.day_of_week()
    }

    #[get("dayOfYear")]
    pub fn day_of_year(&self) -> u16 {
        self.date.day_of_year()
    }

    #[get("daysInMonth")]
    pub fn days_in_month(&self) -> u16 {
        self.date.days_in_month()
    }

    #[get("daysInWeek")]
    pub fn days_in_week(&self) -> u16 {
        self.date.days_in_week()
    }

    #[get("daysInYear")]
    pub fn days_in_year(&self) -> u16 {
        self.date.days_in_year()
    }

    #[get("epochMilliseconds")]
    pub fn epoch_milliseconds(&self) -> i64 {
        self.date.epoch_milliseconds()
    }

    #[get("epochNanoseconds")]
    pub fn epoch_nanoseconds(&self) -> BigInt {
        BigInt::from(self.date.epoch_nanoseconds().as_i128())
    }

    #[get("era")]
    pub fn era(&self) -> Value {
        self.date
            .era()
            .map(|era| YSString::from(era.to_string()))
            .map_or(Value::Undefined, Into::into)
    }

    #[get("eraYear")]
    pub fn era_year(&self) -> Value {
        self.date.era().map_or(Value::Undefined, |era| {
            YSString::from_ref(era.as_str()).into()
        })
    }

    #[get("hour")]
    pub fn hour(&self) -> u8 {
        self.date.hour()
    }

    #[get("hoursInDay")]
    pub fn hours_in_day(&self) -> Res<f64> {
        self.date.hours_in_day().map_err(Error::from_temporal)
    }

    #[get("inLeapYear")]
    pub fn in_leap_year(&self) -> bool {
        self.date.in_leap_year()
    }

    #[get("microsecond")]
    pub fn microsecond(&self) -> u16 {
        self.date.microsecond()
    }

    #[get("millisecond")]
    pub fn millisecond(&self) -> u16 {
        self.date.millisecond()
    }

    #[get("minute")]
    pub fn minute(&self) -> u8 {
        self.date.minute()
    }

    #[get("month")]
    pub fn month(&self) -> u8 {
        self.date.month()
    }

    #[get("monthCode")]
    pub fn month_code(&self) -> YSString {
        let month_code = self.date.month_code();

        YSString::from_ref(month_code.as_str())
    }

    #[get("monthsInYear")]
    pub fn months_in_year(&self) -> u16 {
        self.date.months_in_year()
    }

    #[get("nanosecond")]
    pub fn nanosecond(&self) -> u16 {
        self.date.nanosecond()
    }

    #[get("offset")]
    pub fn offset(&self) -> String {
        self.date.offset()
    }

    #[get("offsetNanoseconds")]
    pub fn offset_nanoseconds(&self) -> i64 {
        self.date.offset_nanoseconds()
    }

    #[get("second")]
    pub fn second(&self) -> u8 {
        self.date.second()
    }

    #[get("timeZoneId")]
    pub fn time_zone_id(&self) -> Res<String> {
        self.date
            .time_zone()
            .identifier()
            .map_err(Error::from_temporal)
    }

    #[get("weekOfYear")]
    pub fn week_of_year(&self) -> Value {
        self.date
            .week_of_year()
            .map_or(Value::Undefined, Into::into)
    }

    #[get("year")]
    pub fn year(&self) -> i32 {
        self.date.year()
    }

    #[get("yearOfWeek")]
    pub fn year_of_week(&self) -> Value {
        self.date
            .year_of_week()
            .map_or(Value::Undefined, Into::into)
    }
}

pub fn value_to_zoned_date_time(
    value: &Value,
    options: Option<ObjectHandle>,
    realm: &mut Realm,
) -> Res<temporal_rs::ZonedDateTime> {
    let disambiguation = disambiguation_opt(options.as_ref(), realm)?;
    let offset_disambiguation = offset_disambiguation_opt(options.as_ref(), realm)?;

    Ok(match value {
        Value::Object(obj) => {
            let overflow = overflow_options_opt(options.as_ref(), realm)?;

            if let Some(zdt) = obj.downcast::<ZonedDateTime>() {
                return Ok(zdt.date.clone());
            }

            let partial = partial_zoned_date_time(obj, realm)?;

            temporal_rs::ZonedDateTime::from_partial(
                partial,
                overflow,
                disambiguation,
                offset_disambiguation,
            )
            .map_err(Error::from_temporal)?
        }
        Value::String(str) => {
            let disambiguation = disambiguation.unwrap_or_default();
            let offset_disambiguation =
                offset_disambiguation.unwrap_or(OffsetDisambiguation::Reject);
            _ = overflow_options_opt(options.as_ref(), realm)?;

            temporal_rs::ZonedDateTime::from_utf8(
                str.as_str().as_bytes(),
                disambiguation,
                offset_disambiguation,
            )
            .map_err(Error::from_temporal)?
        }
        _ => return Err(Error::ty("Expected a ZonedDateTime object")),
    })
}

pub fn partial_zoned_date_time(obj: &ObjectHandle, realm: &mut Realm) -> Res<PartialZonedDateTime> {
    let mut partial = PartialZonedDateTime::new();

    if let Some(offset) = obj.get_opt("offset", realm)? {
        let Value::String(offset) = offset else {
            return Err(Error::ty("Expected offset to be a string"));
        };
        let offset = UtcOffset::from_str(&offset).map_err(Error::from_temporal)?;

        partial.fields.offset = Some(offset);
    }

    if let Some(calendar) = obj.get_opt("calendar", realm)? {
        let calendar = calendar.to_string(realm)?;
        let calendar = Calendar::from_str(&calendar).map_err(Error::from_temporal)?;

        partial.calendar = calendar;
    }

    let mut has_year = false;

    if let Some(ns) = obj.get_opt("era", realm)? {
        let era = ns.to_string(realm)?;
        let era = TinyAsciiStr::try_from_str(&era).map_err(|_| Error::ty("Invalid era string"))?;

        partial.fields.calendar_fields.era = Some(era);
        has_year = true;
    }

    if let Some(era_year) = obj.get_opt("eraYear", realm)? {
        let era_year = era_year.to_number(realm)?;
        partial.fields.calendar_fields.era_year = Some(era_year as i32);
        has_year = true;
    }

    if let Some(year) = obj.get_opt("year", realm)? {
        let year = year.to_number(realm)?;
        partial.fields.calendar_fields.year = Some(year as i32);
        has_year = true;
    }

    if !has_year {
        return Err(Error::ty("Expected year, era, or eraYear to be defined"));
    }

    let mut has_month = false;

    if let Some(month) = obj.get_opt("month", realm)? {
        let month = month.to_number(realm)?;
        partial.fields.calendar_fields.month = Some(month as u8);
        has_month = true;
    }

    if let Some(month_code) = obj.get_opt("monthCode", realm)? {
        let month_code = month_code.to_string(realm)?;

        let month_code =
            MonthCode::from_str(&month_code).map_err(|_| Error::ty("Invalid month code"))?;

        partial.fields.calendar_fields.month_code = Some(month_code);
        has_month = true;
    }

    if !has_month {
        return Err(Error::ty("Expected month or monthCode to be defined"));
    }

    if let Some(day) = obj.get_opt("day", realm)? {
        let day = day.to_number(realm)?;
        partial.fields.calendar_fields.day = Some(day as u8);
    } else {
        return Err(Error::ty("Expected day to be defined"));
    }

    if let Some(hour) = obj.get_opt("hour", realm)? {
        let hour = hour.to_number(realm)?;
        partial.fields.time = partial.fields.time.with_hour(Some(hour as u8));
    }

    if let Some(minute) = obj.get_opt("minute", realm)? {
        let minute = minute.to_number(realm)?;
        partial.fields.time = partial.fields.time.with_minute(Some(minute as u8));
    }

    if let Some(second) = obj.get_opt("second", realm)? {
        let second = second.to_number(realm)?;
        partial.fields.time = partial.fields.time.with_second(Some(second as u8));
    }

    if let Some(millisecond) = obj.get_opt("millisecond", realm)? {
        let millisecond = millisecond.to_number(realm)?;
        partial.fields.time = partial
            .fields
            .time
            .with_millisecond(Some(millisecond as u16));
    }

    if let Some(microsecond) = obj.get_opt("microsecond", realm)? {
        let microsecond = microsecond.to_number(realm)?;
        partial.fields.time = partial
            .fields
            .time
            .with_microsecond(Some(microsecond as u16));
    }

    if let Some(nanosecond) = obj.get_opt("nanosecond", realm)? {
        let nanosecond = nanosecond.to_number(realm)?;
        partial.fields.time = partial.fields.time.with_nanosecond(Some(nanosecond as u16));
    }

    if let Some(time_zone) = obj.get_opt("timeZone", realm)? {
        let time_zone = time_zone.to_string(realm)?;
        let time_zone = TimeZone::try_from_str(&time_zone).map_err(Error::from_temporal)?;

        partial.timezone = Some(time_zone);
    } else {
        return Err(Error::ty("Expected timeZone to be defined"));
    }

    Ok(partial)
}

impl PrettyObjectOverride for ZonedDateTime {
    fn pretty_inline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self.date.to_string();

        fmt_properties_to(obj, &mut s, not, realm);

        Some(s)
    }
}
