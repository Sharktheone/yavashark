use crate::builtins::temporal::duration::Duration;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::cell::{Cell, RefCell};
use std::str::FromStr;
use temporal_rs::Calendar;
use temporal_rs::options::DifferenceSettings;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct PlainDateTime {
    date: temporal_rs::PlainDateTime,
}


impl PlainDateTime {
    pub fn new(date: temporal_rs::PlainDateTime, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainDateTime {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_date_time.clone().into()),
            }),
            date,
        }
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
        _calendar: Option<String>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let millisecond = millisecond.unwrap_or(0);
        let microsecond = microsecond.unwrap_or(0);
        let nanosecond = nanosecond.unwrap_or(0);
        
        
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
            Calendar::default()
        ).map_err(Error::from_temporal)?;
        
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

    pub fn since(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let duration = self.date.since(&other.date, DifferenceSettings::default())
            .map_err(Error::from_temporal)?;
        
        Ok(Duration::with_duration(realm, duration).into_object())
    }

    pub fn until(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let duration = other.date.until(&self.date, DifferenceSettings::default())
            .map_err(Error::from_temporal)?;
        
        Ok(Duration::with_duration(realm, duration).into_object())
    }

    pub fn add(&self, duration: &Duration, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self.date.add(&duration.dur, None)
            .map_err(Error::from_temporal)?;
        
        Ok(Self::new(date, realm).into_object())
    }

    pub fn subtract(&self, duration: &Duration, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self.date.subtract(&duration.dur, None)
            .map_err(Error::from_temporal)?;
        
        Ok(Self::new(date, realm).into_object())
        
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> String {
        self.date.to_string()
    }

    #[prop("toString")]
    pub fn to_string_js(&self) -> String {
        self.date.to_string()
    }

    #[prop("valueOf")]
    pub fn value_of() -> Res {
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
        self.date.day_of_week()
            .map_err(Error::from_temporal)
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
        self.date.days_in_week()
            .map_err(Error::from_temporal)
    }

    #[get("daysInYear")]
    pub fn days_in_year(&self) -> u16 {
        self.date.days_in_year()
    }

    #[get("era")]
    pub fn era(&self) -> Value {
        self.date.era()
            .map(|era| YSString::from_ref(era.as_str()).into())
            .unwrap_or(Value::Undefined)
            
    }

    #[get("eraYear")]
    pub fn era_year(&self) -> Value {
        self.date.era_year()
            .map(Into::into)
            .unwrap_or(Value::Undefined)
        
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
        self.date.week_of_year()
            .map(Into::into)
            .unwrap_or(Value::Undefined)
    }

    #[get("year")]
    pub fn year(&self) -> i32 {
        self.date.year()
    }

    #[get("yearOfWeek")]
    pub fn year_of_week(&self) -> Value {
        self.date.year_of_week()
            .map(Into::into)
            .unwrap_or(Value::Undefined)
    }
}

pub fn value_to_plain_date_time(info: Value, realm: &mut Realm) -> Res<temporal_rs::PlainDateTime> {
    if let Value::Object(obj) = &info {
        if let Some(date) = obj.downcast::<PlainDateTime>() {
            return Ok(date.date.clone());
        }
    }

    if let Value::String(str) = &info {
            return temporal_rs::PlainDateTime::from_str(str.as_str())
                .map_err(Error::from_temporal);
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
                Calendar::default()
            ).map_err(Error::from_temporal)?;

            return Ok(datetime);
        }

        Err(Error::range("Invalid date")) //TODO
}
