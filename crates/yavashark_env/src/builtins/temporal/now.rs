use crate::builtins::temporal::instant::Instant;
use crate::builtins::temporal::plain_date::PlainDate;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::plain_time::PlainTime;
use crate::builtins::temporal::zoned_date_time::ZonedDateTime;
use crate::{Error, Realm, Res, Symbol};
use temporal_rs::{Temporal, TimeZone};
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Now {}

#[props(intrinsic_name = temporal_now, to_string_tag = "Temporal.Now")]
impl Now {
    #[prop(Symbol::TO_STRING_TAG)]
    #[configurable]
    const TO_STRING_TAG: &'static str = "Temporal.Now";

    fn instant(realm: &mut Realm) -> Res<Instant> {
        Instant::now_obj(realm)
    }

    #[prop("plainDateISO")]
    fn plain_date_iso(realm: &mut Realm, tz: Option<TimeZone>) -> Res<PlainDate> {
        PlainDate::now_obj(realm, tz)
    }

    #[prop("plainDateTimeISO")]
    fn plain_date_time_iso(realm: &mut Realm, tz: Option<TimeZone>) -> Res<PlainDateTime> {
        PlainDateTime::now_obj(realm, tz)
    }

    #[prop("plainTimeISO")]
    fn plain_time_iso(realm: &mut Realm, tz: Option<TimeZone>) -> Res<PlainTime> {
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
    fn zoned_date_time_iso(realm: &mut Realm, tz: Option<TimeZone>) -> Res<ZonedDateTime> {
        ZonedDateTime::now_obj(realm, tz)
    }
}
