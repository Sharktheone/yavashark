use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::{Calendar, MonthCode, TimeZone, TinyAsciiStr, UtcOffset};
use temporal_rs::options::OffsetDisambiguation;
use temporal_rs::partial::PartialZonedDateTime;
use yavashark_macro::{object, props};
use yavashark_string::YSString;
use yavashark_value::Obj;
use yavashark_value::ops::BigIntOrNumber;
use crate::builtins::temporal::utils::{disambiguation_opt, offset_disambiguation_opt, overflow_options_opt};

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


pub fn value_to_zoned_date_time(
    value: &Value,
    options: Option<ObjectHandle>,
    realm: &mut Realm,
) -> Res<temporal_rs::ZonedDateTime> {
    
    let disambiguation = disambiguation_opt(options.as_ref(), realm)?;
    let offset_disambiguation = offset_disambiguation_opt(options.as_ref(), realm)?;
        
        
    
    
    Ok(match value {
        Value::Object(obj) => {
            if let Some(zdt) = obj.downcast::<ZonedDateTime>() {
                return Ok(zdt.date.clone())
            } 
            
            let overflow = overflow_options_opt(options.as_ref(), realm)?;
            
            let partial = partial_zoned_date_time(obj, realm)?;
            
            temporal_rs::ZonedDateTime::from_partial_with_provider(
                partial,
                overflow,
                disambiguation,
                offset_disambiguation,
                &realm.env.tz_provider
            )
            .map_err(Error::from_temporal)?
            
            
        }
        Value::String(str) => {
            let disambiguation = disambiguation.unwrap_or_default();
            let offset_disambiguation = offset_disambiguation.unwrap_or(OffsetDisambiguation::Reject);
                
            
            
            temporal_rs::ZonedDateTime::from_str_with_provider(
                &str,
                disambiguation,
                offset_disambiguation,
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)?
        }
        _ => return Err(Error::ty("Expected a ZonedDateTime object")),
    })
}

pub fn partial_zoned_date_time(
    obj: &ObjectHandle,
    realm: &mut Realm,
) -> Res<PartialZonedDateTime> {
    let mut partial = PartialZonedDateTime::new();
    
    if let Some(calendar) = obj.get_opt("calendar", realm)? {
        let calendar = calendar.to_string(realm)?;
        let calendar = Calendar::from_str(&calendar)
            .map_err(Error::from_temporal)?;
        
        partial.date = partial.date.with_calendar(calendar);
    }
    
    let mut has_year = false;
    
    if let Some(ns) = obj.get_opt("era", realm)? {
        let era = ns.to_string(realm)?;
        let era = TinyAsciiStr::try_from_str(&era)
            .map_err(|_| Error::ty("Invalid era string"))?;
        
        partial.date = partial.date.with_era(Some(era));
        has_year = true;
    }
    
    
    if let Some(era_year) = obj.get_opt("eraYear", realm)? {
        let era_year = era_year.to_number(realm)?;
        partial.date = partial.date.with_era_year(Some(era_year as i32));
        has_year = true;
    }
    
    if let Some(year) = obj.get_opt("year", realm)? {
        let year = year.to_number(realm)?;
        partial.date = partial.date.with_year(Some(year as i32));
        has_year = true;
    }
    
    if !has_year {
        return Err(Error::ty("Expected year, era, or eraYear to be defined"));
    }
    
    let mut has_month = false;
    
    if let Some(month) = obj.get_opt("month", realm)? {
        let month = month.to_number(realm)?;
        partial.date = partial.date.with_month(Some(month as u8));
        has_month = true;
    }
    
    if let Some(month_code) = obj.get_opt("monthCode", realm)? {
        let month_code = month_code.to_string(realm)?;
        
        let month_code = MonthCode::from_str(&month_code)
            .map_err(|_| Error::ty("Invalid month code"))?;
        
        partial.date = partial.date.with_month_code(Some(month_code));
        has_month = true;
    }
    
    if !has_month {
        return Err(Error::ty("Expected month or monthCode to be defined"));
    }
    
    if let Some(day) = obj.get_opt("day", realm)? {
        let day = day.to_number(realm)?;
        partial.date = partial.date.with_day(Some(day as u8));
    } else {
        return Err(Error::ty("Expected day to be defined"));
    }
    
    if let Some(hour) = obj.get_opt("hour", realm)? {
        let hour = hour.to_number(realm)?;
        partial.time = partial.time.with_hour(Some(hour as u8));
    }
    
    if let Some(minute) = obj.get_opt("minute", realm)? {
        let minute = minute.to_number(realm)?;
        partial.time = partial.time.with_minute(Some(minute as u8));
    }
    
    if let Some(second) = obj.get_opt("second", realm)? {
        let second = second.to_number(realm)?;
        partial.time = partial.time.with_second(Some(second as u8));
    }
    
    if let Some(millisecond) = obj.get_opt("millisecond", realm)? {
        let millisecond = millisecond.to_number(realm)?;
        partial.time = partial.time.with_millisecond(Some(millisecond as u16));
    }
    
    if let Some(microsecond) = obj.get_opt("microsecond", realm)? {
        let microsecond = microsecond.to_number(realm)?;
        partial.time = partial.time.with_microsecond(Some(microsecond as u16));
    }
    
    if let Some(nanosecond) = obj.get_opt("nanosecond", realm)? {
        let nanosecond = nanosecond.to_number(realm)?;
        partial.time = partial.time.with_nanosecond(Some(nanosecond as u16));
    }
    
    if let Some(time_zone) = obj.get_opt("timeZone", realm)? {
        let time_zone = time_zone.to_string(realm)?;
        let time_zone = TimeZone::try_from_str(&time_zone)
            .map_err(Error::from_temporal)?;
        
        partial.timezone = Some(time_zone);
    } else {
        return Err(Error::ty("Expected timeZone to be defined"));
    }
    
    if let Some(offset) = obj.get_opt("offset", realm)? {
        let offset = offset.to_string(realm)?;
        let offset = UtcOffset::from_str(&offset)
            .map_err(Error::from_temporal)?;
        
        partial.offset = Some(offset);
    } else {
        return Err(Error::ty("Expected offset to be defined"));
    }

    
    
    Ok(partial)
}



