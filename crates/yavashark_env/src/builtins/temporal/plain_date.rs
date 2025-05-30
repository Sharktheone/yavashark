use crate::builtins::temporal::duration::Duration;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use chrono::{Datelike, NaiveDate};
use std::cell::{Cell, RefCell};
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct PlainDate {
    date: Cell<NaiveDate>,
}

#[props]
impl PlainDate {
    #[constructor]
    pub fn construct(
        year: i32,
        month: u32,
        day: u32,
        _calendar: Option<String>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(Error::ty("Invalid date"))?;

        Ok(Self {
            inner: RefCell::new(MutablePlainDate {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_date.clone().into()),
            }),
            date: Cell::new(date),
        }
        .into_object())
    }

    pub fn from(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        if let Value::String(str) = &info {
            return Ok(NaiveDate::parse_from_str(str, "%Y-%m-%d")
                .map(|date| Self {
                    inner: RefCell::new(MutablePlainDate {
                        object: MutObject::with_proto(
                            realm.intrinsics.temporal_plain_date.clone().into(),
                        ),
                    }),
                    date: Cell::new(date),
                })
                .map_err(|_| Error::ty("Invalid date"))?
                .into_object());
        }

        let obj = info.to_object()?;

        if obj.contains_key(&"year".into())?
            || obj.contains_key(&"month".into())?
            || obj.contains_key(&"day".into())?
        {
            let year = obj
                .resolve_property(&"year".into(), realm)?
                .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as i32))?;
            let month = obj
                .resolve_property(&"month".into(), realm)?
                .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u32))?;
            let day = obj
                .resolve_property(&"day".into(), realm)?
                .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u32))?;

            return Ok(Self {
                inner: RefCell::new(MutablePlainDate {
                    object: MutObject::with_proto(
                        realm.intrinsics.temporal_plain_date.clone().into(),
                    ),
                }),
                date: Cell::new(
                    NaiveDate::from_ymd_opt(year, month, day).ok_or(Error::ty("Invalid date"))?,
                ),
            }
            .into_object());
        }

        Err(Error::ty("Invalid date")) //TODO
    }

    #[allow(clippy::use_self)]
    pub fn compare(left: &PlainDate, right: &PlainDate) -> i32 {
        left.date.get().cmp(&right.date.get()) as i32
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.date.get() == other.date.get()
    }

    pub fn since(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let hours = self
            .date
            .get()
            .signed_duration_since(other.date.get())
            .num_hours();

        Ok(Duration::constructor(
            None,
            None,
            None,
            None,
            Some(hours),
            None,
            None,
            None,
            None,
            None,
            realm,
        )?
        .into_object())
    }

    pub fn until(&self, other: &Self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let hours = other
            .date
            .get()
            .signed_duration_since(self.date.get())
            .num_hours();

        Ok(Duration::constructor(
            None,
            None,
            None,
            None,
            Some(hours),
            None,
            None,
            None,
            None,
            None,
            realm,
        )?
        .into_object())
    }

    pub fn add(&self, duration: &Duration, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self.date.get();

        // let dur = chrono::Duration::from_std(duration.to_duration())
        //     .map_err(|_| Error::ty("Invalid duration"))?;
        //
        // let date = if duration.is_negative() {
        //     date.checked_sub_signed(dur)
        //         .ok_or(Error::ty("Invalid date"))?
        // } else {
        //     date.checked_add_signed(dur)
        //         .ok_or(Error::ty("Invalid date"))?
        // };

        Ok(Self {
            inner: RefCell::new(MutablePlainDate {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_date.clone().into()),
            }),
            date: Cell::new(date),
        }
        .into_object())
    }

    pub fn subtract(&self, duration: &Duration, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let date = self.date.get();

        // let dur = chrono::Duration::from_std(duration.to_duration())
        //     .map_err(|_| Error::ty("Invalid duration"))?;
        //
        // let date = if duration.is_negative() {
        //     date.checked_add_signed(dur)
        //         .ok_or(Error::ty("Invalid date"))?
        // } else {
        //     date.checked_sub_signed(dur)
        //         .ok_or(Error::ty("Invalid date"))?
        // };

        Ok(Self {
            inner: RefCell::new(MutablePlainDate {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_date.clone().into()),
            }),
            date: Cell::new(date),
        }
        .into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> String {
        self.date.get().to_string()
    }

    #[prop("valueOf")]
    pub fn value_of() -> Res {
        Err(Error::ty("Called valueOf on a Temporal.PlainDate object"))
    }

    #[get("day")]
    pub fn day(&self) -> u32 {
        self.date.get().day()
    }

    #[get("dayOfWeek")]
    pub fn day_of_week(&self) -> u32 {
        self.date.get().weekday().num_days_from_monday()
    }

    #[get("dayOfYear")]
    pub fn day_of_year(&self) -> u32 {
        self.date.get().ordinal0()
    }

    #[get("daysInMonth")]
    pub fn days_in_month(&self) -> u32 {
        let month = self.date.get().month();
        let year = self.date.get().year();

        if month == 2 {
            if year % 4 == 0 {
                29
            } else {
                28
            }
        } else if month == 4 || month == 6 || month == 9 || month == 11 {
            30
        } else {
            31
        }
    }

    #[get("daysInWeek")]
    #[nonstatic]
    pub const fn days_in_week() -> u32 {
        7
    }

    #[get("daysInYear")]
    pub fn days_in_year(&self) -> u32 {
        if self.date.get().year() % 4 == 0 {
            366
        } else {
            365
        }
    }

    #[get("era")]
    #[nonstatic]
    pub const fn era() -> Value {
        Value::Undefined
    }

    #[get("eraYear")]
    #[nonstatic]
    pub const fn era_year() -> Value {
        Value::Undefined
    }

    #[get("month")]
    pub fn month(&self) -> u32 {
        self.date.get().month()
    }

    #[get("monthCode")]
    pub fn month_code(&self) -> String {
        format!("M{:2}", self.date.get().month())
    }

    #[get("monthsInYear")]
    #[nonstatic]
    pub const fn months_in_year() -> u32 {
        12
    }

    #[get("weekOfYear")]
    pub fn week_of_year(&self) -> u32 {
        self.date.get().iso_week().week()
    }

    #[get("year")]
    pub fn year(&self) -> i32 {
        self.date.get().year()
    }

    #[get("yearOfWeek")]
    pub fn year_of_week(&self) -> i32 {
        // honestly, WHAT THE FUCK?
        self.date.get().year()
    }
}
