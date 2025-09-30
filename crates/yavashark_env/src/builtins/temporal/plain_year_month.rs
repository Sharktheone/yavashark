use crate::builtins::temporal::duration::{value_to_duration, Duration};
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_month_day::value_to_partial_date;
use crate::builtins::temporal::utils::{
    calendar_opt, difference_settings, display_calendar, overflow_options,
    value_to_year_month_fields,
};
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use crate::value::{Obj, Object};

#[object]
#[derive(Debug)]
pub struct PlainYearMonth {
    year_month: temporal_rs::PlainYearMonth,
}

impl PlainYearMonth {
    pub fn new(year_month: temporal_rs::PlainYearMonth, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainYearMonth {
                object: MutObject::with_proto(
                    realm.intrinsics.temporal_plain_year_month.clone().into(),
                ),
            }),
            year_month,
        }
    }
}

#[props]
impl PlainYearMonth {
    #[constructor]
    pub fn construct(
        year: i32,
        month: u8,
        calendar: Option<YSString>,
        reference_day: Option<u8>,
        realm: &Realm,
    ) -> Res<ObjectHandle> {
        let calendar = calendar_opt(calendar.as_deref())?;

        let year_month = temporal_rs::PlainYearMonth::new(year, month, reference_day, calendar)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(year_month, realm).into_object())
    }

    pub fn compare(left: Value, right: Value, #[realm] realm: &mut Realm) -> Res<i8> {
        let left = value_to_plain_year_month(left, None, realm)?;
        let right = value_to_plain_year_month(right, None, realm)?;

        Ok(left.compare_iso(&right) as i8)
    }

    pub fn from(
        value: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let year_month = value_to_plain_year_month(value, opts, realm)?;
        Ok(Self::new(year_month, realm).into_object())
    }

    pub fn add(
        &self,
        duration: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = opts
            .as_ref()
            .map(|opts| overflow_options(opts, realm))
            .transpose()?
            .flatten()
            .unwrap_or_default();

        let duration = value_to_duration(duration, realm)?;

        let year_month = self
            .year_month
            .add(&duration, opts)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(year_month, realm).into_object())
    }

    pub fn equals(&self, other: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other = value_to_plain_year_month(other, None, realm)?;

        Ok(self.year_month == other)
    }

    pub fn since(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = opts
            .map(|opts| difference_settings(opts, realm))
            .transpose()?
            .unwrap_or_default();

        let other = value_to_plain_year_month(other, None, realm)?;

        let duration = self
            .year_month
            .since(&other, opts)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, duration).into_object())
    }

    pub fn subtract(
        &self,
        duration: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = opts
            .as_ref()
            .map(|opts| overflow_options(opts, realm))
            .transpose()?
            .flatten()
            .unwrap_or_default();

        let duration = value_to_duration(duration, realm)?;

        let year_month = self
            .year_month
            .subtract(&duration, opts)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(year_month, realm).into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> Res<String> {
        Ok(self.year_month.to_string())
    }

    #[prop("toPlainDate")]
    pub fn to_plain_date(
        &self,
        day_info: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let day_info = day_info
            .map(|info| value_to_partial_date(&info, realm))
            .transpose()?
            .map(|date| date.calendar_fields);

        let plain_date = self
            .year_month
            .to_plain_date(day_info)
            .map_err(Error::from_temporal)?;

        Ok(PlainDate::new(plain_date, realm).into_object())
    }

    #[prop("toString")]
    pub fn to_js_string(
        &self,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<String> {
        let calendar = display_calendar(opts.as_ref(), realm)?;

        Ok(self.year_month.to_ixdtf_string(calendar))
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self) -> String {
        self.year_month.to_string()
    }

    pub fn until(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let opts = opts
            .map(|opts| difference_settings(opts, realm))
            .transpose()?
            .unwrap_or_default();

        let other = value_to_plain_year_month(other, None, realm)?;

        let duration = self
            .year_month
            .until(&other, opts)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, duration).into_object())
    }

    #[prop("valueOf")]
    #[nonstatic]
    pub const fn value_of() -> Res<()> {
        Err(Error::ty("`valueOf` is not supported for PlainYearMonth"))
    }

    pub fn with(&self, other: &ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let overflow = overflow_options(other, realm)?;
        let year_month = value_to_year_month_fields(other, realm)?;

        let year_month = self
            .year_month
            .with(year_month, overflow)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(year_month, realm).into_object())
    }

    #[get("calendarId")]
    pub fn calendar_id(&self) -> &'static str {
        self.year_month.calendar_id()
    }

    #[get("daysInMonth")]
    pub fn days_in_month(&self) -> u16 {
        self.year_month.days_in_month()
    }

    #[get("daysInYear")]
    pub fn days_in_year(&self) -> u16 {
        self.year_month.days_in_year()
    }

    #[get("era")]
    pub fn era(&self) -> Value {
        self.year_month
            .era()
            .as_deref()
            .map(YSString::from_ref)
            .map_or(Value::Undefined, Value::String)
    }

    #[get("eraYear")]
    pub fn era_year(&self) -> Value {
        self.year_month
            .era_year()
            .map_or(Value::Undefined, |v| Value::Number(f64::from(v)))
    }

    #[get("inLeapYear")]
    pub fn in_leap_year(&self) -> bool {
        self.year_month.in_leap_year()
    }

    #[get("month")]
    pub fn month(&self) -> u8 {
        self.year_month.month()
    }

    #[get("monthCode")]
    pub fn month_code(&self) -> YSString {
        YSString::from_ref(self.year_month.month_code().as_str())
    }

    #[get("monthsInYear")]
    pub fn months_in_year(&self) -> u16 {
        self.year_month.months_in_year()
    }

    #[get("year")]
    pub fn year(&self) -> i32 {
        self.year_month.year()
    }
}

pub fn value_to_plain_year_month(
    value: Value,
    opts: Option<ObjectHandle>,
    realm: &mut Realm,
) -> Res<temporal_rs::PlainYearMonth> {
    match value {
        Value::Object(obj) => {
            if let Some(obj) = obj.downcast::<PlainYearMonth>() {
                return Ok(obj.year_month.clone());
            }

            let opts = opts
                .as_ref()
                .map(|opts| overflow_options(opts, realm))
                .transpose()?
                .flatten()
                .unwrap_or_default();

            let calendar = obj
                .get_property_opt(&"calendar".into())?
                .map(|v| v.value)
                .and_then(|v| v.to_string(realm).ok());

            // let era = obj
            //     .get_property_opt(&"era".into())?
            //     .map(|v| v.value)
            //     .and_then(|v| v.to_string(realm).ok());
            //
            // let era_year = obj
            //     .get_property_opt(&"eraYear".into())?
            //     .map(|v| v.value)
            //     .and_then(|v| v.to_number(realm).ok());

            let month = obj
                .get_property_opt(&"month".into())?
                .map(|v| v.value)
                .and_then(|v| v.to_number(realm).ok())
                .map(|v| v as u8)
                .ok_or_else(|| Error::ty("Expected month to be a number"))?;

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

            let year = obj.get_property(&"year".into())?.value.to_number(realm)?;

            let year = year as i32;

            let calendar = calendar_opt(calendar.as_deref())?;

            temporal_rs::PlainYearMonth::new_with_overflow(year, month, None, calendar, opts)
                .map_err(Error::from_temporal)
        }
        Value::String(str) => {
            temporal_rs::PlainYearMonth::from_str(str.as_str()).map_err(Error::from_temporal)
        }

        _ => Err(Error::ty("Expected object or string")),
    }
}

impl PrettyObjectOverride for PlainYearMonth {
    fn pretty_inline(&self, obj: &Object, not: &mut Vec<usize>) -> Option<String> {
        let mut s = self.year_month.to_string();

        fmt_properties_to(obj, &mut s, not);

        Some(s)
    }
}
