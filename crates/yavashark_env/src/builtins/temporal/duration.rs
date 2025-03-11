use std::cell::RefCell;
use crate::conversion::FromValueOutput;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Duration {
    #[mutable]
    duration: std::time::Duration,
}

impl Duration {
    #[allow(unused)]
    fn new(realm: &Realm) -> Self {
        Self::with_duration(realm, std::time::Duration::ZERO)
    }

    fn with_duration(realm: &Realm, duration: std::time::Duration) -> Self {
        Self {
            inner: RefCell::new(MutableDuration {
                object: MutObject::with_proto(realm.intrinsics.temporal_duration.clone().into()),
                duration,
            }),
        }
    }

    fn from_value(info: Value, realm: &mut Realm) -> Res<Self> {
        if let Ok(this) = <&Self>::from_value_out(info.copy()) {
            let inner = this.inner.borrow();
            
            return Ok(Self::with_duration(realm, inner.duration));
        }

        if let Value::Object(obj) = info {
            let mut extract = || Result::<Option<u64>, Error>::Ok(obj.resolve_property(&"years".into(), realm)?.map(|v| v.to_number(realm).unwrap_or(0.0) as u64));
            
            let years = extract()?;
            let months = extract()?;
            let days = extract()?;
            let hours = extract()?;
            let minutes = extract()?;
            let seconds = extract()?;
            let milliseconds = extract()?;
            let microseconds = extract()?;
            let nanoseconds = extract()?;
            
            return Ok(Self::constructor(years, months, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, realm));
        }
        
        
        Err(Error::ty("Invalid value for Duration"))
    }  
    
    #[allow(clippy::too_many_arguments)]
    fn constructor(years: Option<u64>, months: Option<u64>, days: Option<u64>, hours: Option<u64>, minutes: Option<u64>, seconds: Option<u64>, milliseconds: Option<u64>, microseconds: Option<u64>, nanoseconds: Option<u64>, realm: &Realm) -> Self {
        let mut dur = std::time::Duration::ZERO;

        if let Some(years) = years {
            dur += std::time::Duration::from_secs(years * 365 * 24 * 60 * 60);
        }

        if let Some(months) = months {
            dur += std::time::Duration::from_secs(months * 30 * 24 * 60 * 60);
        }

        if let Some(days) = days {
            dur += std::time::Duration::from_secs(days * 24 * 60 * 60);
        }

        if let Some(hours) = hours {
            dur += std::time::Duration::from_secs(hours * 60 * 60);
        }

        if let Some(minutes) = minutes {
            dur += std::time::Duration::from_secs(minutes * 60);
        }

        if let Some(seconds) = seconds {
            dur += std::time::Duration::from_secs(seconds);
        }

        if let Some(milliseconds) = milliseconds {
            dur += std::time::Duration::from_millis(milliseconds);
        }

        if let Some(microseconds) = microseconds {
            dur += std::time::Duration::from_micros(microseconds);
        }

        if let Some(nanoseconds) = nanoseconds {
            dur += std::time::Duration::from_nanos(nanoseconds);
        }


        Self::with_duration(realm, dur)
    }
}


#[props]
impl Duration {
    #[constructor]
    #[allow(clippy::too_many_arguments)]
    fn construct(years: Option<u64>, months: Option<u64>, days: Option<u64>, hours: Option<u64>, minutes: Option<u64>, seconds: Option<u64>, milliseconds: Option<u64>, microseconds: Option<u64>, nanoseconds: Option<u64>, #[realm] realm: &Realm) -> ObjectHandle {
        Self::constructor(years, months, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, realm).into_object()
    }

    fn from(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Self::from_value(info, realm)?.into_object())
    }
}
