use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::utils::{
    display_calendar, overflow_options, overflow_options_opt, value_to_calendar_fields,
};
use crate::native_obj::NativeObject;
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::{Obj, Object};
use crate::{Error, ObjectHandle, Realm, Res, Value};
use icu::calendar::AnyCalendarKind;
use std::str::FromStr;
use temporal_rs::options::Overflow;
use temporal_rs::partial::PartialDate;
use temporal_rs::Calendar;
use yavashark_macro::props;
use yavashark_string::YSString;

#[derive(Debug)]
pub struct PlainMonthDay {
    pub month_day: temporal_rs::PlainMonthDay,
}

impl PlainMonthDay {
    pub fn new(month_day: temporal_rs::PlainMonthDay, realm: &mut crate::Realm) -> Res<NativeObject<Self>> {
        NativeObject::new(Self { month_day }, realm)
    }
}

#[props(intrinsic_name = temporal_plain_month_day, to_string_tag = "Temporal.PlainMonthDay")]
impl PlainMonthDay {
    #[constructor]
    pub fn construct(
        month: u8,
        day: u8,
        calendar: Option<Calendar>,
        ref_year: Option<i32>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let calendar = calendar.unwrap_or_default();

        let month_day = temporal_rs::PlainMonthDay::new_with_overflow(
            month,
            day,
            calendar,
            Overflow::Constrain,
            ref_year,
        )
        .map_err(Error::from_temporal)?;

        Ok(Self::new(month_day, realm)?.into_object())
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

        Ok(Self::new(month_day, realm)?.into_object())
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

        Ok(PlainDate::new(plain_date, realm)?.into_object())
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
        let fields = value_to_calendar_fields(other, false, true, realm)?;

        let month_day = self
            .month_day
            .with(fields, overflow)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(month_day, realm)?.into_object())
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
            if let Some(plain_month_day) = obj.downcast::<NativeObject<PlainMonthDay>>() {
                return Ok(plain_month_day.month_day.clone());
            }

            let overflow = overflow.unwrap_or(Overflow::Constrain);

            if (obj.contains_key("month".into(), realm)?
                || obj.contains_key("monthCode".into(), realm)?)
                && obj.contains_key("day".into(), realm)?
            {
                let year = obj
                    .resolve_property("year", realm)?
                    .map(|v| v.to_number(realm).map(|v| v as i32))
                    .transpose()?;

                let month = obj
                    .resolve_property("month", realm)?
                    .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;

                let month = if month == 0 {
                    obj.resolve_property("monthCode", realm)?
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
                    .resolve_property("day", realm)?
                    .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;

                let calendar = obj
                    .extract_opt::<Calendar>("calendar", realm)?
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
    let fields = value_to_calendar_fields(value, false, false, realm)?;

    let mut partial_date = PartialDate {
        calendar_fields: fields,
        calendar: Calendar::new(AnyCalendarKind::Iso),
    };

    if let Some(cal) = value.extract_opt::<Calendar>("calendar", realm)? {
        partial_date = partial_date.with_calendar(cal);
    }

    Ok(partial_date)
}

impl PrettyObjectOverride for PlainMonthDay {
    fn pretty_inline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self.month_day.to_string();

        fmt_properties_to(obj, &mut s, not, realm);

        Some(s)
    }
}
