use crate::builtins::temporal::instant::Instant;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::plain_time::PlainTime;
use crate::builtins::temporal::zoned_date_time::ZonedDateTime;
use crate::{Error, ObjectHandle, Realm, Res};
use std::time::UNIX_EPOCH;
use temporal_rs::now::NowBuilder;
use temporal_rs::unix_time::EpochNanoseconds;
use temporal_rs::{now, TimeZone};
use yavashark_macro::{object, props};
use yavashark_string::YSString;

#[object]
#[derive(Debug)]
pub struct Now {}

impl Now {
    pub fn get_now() -> Res<now::Now> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::new("System time before UNIX epoch"))?;

        let nanos = EpochNanoseconds::from(now.as_nanos() as i128);

        let tz = iana_time_zone::get_timezone().map_err(|e| Error::new_error(e.to_string()))?;

        let tz =
            TimeZone::try_from_identifier_str(&tz).map_err(|e| Error::new_error(e.to_string()))?;

        Ok(NowBuilder::default()
            .with_system_nanoseconds(nanos)
            .with_system_zone(tz)
            .build())
    }
}

#[props]
impl Now {
    fn instant(realm: &Realm) -> Res<ObjectHandle> {
        Instant::now_obj(realm)
    }

    #[prop("plainDateISO")]
    fn plain_date_iso(realm: &Realm, tz: Option<YSString>) -> Res<ObjectHandle> {
        let tz = tz
            .as_deref()
            .map(TimeZone::try_from_str)
            .transpose()
            .map_err(Error::from_temporal)?;

        PlainDate::now_obj(realm, tz)
    }

    #[prop("plainDateTimeISO")]
    fn plain_date_time_iso(realm: &Realm, tz: Option<YSString>) -> Res<ObjectHandle> {
        let tz = tz
            .as_deref()
            .map(TimeZone::try_from_str)
            .transpose()
            .map_err(Error::from_temporal)?;

        PlainDateTime::now_obj(realm, tz)
    }

    #[prop("plainTimeISO")]
    fn plain_time_iso(realm: &Realm, tz: Option<YSString>) -> Res<ObjectHandle> {
        let tz = tz
            .as_deref()
            .map(TimeZone::try_from_str)
            .transpose()
            .map_err(Error::from_temporal)?;

        PlainTime::now_obj(realm, tz)
    }

    #[prop("timeZoneId")]
    fn time_zone_id() -> Res<String> {
        let now = Self::get_now()?;

        Ok(now.time_zone().identifier())
    }

    #[prop("zonedDateTimeISO")]
    fn zoned_date_time_iso(realm: &Realm, tz: Option<YSString>) -> Res<ObjectHandle> {
        let tz = tz
            .as_deref()
            .map(TimeZone::try_from_str)
            .transpose()
            .map_err(Error::from_temporal)?;

        ZonedDateTime::now_obj(realm, tz)
    }
}
