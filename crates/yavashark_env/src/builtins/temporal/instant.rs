use crate::builtins::temporal::duration::Duration;
use crate::builtins::temporal::utils::{
    difference_settings, rounding_options, string_rounding_mode_opts,
};
use crate::builtins::temporal::zoned_date_time::ZonedDateTime;
use crate::conversion::downcast_obj;
use crate::native_obj::NativeObject;
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::ops::BigIntOrNumber;
use crate::value::{Obj, Object};
use crate::{Error, ObjectHandle, Realm, Res, Value};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::str::FromStr;
use temporal_rs::options::{DifferenceSettings, ToStringRoundingOptions};
use temporal_rs::unix_time::EpochNanoseconds;
use temporal_rs::{Temporal, TimeZone};
use yavashark_macro::props;

#[derive(Debug)]
pub struct Instant {
    pub(crate) stamp: temporal_rs::Instant,
}

impl Instant {
    pub fn from_stamp(stamp: temporal_rs::Instant, realm: &mut Realm) -> Res<NativeObject<Self>> {
        NativeObject::new(Self { stamp }, realm)
    }

    #[allow(unused)]
    pub fn from(value: Value, realm: &mut Realm) -> Res<NativeObject<Self>> {
        if let Value::Object(_) = &value {
            let instant = downcast_obj::<NativeObject<Self>>(value)?;

            return Self::from_stamp(instant.stamp, realm);
        }

        let str = value.to_string(realm)?;

        temporal_rs::Instant::from_str(str.as_str())
            .map_err(Error::from_temporal)
            .and_then(|dt| Self::from_stamp(dt, realm))
    }

    pub fn now() -> Res<temporal_rs::Instant> {
        Temporal::now().instant().map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &mut Realm) -> Res<NativeObject<Self>> {
        let i = Self::now()?;

        Self::from_stamp(i, realm)
    }
}

#[props(intrinsic_name = temporal_instant, to_string_tag = "Temporal.Instant")]
impl Instant {
    #[constructor]
    fn construct(epoch: &BigInt, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        Self::from_epoch_nanoseconds(epoch, realm)
    }

    #[allow(clippy::use_self)]
    fn compare(left: Value, right: Value, #[realm] realm: &mut Realm) -> Res<i8> {
        let left = value_to_instant(left, realm)?;
        let right = value_to_instant(right, realm)?;

        Ok(left.cmp(&right) as i8)
    }

    #[prop("from")]
    fn from_js(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Self::from(info, realm).map(NativeObject::into_object)
    }

    #[prop("fromEpochMilliseconds")]
    fn from_epoch_milliseconds(epoch: &Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let epoch = epoch.to_numeric(realm)?;

        let stamp = match epoch {
            BigIntOrNumber::BigInt(bigint) => temporal_rs::Instant::from_epoch_milliseconds(
                bigint.to_i64().ok_or(Error::range("epoch out of range"))?,
            ),
            BigIntOrNumber::Number(num) => {
                temporal_rs::Instant::from_epoch_milliseconds(num as i64)
            }
        }
        .map_err(Error::from_temporal)?;

        Ok(Self::from_stamp(stamp, realm)?.into_object())
    }

    #[prop("fromEpochNanoseconds")]
    fn from_epoch_nanoseconds(
        epoch: &BigInt,
        #[realm] realm: &mut Realm,
    ) -> Res<NativeObject<Self>> {
        let ns = epoch.to_i128().ok_or(Error::range("epoch out of range"))?;

        let i = temporal_rs::Instant::from(EpochNanoseconds::from(ns));

        Self::from_stamp(i, realm)
    }

    fn add(&self, other: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let other = Duration::from_value_ref(other, realm)?;

        let i = self.stamp.add(&other.dur).map_err(Error::from_temporal)?;

        Self::from_stamp(i, realm)
    }

    fn equals(&self, other: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other = value_to_instant(other, realm)?;

        Ok(self.stamp == other)
    }

    fn round(&self, opts: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let (opts, _) = rounding_options(opts, realm)?;

        let stamp = self.stamp.round(opts).map_err(Error::from_temporal)?;

        Self::from_stamp(stamp, realm)
    }

    fn since(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<NativeObject<Duration>> {
        let other = value_to_instant(other, realm)?;

        let opts = if let Some(opts) = opts {
            difference_settings(opts, realm)?
        } else {
            DifferenceSettings::default()
        };

        let res = self
            .stamp
            .since(&other, opts)
            .map_err(Error::from_temporal)?;

        Duration::with_duration(realm, res)
    }

    pub fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let other = Duration::from_value_ref(other, realm)?;

        let i = self
            .stamp
            .subtract(&other.dur)
            .map_err(Error::from_temporal)?;

        Self::from_stamp(i, realm)
    }

    #[prop("toJSON")]
    fn to_json(&self) -> Res<String> {
        self.stamp
            .to_ixdtf_string(None, ToStringRoundingOptions::default())
            .map_err(Error::from_temporal)
    }

    #[prop("toString")]
    fn to_string_js(&self, opts: Option<ObjectHandle>, #[realm] realm: &mut Realm) -> Res<String> {
        let timezone = if let Some(ref obj) = opts {
            let tz_val = obj.get("timeZone", realm)?;
            if tz_val.is_undefined() {
                None
            } else {
                let tz_str = tz_val.to_string(realm)?;
                Some(TimeZone::try_from_str(&tz_str).map_err(Error::from_temporal)?)
            }
        } else {
            None
        };

        let opts = string_rounding_mode_opts(opts, realm)?;

        self.stamp
            .to_ixdtf_string(timezone, opts)
            .map_err(Error::from_temporal)
    }

    #[prop("toLocaleString")]
    fn to_locale_string(&self) -> Res<String> {
        self.stamp
            .to_ixdtf_string(None, ToStringRoundingOptions::default())
            .map_err(Error::from_temporal)
    }

    #[prop("toZonedDateTimeISO")]
    fn to_zoned_date_time_iso(
        &self,
        time_zone: TimeZone,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let zdt = self
            .stamp
            .to_zoned_date_time_iso(time_zone)
            .map_err(Error::from_temporal)?;

        Ok(ZonedDateTime::new(zdt, realm)?.into_object())
    }

    pub fn until(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<NativeObject<Duration>> {
        let other = value_to_instant(other, realm)?;

        let opts = if let Some(opts) = opts {
            difference_settings(opts, realm)?
        } else {
            DifferenceSettings::default()
        };

        let dur = self
            .stamp
            .until(&other, opts)
            .map_err(Error::from_temporal)?;

        Duration::with_duration(realm, dur)
    }

    #[prop("valueOf")]
    #[nonstatic]
    const fn value_of() -> Res {
        Err(Error::ty("Called valueOf on a Temporal.Instant object"))
    }

    #[get("epochNanoseconds")]
    fn epoch_nanoseconds(&self) -> BigInt {
        BigInt::from(self.stamp.epoch_nanoseconds().as_i128())
    }

    #[get("epochMilliseconds")]
    fn epoch_milliseconds(&self) -> i64 {
        self.stamp.epoch_milliseconds()
    }
}

pub fn value_to_instant(value: Value, realm: &mut Realm) -> Res<temporal_rs::Instant> {
    match value {
        Value::Object(obj) => {
            if let Some(other_instant) = obj.downcast::<NativeObject<Instant>>() {
                Ok(other_instant.stamp)
            } else {
                if obj.eq(realm
                    .intrinsics
                    .clone_public()
                    .temporal_instant
                    .get(realm)?)
                {
                    return Err(Error::ty("Expected a Temporal.Instant object"));
                }

                let str = obj.to_string(realm)?;

                temporal_rs::Instant::from_str(str.as_str()).map_err(Error::from_temporal)
            }
        }
        Value::String(s) => {
            temporal_rs::Instant::from_str(s.as_str()).map_err(Error::from_temporal)
        }
        _ => Err(Error::ty("Expected a Temporal.Instant object")),
    }
}

impl PrettyObjectOverride for Instant {
    fn pretty_inline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self
            .stamp
            .to_ixdtf_string(None, ToStringRoundingOptions::default())
            .ok()?;

        fmt_properties_to(obj, &mut s, not, realm);

        Some(s)
    }
}
