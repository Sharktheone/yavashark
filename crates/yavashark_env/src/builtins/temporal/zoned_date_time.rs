use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::{Calendar, TimeZone};
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;
use yavashark_value::ops::BigIntOrNumber;

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
}

#[props]
impl ZonedDateTime {
    #[constructor]
    pub fn construct(
        ns: &BigIntOrNumber,
        tz: &str,
        calendar: Option<YSString>,
        realm: &Realm
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
    
    
}