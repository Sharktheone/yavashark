use crate::builtins::temporal::duration::Duration;
use crate::builtins::temporal::now::Now;
use crate::builtins::temporal::utils::{
    difference_settings, rounding_options, string_rounding_mode_opts,
};
use crate::conversion::FromValueOutput;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::options::{DifferenceSettings, ToStringRoundingOptions};
use temporal_rs::unix_time::EpochNanoseconds;
use yavashark_macro::{object, props};
use yavashark_value::ops::BigIntOrNumber;
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Instant {
    stamp: temporal_rs::Instant,
}

impl Instant {
    pub fn from_stamp(stamp: temporal_rs::Instant, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableInstant {
                object: MutObject::with_proto(realm.intrinsics.temporal_instant.clone().into()),
            }),
            stamp,
        }
    }

    #[allow(unused)]
    pub fn from(value: Value, realm: &mut Realm) -> Res<Self> {
        if let Value::Object(obj) = &value {
            let instant = <&Self>::from_value_out(value)?;

            return Ok(Self::from_stamp(instant.stamp, realm));
        }

        let str = value.to_string(realm)?;

        temporal_rs::Instant::from_str(str.as_str())
            .map(|dt| Self::from_stamp(dt, realm))
            .map_err(Error::from_temporal)
    }

    pub fn now() -> Res<temporal_rs::Instant> {
        Now::get_now()?.instant().map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &Realm) -> Res<ObjectHandle> {
        let i = Self::now()?;

        Ok(Self::from_stamp(i, realm).into_object())
    }
}

#[props]
impl Instant {
    #[constructor]
    fn construct(epoch: &BigInt, #[realm] realm: &Realm) -> Res<ObjectHandle> {
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
        Self::from(info, realm).map(Obj::into_object)
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

        Ok(Self::from_stamp(stamp, realm).into_object())
    }

    #[prop("fromEpochNanoseconds")]
    fn from_epoch_nanoseconds(epoch: &BigInt, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let ns = epoch.to_i128().ok_or(Error::range("epoch out of range"))?;

        let i = temporal_rs::Instant::from(
            EpochNanoseconds::try_from(ns).map_err(Error::from_temporal)?,
        );

        Ok(Self::from_stamp(i, realm).into_object())
    }

    fn add(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Duration::from_value_ref(other, realm)?;

        let i = self.stamp.add(other.dur).map_err(Error::from_temporal)?;

        Ok(Self::from_stamp(i, realm).into_object())
    }

    fn equals(&self, other: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other = value_to_instant(other, realm)?;

        Ok(self.stamp == other)
    }

    fn round(&self, opts: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let (opts, _) = rounding_options(opts, realm)?;

        let stamp = self.stamp.round(opts).map_err(Error::from_temporal)?;

        Ok(Self::from_stamp(stamp, realm).into_object())
    }

    fn since(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
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

        Ok(Duration::with_duration(realm, res).into_object())
    }

    pub fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Duration::from_value_ref(other, realm)?;

        let i = self
            .stamp
            .subtract(other.dur)
            .map_err(Error::from_temporal)?;

        Ok(Self::from_stamp(i, realm).into_object())
    }

    #[prop("toJSON")]
    fn to_json(&self, #[realm] realm: &Realm) -> Res<String> {
        self.stamp
            .to_ixdtf_string_with_provider(
                None,
                ToStringRoundingOptions::default(),
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)
    }

    #[prop("toString")]
    fn to_string_js(&self, opts: Option<ObjectHandle>, #[realm] realm: &mut Realm) -> Res<String> {
        let opts = string_rounding_mode_opts(opts, realm)?;

        self.stamp
            .to_ixdtf_string_with_provider(None, opts, &realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[prop("toLocaleString")]
    fn to_locale_string(&self, #[realm] realm: &Realm) -> Res<String> {
        self.stamp
            .to_ixdtf_string_with_provider(
                None,
                ToStringRoundingOptions::default(),
                &realm.env.tz_provider,
            )
            .map_err(Error::from_temporal)
    }


    pub fn until(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
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

        Ok(Duration::with_duration(realm, dur).into_object())
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
            if let Some(other_instant) = obj.downcast::<Instant>() {
                Ok(other_instant.stamp)
            } else {
                if obj.eq(&realm.intrinsics.temporal_instant) {
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
