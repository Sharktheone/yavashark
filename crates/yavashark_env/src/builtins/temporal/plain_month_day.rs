use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::utils::{
    calendar_opt, display_calendar, overflow_options, overflow_options_opt,
    value_to_calendar_fields,
};
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::options::Overflow;
use temporal_rs::partial::PartialDate;
use temporal_rs::Calendar;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use crate::value::{Obj, Object};

#[object]
#[derive(Debug)]
pub struct PlainMonthDay {
    month_day: temporal_rs::PlainMonthDay,
}

impl PlainMonthDay {
    pub fn new(month_day: temporal_rs::PlainMonthDay, realm: &crate::Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainMonthDay {
                object: MutObject::with_proto(
                    realm.intrinsics.temporal_plain_month_day.clone().into(),
                ),
            }),
            month_day,
        }
    }
}

#[props]
impl PlainMonthDay {
    #[constructor]
    pub fn construct(
        month: u8,
        day: u8,
        calendar: Option<YSString>,
        ref_year: Option<i32>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let calendar = calendar_opt(calendar.as_deref())?;

        let month_day = temporal_rs::PlainMonthDay::new_with_overflow(
            month,
            day,
            calendar,
            Overflow::Constrain,
            ref_year,
        )
        .map_err(Error::from_temporal)?;

        Ok(Self::new(month_day, realm).into_object())
    }

    pub fn from(
        info: Value,
        options: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let overflow = options
            .as_ref()
            .map(|o| overflow_options(o, realm))
            .transpose()?
            .flatten();

        let mut month_day = value_to_plain_month_day(info, realm, overflow)?;

        month_day.iso.year = 1972;

        Ok(Self::new(month_day, realm).into_object())
    }

    pub fn equals(&self, other: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other = value_to_plain_month_day(other, realm, None)?;

        Ok(self.month_day == other)
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> String {
        self.month_day.to_string()
    }

    #[prop("toPlainDate")]
    pub fn to_plain_date(
        &self,
        info: ObjectHandle,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let date = value_to_partial_date(&info, realm)?;

        let plain_date = self
            .month_day
            .to_plain_date(Some(date.calendar_fields))
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

        Ok(self.month_day.to_ixdtf_string(calendar))
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self) -> String {
        self.month_day.to_string()
    }

    #[prop("valueOf")]
    #[nonstatic]
    pub const fn value_of() -> Res<()> {
        Err(Error::ty(
            "Called valueOf on a Temporal.PlainMonthDay object",
        ))
    }

    fn with(&self, other: &ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let overflow = overflow_options_opt(Some(other), realm)?;
        let fields = value_to_calendar_fields(other, realm)?;

        let month_day = self
            .month_day
            .with(fields, overflow)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(month_day, realm).into_object())
    }

    #[get("calendarId")]
    pub fn calendar_id(&self) -> &'static str {
        self.month_day.calendar_id()
    }

    #[get("day")]
    pub fn day(&self) -> u8 {
        self.month_day.day()
    }

    #[get("monthCode")]
    pub fn month_code(&self) -> YSString {
        YSString::from_ref(self.month_day.month_code().as_str())
    }
}

pub fn value_to_plain_month_day(
    value: Value,
    realm: &mut Realm,
    overflow: Option<Overflow>,
) -> Res<temporal_rs::PlainMonthDay> {
    match value {
        Value::Object(obj) => {
            if let Some(plain_month_day) = obj.downcast::<PlainMonthDay>() {
                return Ok(plain_month_day.month_day.clone());
            }

            let overflow = overflow.unwrap_or(Overflow::Constrain);

            if (obj.contains_key(&"month".into())? || obj.contains_key(&"monthCode".into())?)
                && obj.contains_key(&"day".into())?
            {
                let year = obj
                    .resolve_property(&"year".into(), realm)?
                    .map(|v| v.to_number(realm).map(|v| v as i32))
                    .transpose()?;

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

                let calendar = obj
                    .resolve_property(&"calendar".into(), realm)?
                    .and_then(|v| v.to_string(realm).ok());

                let calendar = calendar
                    .as_deref()
                    .map(Calendar::from_str)
                    .transpose()
                    .map_err(Error::from_temporal)?
                    .unwrap_or_default();

                return temporal_rs::PlainMonthDay::new_with_overflow(
                    month, day, calendar, overflow, year,
                )
                .map_err(Error::from_temporal);
            }

            Err(Error::ty(
                "Expected PlainMonthDay object with year, month, and day properties",
            ))
        }
        Value::String(s) => {
            let month_day = temporal_rs::PlainMonthDay::from_str(&s.to_string())
                .map_err(Error::from_temporal)?;
            Ok(month_day)
        }
        _ => Err(Error::ty("Expected PlainMonthDay or String")),
    }
}

pub fn value_to_partial_date(value: &ObjectHandle, realm: &mut Realm) -> Res<PartialDate> {
    let mut partial_date = PartialDate::new();

    if let Some(era) = value.get_opt("era", realm)? {
        let era = era.to_string(realm)?;

        let str = FromStr::from_str(&era)?;

        partial_date = partial_date.with_era(Some(str));
    }

    if let Some(era_year) = value.get_opt("eraYear", realm)? {
        let era_year = era_year.to_number(realm)?;

        partial_date = partial_date.with_era_year(Some(era_year as i32));
    }

    if let Some(year) = value.get_opt("year", realm)? {
        let year = year.to_number(realm)?;

        partial_date = partial_date.with_year(Some(year as i32));
    }

    Ok(partial_date)
}

impl PrettyObjectOverride for PlainMonthDay {
    fn pretty_inline(&self, obj: &Object, not: &mut Vec<usize>) -> Option<String> {
        let mut s = self.month_day.to_string();

        fmt_properties_to(obj, &mut s, not);

        Some(s)
    }
}
