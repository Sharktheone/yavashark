use crate::builtins::temporal::duration::Duration;
use crate::builtins::temporal::utils::difference_settings;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::Calendar;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct PlainDate {
    date: temporal_rs::PlainDate,
}

impl PlainDate {
    pub fn new(date: temporal_rs::PlainDate, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainDate {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_date.clone().into()),
            }),
            date,
        }
    }
}

#[props]
impl PlainDate {
    #[constructor]
    pub fn construct(
        year: i32,
        month: u8,
        day: u8,
        calendar: Option<YSString>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let calendar = calendar.as_deref().map(Calendar::from_str)
            .transpose()
            .map_err(Error::from_temporal)?
            .unwrap_or_default();

        let date = temporal_rs::PlainDate::new(year, month, day, calendar)
            .map_err(Error::from_temporal)?;

        Ok(Self {
            inner: RefCell::new(MutablePlainDate {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_date.clone().into()),
            }),
            date,
        }
        .into_object())
    }

    pub fn from(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let date = value_to_plain_date(info, realm)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[allow(clippy::use_self)]
    pub fn compare(left: &PlainDate, right: &PlainDate) -> i8 {
        left.date.compare_iso(&right.date) as i8
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.date == other.date
    }

    pub fn since(
        &self,
        other: &Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other = value_to_plain_date(other.clone(), realm)?;
        
        let settings = opts
            .map(|s| difference_settings(s, realm))
            .transpose()?
            .unwrap_or_default();
        

        let dur = self
            .date
            .since(&other, settings)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, dur).into_object())
    }

    pub fn until(
        &self,
        other: &Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other = value_to_plain_date(other.clone(), realm)?;
        
        let settings = opts
            .map(|s| difference_settings(s, realm))
            .transpose()?
            .unwrap_or_default();

        let dur = self
            .date
            .until(&other, settings)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, dur).into_object())
    }

    pub fn add(&self, duration: &Duration, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .add(&duration.dur, None)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    pub fn subtract(&self, duration: &Duration, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self
            .date
            .subtract(&duration.dur, None)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm).into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> String {
        self.date.to_string()
    }

    #[prop("valueOf")]
    pub const fn value_of() -> Res {
        Err(Error::ty("Called valueOf on a Temporal.PlainDate object"))
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

pub fn value_to_plain_date(info: Value, realm: &mut Realm) -> Res<temporal_rs::PlainDate> {
    if let Value::String(str) = &info {
        let date = temporal_rs::PlainDate::from_str(str).map_err(Error::from_temporal)?;

        return Ok(date);
    }

    let obj = info.to_object()?;

    if let Some(this) = obj.downcast::<PlainDate>() {
        return Ok(this.date.clone());
    }

    if obj.contains_key(&"year".into())?
        || obj.contains_key(&"month".into())?
        || obj.contains_key(&"day".into())?
    {
        let year = obj
            .resolve_property(&"year".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as i32))?;
        let month = obj
            .resolve_property(&"month".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
        let day = obj
            .resolve_property(&"day".into(), realm)?
            .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;

        let calendar = obj
            .resolve_property(&"calendar".into(), realm)?
            .and_then(|v| v.to_string(realm).ok());

        let calendar = calendar.as_deref().map(Calendar::from_str)
            .transpose()
            .map_err(Error::from_temporal)?
            .unwrap_or_default();
        
        return  temporal_rs::PlainDate::new(year, month, day, calendar)
            .map_err(Error::from_temporal);
        
    }

    Err(Error::ty("Invalid date")) //TODO
}
