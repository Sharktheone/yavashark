use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::panic::resume_unwind;
use std::str::FromStr;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct PlainTime {
    time: temporal_rs::PlainTime,
}

impl PlainTime {
    pub fn new(time: temporal_rs::PlainTime, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainTime {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_time.clone().into()),
            }),
            time,
        }
    }
}

#[props]
impl PlainTime {
    #[constructor]
    pub fn construct(
        #[realm] realm: &Realm,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> Res<ObjectHandle> {
        let time =
            temporal_rs::PlainTime::new(hour, minute, second, millisecond, microsecond, nanosecond)
                .map_err(Error::from_temporal)?;

        Ok(Self::new(time, realm).into_object())
    }
    
    pub fn compare(
        left: Value,
        right: Value,
        #[realm] realm: &mut Realm,
    ) -> Res<i8> {
        let left_time = value_to_plain_time(left, realm)?;
        let right_time = value_to_plain_time(right, realm)?;

        let result = left_time.cmp(&right_time);

        Ok(result as i8)
    }
    
    pub fn from(
        value: Value,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let time = value_to_plain_time(value, realm)?;

        Ok(Self::new(time, realm).into_object())
    }
}

pub fn value_to_plain_time(info: Value, realm: &mut Realm) -> Res<temporal_rs::PlainTime> {
    match info {
        Value::String(str) => {
            let time = temporal_rs::PlainTime::from_str(&str).map_err(Error::from_temporal)?;

            Ok(time)
        }
        Value::Object(obj) => {
            if let Some(plain_time) = obj.downcast::<PlainTime>() {
                return Ok(plain_time.time);
            }

            if let Some(plain_date_time) = obj.downcast::<PlainDateTime>() {
                return plain_date_time
                    .date
                    .to_plain_time()
                    .map_err(Error::from_temporal);
            }

            let hour = obj.get("hour", realm).and_then(|v| v.to_number(realm))? as u8;

            let minute = obj.get("minute", realm).and_then(|v| v.to_number(realm))? as u8;

            let second = obj.get("second", realm).and_then(|v| v.to_number(realm))? as u8;

            let millisecond = obj
                .get("millisecond", realm)
                .and_then(|v| v.to_number(realm))? as u16;

            let microsecond = obj
                .get("microsecond", realm)
                .and_then(|v| v.to_number(realm))? as u16;

            let nanosecond = obj
                .get("nanosecond", realm)
                .and_then(|v| v.to_number(realm))? as u16;

            temporal_rs::PlainTime::new(hour, minute, second, millisecond, microsecond, nanosecond)
                .map_err(Error::from_temporal)
        }
        
        _ => Err(Error::ty(
            "Expected a string or an object representing PlainTime",
        )),
    }
}
