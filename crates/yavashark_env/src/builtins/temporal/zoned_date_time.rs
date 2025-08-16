use crate::builtins::temporal::duration::{value_to_duration, Duration};
use crate::builtins::temporal::instant::Instant;
use crate::builtins::temporal::now::Now;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::plain_time::{value_to_plain_time, PlainTime};
use crate::builtins::temporal::utils::{
    difference_settings, disambiguation_opt, display_calendar, display_offset, display_timezone,
    offset_disambiguation_opt, overflow_options_opt, rounding_options, string_rounding_mode_opts,
    transition_direction,
};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::options::OffsetDisambiguation;
use temporal_rs::partial::PartialZonedDateTime;
use temporal_rs::{Calendar, MonthCode, TimeZone, TinyAsciiStr, UtcOffset};
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::ops::BigIntOrNumber;
use yavashark_value::{Obj, Object};
use crate::print::{fmt_properties_to, PrettyObjectOverride};

#[object]
#[derive(Debug)]
pub struct ZonedDateTime {
    date: temporal_rs::ZonedDateTime,
}

impl ZonedDateTime {
    pub fn new(date: temporal_rs::ZonedDateTime, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableZonedDateTime {
                object: MutObject::with_proto(
                    realm.intrinsics.temporal_zoned_date_time.clone().into(),
                ),
            }),
            date,
        }
    }

    pub fn now(tz: Option<TimeZone>) -> Res<temporal_rs::ZonedDateTime> {
        Now::get_now()?
            .zoned_date_time_iso(tz)
            .map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &Realm, tz: Option<TimeZone>) -> Res<ObjectHandle> {
        let date = Self::now(tz)?;

        Ok(Self::new(date, realm).into_object())
    }
}

#[props]
impl ZonedDateTime {
    #[constructor]
    pub fn construct(
        ns: &BigIntOrNumber,
        tz: &str,
        calendar: Option<YSString>,
        realm: &Realm,
    ) -> Res<ObjectHandle> {
        let nanos = ns
            .to_big_int()
            .and_then(|n| n.to_i128())
            .ok_or_else(|| Error::ty("Invalid nanoseconds value"))?;

        let tz = TimeZone::try_from_str(tz).map_err(Error::from_temporal)?;

        let date = if let Some(cal) = calendar {
            let cal = Calendar::from_str(&cal).map_err(Error::from_temporal)?;

            temporal_rs::ZonedDateTime::try_new(nanos, cal, tz)
        } else {
            temporal_rs::ZonedDateTime::try_new_iso(nanos, tz)
        }
        .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    fn compare(left: &Value, right: &Value, realm: &mut Realm) -> Res<i8> {
        let left = value_to_zoned_date_time(left, None, realm)?;
        let right = value_to_zoned_date_time(right, None, realm)?;

        Ok(left.compare_instant(&right) as i8)
    }

    fn from(value: &Value, options: Option<ObjectHandle>, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = value_to_zoned_date_time(value, options, realm)?;

        Ok(Self::new(date, realm).into_object())
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
            .add_with_provider(&duration, options, &realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    fn equals(&self, other: Value, realm: &mut Realm) -> Res<bool> {
        let other = value_to_zoned_date_time(&other, None, realm)?;

        Ok(self.date == other)
    }

    #[prop("getTimeZoneTransition")]
    pub fn get_time_zone_transition(&self, options: &Value, realm: &mut Realm) -> Res<Value> {
        let direction = transition_direction(options, realm)?;

        let Some(transition) = self
            .date
            .get_time_zone_transition_with_provider(direction, &realm.env.tz_provider)
            .map_err(Error::from_temporal)?
        else {
            return Ok(Value::Null);
        };

        Ok(Self::new(transition, realm).into_value())
    }

    pub fn round(&self, unit: Value, realm: &mut Realm) -> Res<ObjectHandle> {
        let (opts, _) = rounding_options(unit, realm)?;

        let date = self
            .date
            .round_with_provider(opts, &realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
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
            .since_with_provider(&other, settings, &realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, dur).into_object())
    }

    #[prop("startOfDay")]
    pub fn start_of_day(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .start_of_day_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
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
            .subtract_with_provider(&duration, options, &realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[prop("toInstant")]
    pub fn to_instant(&self, realm: &Realm) -> Res<ObjectHandle> {
        let instant = self.date.to_instant();

        Ok(Instant::from_stamp(instant, realm).into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self, realm: &Realm) -> Res<String> {
        self.date
            .to_string_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[prop("toPlainDate")]
    pub fn to_plain_date(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .to_plain_date_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(PlainDate::new(date, realm).into_object())
    }

    #[prop("toPlainDateTime")]
    pub fn to_plain_date_time(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .to_plain_datetime_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(PlainDateTime::new(date, realm).into_object())
    }

    #[prop("toPlainTime")]
    pub fn to_plain_time(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .to_plain_time_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(PlainTime::new(date, realm).into_object())
    }

    #[prop("toString")]
    pub fn to_js_string(&self, options: Option<ObjectHandle>, realm: &mut Realm) -> Res<String> {
        let display_offset = display_offset(options.as_ref(), realm)?;
        let display_timezone = display_timezone(options.as_ref(), realm)?;
        let display_calendar = display_calendar(options.as_ref(), realm)?;

        let options = string_rounding_mode_opts(options, realm)?;

        self.date
            .to_ixdtf_string_with_provider(
                display_offset,
                display_timezone,
                display_calendar,
                options,
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self, realm: &Realm) -> Res<String> {
        self.date
            .to_string_with_provider(&realm.env.tz_provider)
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
            .until_with_provider(&other, settings, &realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, dur).into_object())
    }

    #[prop("valueOf")]
    #[nonstatic]
    pub const fn value_of() -> Res<()> {
        Err(Error::ty("ZonedDateTime does not support valueOf"))
    }

    #[prop("withCalendar")]
    pub fn with_calendar(&self, calendar: &str, realm: &Realm) -> Res<ObjectHandle> {
        let calendar = Calendar::from_str(calendar).map_err(Error::from_temporal)?;

        let date = self
            .date
            .with_calendar(calendar)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[prop("withPlainTime")]
    pub fn with_plain_time(&self, time: Value, realm: &mut Realm) -> Res<ObjectHandle> {
        let time = value_to_plain_time(time, realm)?;

        let date = self
            .date
            .with_plain_time_and_provider(Some(time), &realm.env.tz_provider)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[prop("withTimeZone")]
    pub fn with_time_zone(&self, time_zone: &str, realm: &Realm) -> Res<Value> {
        let tz = TimeZone::try_from_str(time_zone).map_err(Error::from_temporal)?;

        let date = self.date.with_timezone(tz).map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_value())
    }

    #[get("calendarId")]
    pub fn calendar_id(&self) -> &'static str {
        self.date.calendar().identifier()
    }

    #[get("day")]
    pub fn day(&self, realm: &Realm) -> Res<u8> {
        self.date
            .day_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("dayOfWeek")]
    pub fn day_of_week(&self, realm: &Realm) -> Res<u16> {
        self.date
            .day_of_week_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("dayOfYear")]
    pub fn day_of_year(&self, realm: &Realm) -> Res<u16> {
        self.date
            .day_of_year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("daysInMonth")]
    pub fn days_in_month(&self, realm: &Realm) -> Res<u16> {
        self.date
            .days_in_month_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("daysInWeek")]
    pub fn days_in_week(&self, realm: &Realm) -> Res<u16> {
        self.date
            .days_in_week_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("daysInYear")]
    pub fn days_in_year(&self, realm: &Realm) -> Res<u16> {
        self.date
            .days_in_year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
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
    pub fn era(&self, realm: &Realm) -> Res<Value> {
        Ok(self
            .date
            .era_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?
            .map(|era| YSString::from(era.to_string()))
            .map_or(Value::Undefined, Into::into))
    }

    #[get("eraYear")]
    pub fn era_year(&self, realm: &Realm) -> Res<Value> {
        Ok(self
            .date
            .era_year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?
            .map_or(Value::Undefined, Into::into))
    }

    #[get("hour")]
    pub fn hour(&self, realm: &Realm) -> Res<u8> {
        self.date
            .hour_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("hoursInDay")]
    pub fn hours_in_day(&self, realm: &Realm) -> Res<u8> {
        self.date
            .hours_in_day_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("inLeapYear")]
    pub fn in_leap_year(&self, realm: &Realm) -> Res<bool> {
        self.date
            .in_leap_year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("microsecond")]
    pub fn microsecond(&self, realm: &Realm) -> Res<u16> {
        self.date
            .microsecond_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("millisecond")]
    pub fn millisecond(&self, realm: &Realm) -> Res<u16> {
        self.date
            .millisecond_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("minute")]
    pub fn minute(&self, realm: &Realm) -> Res<u8> {
        self.date
            .minute_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("month")]
    pub fn month(&self, realm: &Realm) -> Res<u8> {
        self.date
            .month_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("monthCode")]
    pub fn month_code(&self, realm: &Realm) -> Res<YSString> {
        self.date
            .month_code_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
            .map(|code| YSString::from_ref(code.as_str()))
    }

    #[get("monthsInYear")]
    pub fn months_in_year(&self, realm: &Realm) -> Res<u16> {
        self.date
            .months_in_year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("nanosecond")]
    pub fn nanosecond(&self, realm: &Realm) -> Res<u16> {
        self.date
            .nanosecond_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("offset")]
    pub fn offset(&self, realm: &Realm) -> Res<YSString> {
        self.date
            .offset_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
            .map(|offset| YSString::from_ref(offset.as_str()))
    }

    #[get("offsetNanoseconds")]
    pub fn offset_nanoseconds(&self, realm: &Realm) -> Res<i64> {
        self.date
            .offset_nanoseconds_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("second")]
    pub fn second(&self, realm: &Realm) -> Res<u8> {
        self.date
            .second_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("timeZoneId")]
    pub fn time_zone_id(&self) -> String {
        self.date.timezone().identifier()
    }

    #[get("weekOfYear")]
    pub fn week_of_year(&self, realm: &Realm) -> Res<Value> {
        Ok(self
            .date
            .week_of_year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?
            .map_or(Value::Undefined, Into::into))
    }

    #[get("year")]
    pub fn year(&self, realm: &Realm) -> Res<i32> {
        self.date
            .year_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[get("yearOfWeek")]
    pub fn year_of_week(&self, realm: &Realm) -> Res<Value> {
        Ok(self
            .date
            .year_of_week_with_provider(&realm.env.tz_provider)
            .map_err(Error::from_temporal)?
            .map_or(Value::Undefined, Into::into))
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
            if let Some(zdt) = obj.downcast::<ZonedDateTime>() {
                return Ok(zdt.date.clone());
            }

            let overflow = overflow_options_opt(options.as_ref(), realm)?;

            let partial = partial_zoned_date_time(obj, realm)?;

            temporal_rs::ZonedDateTime::from_partial_with_provider(
                partial,
                overflow,
                disambiguation,
                offset_disambiguation,
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)?
        }
        Value::String(str) => {
            let disambiguation = disambiguation.unwrap_or_default();
            let offset_disambiguation =
                offset_disambiguation.unwrap_or(OffsetDisambiguation::Reject);

            temporal_rs::ZonedDateTime::from_utf8_with_provider(
                str.as_str().as_bytes(),
                disambiguation,
                offset_disambiguation,
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)?
        }
        _ => return Err(Error::ty("Expected a ZonedDateTime object")),
    })
}

pub fn partial_zoned_date_time(obj: &ObjectHandle, realm: &mut Realm) -> Res<PartialZonedDateTime> {
    let mut partial = PartialZonedDateTime::new();

    if let Some(calendar) = obj.get_opt("calendar", realm)? {
        let calendar = calendar.to_string(realm)?;
        let calendar = Calendar::from_str(&calendar).map_err(Error::from_temporal)?;

        partial.date = partial.date.with_calendar(calendar);
    }

    let mut has_year = false;

    if let Some(ns) = obj.get_opt("era", realm)? {
        let era = ns.to_string(realm)?;
        let era = TinyAsciiStr::try_from_str(&era).map_err(|_| Error::ty("Invalid era string"))?;

        partial.date = partial.date.with_era(Some(era));
        has_year = true;
    }

    if let Some(era_year) = obj.get_opt("eraYear", realm)? {
        let era_year = era_year.to_number(realm)?;
        partial.date = partial.date.with_era_year(Some(era_year as i32));
        has_year = true;
    }

    if let Some(year) = obj.get_opt("year", realm)? {
        let year = year.to_number(realm)?;
        partial.date = partial.date.with_year(Some(year as i32));
        has_year = true;
    }

    if !has_year {
        return Err(Error::ty("Expected year, era, or eraYear to be defined"));
    }

    let mut has_month = false;

    if let Some(month) = obj.get_opt("month", realm)? {
        let month = month.to_number(realm)?;
        partial.date = partial.date.with_month(Some(month as u8));
        has_month = true;
    }

    if let Some(month_code) = obj.get_opt("monthCode", realm)? {
        let month_code = month_code.to_string(realm)?;

        let month_code =
            MonthCode::from_str(&month_code).map_err(|_| Error::ty("Invalid month code"))?;

        partial.date = partial.date.with_month_code(Some(month_code));
        has_month = true;
    }

    if !has_month {
        return Err(Error::ty("Expected month or monthCode to be defined"));
    }

    if let Some(day) = obj.get_opt("day", realm)? {
        let day = day.to_number(realm)?;
        partial.date = partial.date.with_day(Some(day as u8));
    } else {
        return Err(Error::ty("Expected day to be defined"));
    }

    if let Some(hour) = obj.get_opt("hour", realm)? {
        let hour = hour.to_number(realm)?;
        partial.time = partial.time.with_hour(Some(hour as u8));
    }

    if let Some(minute) = obj.get_opt("minute", realm)? {
        let minute = minute.to_number(realm)?;
        partial.time = partial.time.with_minute(Some(minute as u8));
    }

    if let Some(second) = obj.get_opt("second", realm)? {
        let second = second.to_number(realm)?;
        partial.time = partial.time.with_second(Some(second as u8));
    }

    if let Some(millisecond) = obj.get_opt("millisecond", realm)? {
        let millisecond = millisecond.to_number(realm)?;
        partial.time = partial.time.with_millisecond(Some(millisecond as u16));
    }

    if let Some(microsecond) = obj.get_opt("microsecond", realm)? {
        let microsecond = microsecond.to_number(realm)?;
        partial.time = partial.time.with_microsecond(Some(microsecond as u16));
    }

    if let Some(nanosecond) = obj.get_opt("nanosecond", realm)? {
        let nanosecond = nanosecond.to_number(realm)?;
        partial.time = partial.time.with_nanosecond(Some(nanosecond as u16));
    }

    if let Some(time_zone) = obj.get_opt("timeZone", realm)? {
        let time_zone = time_zone.to_string(realm)?;
        let time_zone = TimeZone::try_from_str(&time_zone).map_err(Error::from_temporal)?;

        partial.timezone = Some(time_zone);
    } else {
        return Err(Error::ty("Expected timeZone to be defined"));
    }

    if let Some(offset) = obj.get_opt("offset", realm)? {
        let offset = offset.to_string(realm)?;
        let offset = UtcOffset::from_str(&offset).map_err(Error::from_temporal)?;

        partial.offset = Some(offset);
    }

    Ok(partial)
}


impl PrettyObjectOverride for ZonedDateTime {
    fn pretty_inline(&self, obj: &Object<Realm>, not: &mut Vec<usize>) -> Option<String> {
        let mut s = self.date.to_string();

        fmt_properties_to(obj, &mut s, not);

        Some(s)
    }
}

