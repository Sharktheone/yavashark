use crate::builtins::temporal::duration::Duration;
use crate::conversion::FromValueOutput;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cell::{Cell, RefCell};
use std::str::FromStr;
use std::time::UNIX_EPOCH;
use temporal_rs::options::{DifferenceSettings, ToStringRoundingOptions};
use temporal_rs::unix_time::EpochNanoseconds;
use yavashark_macro::{object, props};
use yavashark_value::ops::BigIntOrNumber;
use yavashark_value::Obj;
use crate::builtins::temporal::utils::string_rounding_mode_opts;

#[object]
#[derive(Debug)]
pub struct Instant {
    stamp: Cell<temporal_rs::Instant>,
}

impl Instant {
    pub fn from_stamp(stamp: temporal_rs::Instant, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableInstant {
                object: MutObject::with_proto(realm.intrinsics.temporal_instant.clone().into()),
            }),
            stamp: Cell::new(stamp),
        }
    }

    #[allow(unused)]
    pub fn from(value: Value, realm: &mut Realm) -> Res<Self> {
        if let Value::Object(obj) = &value {
            let instant = <&Self>::from_value_out(value)?;

            return Ok(Self::from_stamp(instant.stamp.get(), realm));
        }

        let str = value.to_string(realm)?;

        temporal_rs::Instant::from_str(str.as_str())
            .map(|dt| Self::from_stamp(dt, realm))
            .map_err(|_| Error::ty("Invalid date"))
    }

    pub fn now(realm: &Realm) -> Res<ObjectHandle> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::new("System time before UNIX epoch"))?;

        let i =
            temporal_rs::Instant::try_new(now.as_nanos() as i128).map_err(Error::from_temporal)?;

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
    fn compare(left: &Instant, right: &Instant) -> i8 {
        left.stamp.cmp(&right.stamp) as i8
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
            BigIntOrNumber::Number(num) => temporal_rs::Instant::from_epoch_milliseconds(num as i64),
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

        let i = self
            .stamp
            .get()
            .add(other.dur.get())
            .map_err(Error::from_temporal)?;

        Ok(Self::from_stamp(i, realm).into_object())
    }

    fn equals(&self, other: &Self) -> bool {
        self.stamp == other.stamp
    }

    fn round(&self, _opts: Value, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        Ok(Self::from_stamp(self.stamp.get(), realm).into_object())
    }

    fn since(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let res = self
            .stamp
            .get()
            .since(&other.stamp.get(), DifferenceSettings::default())
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, res).into_object())
    }

    pub fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Duration::from_value_ref(other, realm)?;

        let i = self
            .stamp
            .get()
            .subtract(other.dur.get())
            .map_err(Error::from_temporal)?;

        Ok(Self::from_stamp(i, realm).into_object())
    }

    #[prop("toJSON")]
    fn to_json(&self, #[realm] realm: &Realm) -> Res<String> {
        self.stamp
            .get()
            .to_ixdtf_string_with_provider(None, ToStringRoundingOptions::default(), &realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    #[prop("toString")]
    fn to_string_js(&self, opts: Option<ObjectHandle>, #[realm] realm: &mut Realm) -> Res<String> {
        let opts = string_rounding_mode_opts(opts, realm)?;
        
        self.stamp
            .get()
            .to_ixdtf_string_with_provider(None, opts, &realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    pub fn until(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let dur = other
            .stamp
            .get()
            .until(&self.stamp.get(), DifferenceSettings::default())
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
        BigInt::from(self.stamp.get().epoch_nanoseconds().as_i128())
    }

    #[get("epochMilliseconds")]
    fn epoch_milliseconds(&self) -> i64 {
        self.stamp.get().epoch_milliseconds()
    }
}
