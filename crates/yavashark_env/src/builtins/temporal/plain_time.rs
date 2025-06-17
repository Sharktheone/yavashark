use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res};

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
        let time = temporal_rs::PlainTime::new(
            hour, minute, second, millisecond, microsecond, nanosecond,
        ).map_err(Error::from_temporal)?;
        
        Ok(Self::new(time, realm).into_object())
    }
}
