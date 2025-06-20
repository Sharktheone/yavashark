use std::cell::RefCell;
use temporal_rs::Calendar;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res};
use crate::builtins::temporal::utils::calendar_opt;

#[object]
#[derive(Debug)]
pub struct PlainYearMonth {
    year_month: temporal_rs::PlainYearMonth,
}

impl PlainYearMonth {
    pub fn new(year_month: temporal_rs::PlainYearMonth, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainYearMonth {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_year_month.clone().into()),
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
        calendar: Option<String>,
        reference_day: Option<u8>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let calendar = calendar_opt(calendar)?;

        let year_month = temporal_rs::PlainYearMonth::new(year, month, reference_day, calendar)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(year_month, realm).into_object())
    }
}
