use crate::builtins::temporal::duration::Duration;
use crate::conversion::FromValueOutput;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cell::{Cell, RefCell};
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Instant {
    stamp: Cell<DateTime<Utc>>,
}

impl Instant {
    pub fn new(stamp: &BigInt, realm: &Realm) -> Res<Self> {
        let ns = stamp.to_i64().ok_or(Error::range("stamp out of range"))?;
        let dt = DateTime::from_timestamp_nanos(ns);

        Ok(Self {
            inner: RefCell::new(MutableInstant {
                object: MutObject::with_proto(realm.intrinsics.temporal_instant.clone().into()),
            }),
            stamp: Cell::new(dt),
        })
    }

    pub fn from_stamp(stamp: DateTime<Utc>, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableInstant {
                object: MutObject::with_proto(realm.intrinsics.temporal_instant.clone().into()),
            }),
            stamp: Cell::new(stamp),
        }
    }

    #[allow(unused)]
    pub fn from(value: Value, realm: &Realm) -> Res<Self> {
        if let Value::String(str) = &value {
            return DateTime::parse_from_rfc3339(str)
                .map(|dt| Self {
                    //TODO: this needs to be RFC 9557
                    inner: RefCell::new(MutableInstant {
                        object: MutObject::with_proto(
                            realm.intrinsics.temporal_instant.clone().into(),
                        ),
                    }),
                    stamp: Cell::new(dt.into()),
                })
                .map_err(|_| Error::ty("Invalid date"));
        }

        let instant = <&Self>::from_value_out(value)?;

        Ok(Self::from_stamp(instant.stamp.get(), realm))
    }

    pub fn now(realm: &Realm) -> Res<ObjectHandle> {
        Ok(Self::from_stamp(Utc::now(), realm).into_object())
    }
}

#[props]
impl Instant {
    #[constructor]
    fn construct(epoch: &BigInt, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        Ok(Self::new(epoch, realm)?.into_object())
    }

    #[allow(clippy::use_self)]
    fn compare(left: &Instant, right: &Instant) -> i8 {
        left.stamp.cmp(&right.stamp) as i8
    }

    #[prop("from")]
    fn from_js(info: Value, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        if let Value::String(str) = &info {
            return Ok(DateTime::parse_from_rfc3339(str)
                .map(|dt| Self {
                    //TODO: this needs to be RFC 9557
                    inner: RefCell::new(MutableInstant {
                        object: MutObject::with_proto(
                            realm.intrinsics.temporal_instant.clone().into(),
                        ),
                    }),
                    stamp: Cell::new(dt.into()),
                })
                .map_err(|_| Error::ty("Invalid date"))?
                .into_object());
        }

        let instant = <&Self>::from_value_out(info)?;

        Ok(Self::from_stamp(instant.stamp.get(), realm).into_object())
    }

    #[prop("fromEpochMilliseconds")]
    fn from_epoch_milliseconds(epoch: &BigInt, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let ns = epoch.to_i64().ok_or(Error::range("epoch out of range"))?;
        let dt = DateTime::from_timestamp_millis(ns).ok_or(Error::range("epoch out of range"))?;

        Ok(Self::from_stamp(dt, realm).into_object())
    }

    #[prop("fromEpochNanoseconds")]
    fn from_epoch_nanoseconds(epoch: &BigInt, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let ns = epoch.to_i64().ok_or(Error::range("epoch out of range"))?;
        let dt = DateTime::from_timestamp_nanos(ns);

        Ok(Self::from_stamp(dt, realm).into_object())
    }

    fn add(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Duration::from_value_ref(other, realm)?;

        let dt = if other.is_negative() {
            self.stamp.get() - other.to_duration()
        } else {
            self.stamp.get() + other.to_duration()
        };

        Ok(Self::from_stamp(dt, realm).into_object())
    }

    fn equals(&self, other: &Self) -> bool {
        self.stamp == other.stamp
    }

    fn round(&self, _opts: Value, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        Ok(Self::from_stamp(self.stamp.get(), realm).into_object())
    }

    fn since(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let dur = self.stamp.get().signed_duration_since(other.stamp.get());

        Ok(Duration::from_duration(dur, realm)?.into_object())
    }

    pub fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Duration::from_value_ref(other, realm)?;

        let dt = if other.is_negative() {
            self.stamp.get() + other.to_duration()
        } else {
            self.stamp.get() - other.to_duration()
        };

        Ok(Self::from_stamp(dt, realm).into_object())
    }

    #[prop("toJSON")]
    fn to_json(&self) -> String {
        self.stamp.get().to_rfc3339() //TODO: this needs to be RFC 9557
    }

    #[prop("toString")]
    fn to_string_js(&self) -> String {
        self.stamp.get().to_rfc3339() //TODO: this needs to be RFC 9557
    }

    pub fn until(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let dur = other.stamp.get().signed_duration_since(self.stamp.get());

        Ok(Duration::from_duration(dur, realm)?.into_object())
    }

    #[prop("valueOf")]
    #[nonstatic]
    fn value_of() -> Res {
        Err(Error::ty("Called valueOf on a Temporal.Instant object"))
    }

    #[get("epochNanoseconds")]
    fn epoch_nanoseconds(&self) -> BigInt {
        BigInt::from(self.stamp.get().timestamp_nanos_opt().unwrap_or(0))
    }

    #[get("epochMilliseconds")]
    fn epoch_milliseconds(&self) -> BigInt {
        BigInt::from(self.stamp.get().timestamp_millis())
    }
}
