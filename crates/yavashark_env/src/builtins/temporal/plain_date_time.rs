use crate::builtins::temporal::duration::{value_to_duration, Duration};
use crate::builtins::temporal::now::Now;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::utils::{
    difference_settings, disambiguation_opt, display_calendar, overflow_options, rounding_options,
    string_rounding_mode_opts,
};
use crate::builtins::temporal::zoned_date_time::ZonedDateTime;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::{Calendar, TimeZone};
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;
use crate::builtins::temporal::plain_time::{value_to_plain_time, PlainTime};

#[object]
#[derive(Debug)]
pub struct PlainDateTime {
    pub date: temporal_rs::PlainDateTime,
}

impl PlainDateTime {
    pub fn new(date: temporal_rs::PlainDateTime, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainDateTime {
                object: MutObject::with_proto(
                    realm.intrinsics.temporal_plain_date_time.clone().into(),
                ),
            }),
            date,
        }
    }

    pub fn now(realm: &Realm, tz: Option<TimeZone>) -> Res<temporal_rs::PlainDateTime> {
        Now::get_now()?
            .plain_date_time_iso_with_provider(tz, &realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &Realm, tz: Option<TimeZone>) -> Res<ObjectHandle> {
        let date = Self::now(realm, tz)?;

        Ok(Self::new(date, realm).into_object())
    }
}

#[props]
impl PlainDateTime {
    #[constructor]
    #[allow(clippy::too_many_arguments)]
    pub fn construct(
        year: i32,
        month: u8,
        day: u8,
        hour: Option<u8>,
        minute: Option<u8>,
        second: Option<u8>,
        millisecond: Option<u16>,
        microsecond: Option<u16>,
        nanosecond: Option<u16>,
        calendar: Option<String>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let millisecond = millisecond.unwrap_or(0);
        let microsecond = microsecond.unwrap_or(0);
        let nanosecond = nanosecond.unwrap_or(0);

        let calendar = calendar
            .as_deref()
            .map(Calendar::from_str)
            .transpose()
            .map_err(Error::from_temporal)?
            .unwrap_or_default();

        let datetime = temporal_rs::PlainDateTime::new(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            calendar,
        )
        .map_err(Error::from_temporal)?;

        Ok(Self::new(datetime, realm).into_object())
    }

    pub fn from(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let date = value_to_plain_date_time(info, realm)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[allow(clippy::use_self)]
    pub fn compare(left: &Value, right: &Value, #[realm] relam: &mut Realm) -> Res<i8> {
        let left = value_to_plain_date_time(left.clone(), relam)?;
        let right = value_to_plain_date_time(right.clone(), relam)?;

        Ok(left.compare_iso(&right) as i8)
    }

    pub fn equals(&self, other: &Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other = value_to_plain_date_time(other.clone(), realm)?;

        Ok(self.date == other)
    }

    pub fn since(
        &self,
        other: &Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other = value_to_plain_date_time(other.clone(), realm)?;

        let opts = opts
            .map(|s| difference_settings(s, realm))
            .transpose()?
            .unwrap_or_default();

        let duration = self
            .date
            .since(&other, opts)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, duration).into_object())
    }

    pub fn until(
        &self,
        other: &Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other = value_to_plain_date_time(other.clone(), realm)?;

        let opts = opts
            .map(|s| difference_settings(s, realm))
            .transpose()?
            .unwrap_or_default();

        let duration = self
            .date
            .until(&other, opts)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, duration).into_object())
    }

    pub fn add(
        &self,
        duration: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let dur = value_to_duration(duration, realm)?;

        let opts = opts
            .as_ref()
            .map(|s| overflow_options(s, realm))
            .transpose()?
            .flatten();

        let date = self.date.add(&dur, opts).map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    pub fn subtract(
        &self,
        duration: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let dur = value_to_duration(duration, realm)?;

        let opts = opts
            .as_ref()
            .map(|s| overflow_options(s, realm))
            .transpose()?
            .flatten();

        let date = self
            .date
            .subtract(&dur, opts)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    pub fn round(&self, opts: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let (opts, _) = rounding_options(opts, realm)?;

        let date = self.date.round(opts).map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> String {
        self.date.to_string()
    }

    #[prop("toString")]
    pub fn to_string_js(
        &self,
        options: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<String> {
        let display_calendar = display_calendar(options.as_ref(), realm)?;
        let opts = string_rounding_mode_opts(options, realm)?;

        self.date
            .to_ixdtf_string(opts, display_calendar)
            .map_err(Error::from_temporal)
    }

    #[prop("valueOf")]
    pub const fn value_of() -> Res {
        Err(Error::ty(
            "Called valueOf on a Temporal.PlainDateTime object",
        ))
    }

    #[get("day")]
    pub fn day(&self) -> u8 {
        self.date.day()
    }

    #[get("dayOfWeek")]
    pub fn day_of_week(&self) -> Res<u16> {
        self.date.day_of_week().map_err(Error::from_temporal)
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
    pub fn days_in_week(&self) -> Res<u16> {
        self.date.days_in_week().map_err(Error::from_temporal)
    }

    #[get("daysInYear")]
    pub fn days_in_year(&self) -> u16 {
        self.date.days_in_year()
    }

    #[get("era")]
    pub fn era(&self) -> Value {
        self.date.era().map_or(Value::Undefined, |era| {
            YSString::from_ref(era.as_str()).into()
        })
    }

    #[get("eraYear")]
    pub fn era_year(&self) -> Value {
        self.date.era_year().map_or(Value::Undefined, Into::into)
    }

    #[get("inLeapYear")]
    pub fn in_leap_year(&self) -> bool {
        self.date.in_leap_year()
    }

    #[get("hour")]
    pub fn hour(&self) -> u8 {
        self.date.hour()
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
        YSString::from_ref(self.date.month_code().as_str())
    }

    #[get("monthsInYear")]
    pub fn months_in_year(&self) -> u16 {
        self.date.months_in_year()
    }

    #[get("nanosecond")]
    pub fn nanosecond(&self) -> u16 {
        self.date.nanosecond()
    }

    #[get("second")]
    pub fn second(&self) -> u8 {
        self.date.second()
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

    #[get("calendarId")]
    pub fn calendar_id(&self) -> &'static str {
        self.date.calendar().identifier()
    }

    #[prop("toPlainDate")]
    pub fn to_plain_date(&self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self.date.to_plain_date().map_err(Error::from_temporal)?;

        Ok(PlainDate::new(date, realm).into_object())
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
    pub fn with_plain_time(&self, plain_time: Option<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        let plain_time = plain_time.map(|p| value_to_plain_time(p, realm))
            .transpose()?
            .unwrap_or_default();
        
        let dt = self.date.with_time(plain_time)
            .map_err(Error::from_temporal)?;
        
        Ok(Self::new(dt, realm).into_object())
    }

    #[prop("toZonedDateTime")]
    pub fn to_zoned_date_time(
        &self,
        options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let disambiguation = disambiguation_opt(options.as_ref(), realm)?;

        let date = self
            .date
            .to_zoned_date_time_with_provider(
                &TimeZone::default(),
                disambiguation.unwrap_or_default(),
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)?;

        Ok(ZonedDateTime::new(date, realm).into_object())
    }
}

pub fn value_to_plain_date_time(info: Value, realm: &mut Realm) -> Res<temporal_rs::PlainDateTime> {
    if let Value::Object(obj) = &info {
        if let Some(date) = obj.downcast::<PlainDateTime>() {
            return Ok(date.date.clone());
        }
    }

    if let Value::String(str) = &info {
        return temporal_rs::PlainDateTime::from_str(str.as_str()).map_err(Error::from_temporal);
    }

    let obj = info.to_object()?;

    if obj.contains_key(&"year".into())?
        || obj.contains_key(&"month".into())?
        || obj.contains_key(&"monthCode".into())?
        || obj.contains_key(&"day".into())?
    {
        let year = obj
            .resolve_property(&"year".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as i32))?;
        let month = obj
            .resolve_property(&"month".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;

        let month = if month == 0 {
            obj.resolve_property(&"monthCode".into(), realm)?
                .and_then(|v| v.to_string(realm).ok())
                .and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        s.as_str()[1..].parse::<u8>().ok()
                    }
                })
                .unwrap_or(0)
        } else {
            month
        };

        let day = obj
            .resolve_property(&"day".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
        let hour = obj
            .resolve_property(&"hour".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
        let minute = obj
            .resolve_property(&"minute".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
        let second = obj
            .resolve_property(&"second".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
        let millisecond = obj
            .resolve_property(&"millisecond".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u16))?;
        let microsecond = obj
            .resolve_property(&"microsecond".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u16))?;
        let nanosecond = obj
            .resolve_property(&"nanosecond".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u16))?;

        let calendar = obj
            .resolve_property(&"calendar".into(), realm)?
            .and_then(|v| v.to_string(realm).ok())
            .and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.as_str().to_string())
                }
            });

        let calendar = calendar
            .as_deref()
            .map(Calendar::from_str)
            .transpose()
            .map_err(Error::from_temporal)?
            .unwrap_or_default();

        let datetime = temporal_rs::PlainDateTime::new(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            calendar,
        )
        .map_err(Error::from_temporal)?;

        return Ok(datetime);
    }

    Err(Error::range("Invalid date")) //TODO
}
