use crate::builtins::temporal::utils::{calendar_opt, overflow_options};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;

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
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let calendar = calendar_opt(calendar.as_deref())?;

        let year_month = temporal_rs::PlainYearMonth::new(year, month, reference_day, calendar)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(year_month, realm).into_object())
    }
    
    pub fn compare(
        left: Value,
        right: Value,
        #[realm] realm: &mut Realm,
    ) -> Res<i8> {
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
