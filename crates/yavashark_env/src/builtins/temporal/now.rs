use crate::builtins::temporal::instant::Instant;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::plain_time::PlainTime;
use crate::builtins::temporal::zoned_date_time::ZonedDateTime;
use crate::{Error, ObjectHandle, Realm, Res};
use temporal_rs::{TimeZone, Temporal};
use yavashark_macro::{object, props};
use yavashark_string::YSString;

#[object]
#[derive(Debug)]
pub struct Now {}

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
        Temporal::now()
            .time_zone()
            .map_err(Error::from_temporal)?
            .identifier()
            .map_err(Error::from_temporal)
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
