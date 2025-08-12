use crate::conversion::FromValueOutput;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use chrono::{DateTime, Datelike, Local, LocalResult, Offset, TimeZone, Timelike};
use std::cell::RefCell;
use std::ops::Rem;
use std::str::FromStr;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, Obj};
use crate::print::PrettyObjectOverride;

#[object]
#[derive(Debug)]
pub struct Date {
    #[mutable]
    date: DateTime<Local>,
}

#[properties_new(constructor(DateConstructor::new))]
impl Date {
    #[prop("getDate")]
    pub fn get_date(&self) -> u32 {
        self.date().day()
    }

    #[prop("getDay")]
    pub fn get_day(&self) -> u32 {
        self.date().weekday().num_days_from_sunday()
    }

    #[prop("getFullYear")]
    pub fn get_full_year(&self) -> i32 {
        self.date().year()
    }

    #[prop("getHours")]
    pub fn get_hours(&self) -> u32 {
        self.date().hour()
    }

    #[prop("getMilliseconds")]
    pub fn get_milliseconds(&self) -> f64 {
        f64::from(self.date().nanosecond()) / 1_000_000.0
    }

    #[prop("getMinutes")]
    pub fn get_minutes(&self) -> u32 {
        self.date().minute()
    }

    #[prop("getMonth")]
    pub fn get_month(&self) -> u32 {
        self.date().month()
    }

    #[prop("getSeconds")]
    pub fn get_seconds(&self) -> u32 {
        self.date().second()
    }

    #[prop("getTime")]
    pub fn get_time(&self) -> f64 {
        self.date().timestamp_millis() as f64
    }

    #[prop("getTimezoneOffset")]
    pub fn get_timezone_offset(&self) -> f64 {
        let secs = self.date().offset().fix().utc_minus_local();

        f64::from(secs) / 60.0
    }

    #[prop("getUTCDate")]
    pub fn get_utc_date(&self) -> u32 {
        self.date().to_utc().day()
    }

    #[prop("getUTCDay")]
    pub fn get_utc_day(&self) -> u32 {
        self.date().to_utc().weekday().num_days_from_sunday()
    }

    #[prop("getUTCFullYear")]
    pub fn get_utc_full_year(&self) -> i32 {
        self.date().to_utc().year()
    }

    #[prop("getUTCHours")]
    pub fn get_utc_hours(&self) -> u32 {
        self.date().to_utc().hour()
    }

    #[prop("getUTCMilliseconds")]
    pub fn get_utc_milliseconds(&self) -> f64 {
        f64::from(self.date().to_utc().nanosecond()) / 1_000_000.0
    }

    #[prop("getUTCMinutes")]
    pub fn get_utc_minutes(&self) -> u32 {
        self.date().to_utc().minute()
    }

    #[prop("getUTCMonth")]
    pub fn get_utc_month(&self) -> u32 {
        self.date().to_utc().month()
    }

    #[prop("getUTCSeconds")]
    pub fn get_utc_seconds(&self) -> u32 {
        self.date().to_utc().second()
    }

    #[prop("getYear")]
    pub fn get_year(&self) -> i32 {
        self.date().year() - 1900
    }

    #[prop("setDate")]
    pub fn set_date(&self, date_day: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_day(date_day).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setFullYear")]
    pub fn set_full_year(&self, year: i32, month: Option<u32>, day: Option<u32>) -> f64 {
        let month = month.unwrap_or(0) + 1;
        let day = day.unwrap_or(1);

        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_year(year).unwrap_or(date);
        date = date.with_month(month).unwrap_or(date);
        date = date.with_day(day).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setHours")]
    pub fn set_hours(
        &self,
        hours: u32,
        minutes: Option<u32>,
        seconds: Option<u32>,
        ms: Option<u32>,
    ) -> f64 {
        let minutes = minutes.unwrap_or(0);
        let seconds = seconds.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_hour(hours).unwrap_or(date);
        date = date.with_minute(minutes).unwrap_or(date);
        date = date.with_second(seconds).unwrap_or(date);
        date = date.with_nanosecond(ms * 1_000_000).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setMilliseconds")]
    pub fn set_milliseconds(&self, ms: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_nanosecond(ms * 1_000_000).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setMinutes")]
    pub fn set_minutes(&self, minutes: u32, seconds: Option<u32>, ms: Option<u32>) -> f64 {
        let seconds = seconds.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_minute(minutes).unwrap_or(date);
        date = date.with_second(seconds).unwrap_or(date);
        date = date.with_nanosecond(ms * 1_000_000).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setMonth")]
    pub fn set_month(&self, month: u32, day: Option<u32>) -> f64 {
        let day = day.unwrap_or(1);

        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_month(month).unwrap_or(date);
        date = date.with_day(day).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setSeconds")]
    pub fn set_seconds(&self, seconds: u32, ms: Option<u32>) -> f64 {
        let ms = ms.unwrap_or(0);

        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date;

        date = date.with_second(seconds).unwrap_or(date);
        date = date.with_nanosecond(ms * 1_000_000).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("setTime")]
    pub fn set_time(&self, time: f64) -> f64 {
        let secs = time.div_euclid(1000.0) as i64;
        let nsec = time.rem_euclid(1000.0) as u32 * 1_000_000;

        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let date = DateTime::from_timestamp(secs, nsec).unwrap_or(date.to_utc());

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCDate")]
    pub fn set_utc_date(&self, date_day: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let date = date.to_utc().with_day(date_day).unwrap_or(date.to_utc());

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCFullYear")]
    pub fn set_utc_full_year(&self, year: i32, month: Option<u32>, day: Option<u32>) -> f64 {
        let month = month.unwrap_or(0) + 1;
        let day = day.unwrap_or(1);

        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let mut date = date.to_utc().with_year(year).unwrap_or(date.to_utc());
        date = date.to_utc().with_month(month).unwrap_or(date);
        date = date.to_utc().with_day(day).unwrap_or(date);

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCHours")]
    pub fn set_utc_hours(
        &self,
        hours: u32,
        minutes: Option<u32>,
        seconds: Option<u32>,
        ms: Option<u32>,
    ) -> f64 {
        let minutes = minutes.unwrap_or(0);
        let seconds = seconds.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let mut date = date.to_utc().with_hour(hours).unwrap_or(date.to_utc());
        date = date.to_utc().with_minute(minutes).unwrap_or(date);
        date = date.to_utc().with_second(seconds).unwrap_or(date);
        date = date
            .to_utc()
            .with_nanosecond(ms * 1_000_000)
            .unwrap_or(date);

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCMilliseconds")]
    pub fn set_utc_milliseconds(&self, ms: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let date = date
            .to_utc()
            .with_nanosecond(ms * 1_000_000)
            .unwrap_or(date.to_utc());

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCMinutes")]
    pub fn set_utc_minutes(&self, minutes: u32, seconds: Option<u32>, ms: Option<u32>) -> f64 {
        let seconds = seconds.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let mut date = date.to_utc().with_minute(minutes).unwrap_or(date.to_utc());
        date = date.to_utc().with_second(seconds).unwrap_or(date);
        date = date
            .to_utc()
            .with_nanosecond(ms * 1_000_000)
            .unwrap_or(date);

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCMonth")]
    pub fn set_utc_month(&self, month: u32, day: Option<u32>) -> f64 {
        let day = day.unwrap_or(0) + 1;

        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let mut date = date.to_utc().with_month(month).unwrap_or(date.to_utc());
        date = date.to_utc().with_day(day).unwrap_or(date);

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setUTCSeconds")]
    pub fn set_utc_seconds(&self, seconds: u32, ms: Option<u32>) -> f64 {
        let ms = ms.unwrap_or(0);

        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let mut date = date.to_utc().with_second(seconds).unwrap_or(date.to_utc());
        date = date
            .to_utc()
            .with_nanosecond(ms * 1_000_000)
            .unwrap_or(date);

        inner.date = date.into();

        date.timestamp_millis() as f64
    }

    #[prop("setYear")]
    pub fn set_year(&self, year: i32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let date = inner.date;

        let date = date.with_year(year + 1900).unwrap_or(date);

        inner.date = date;

        date.timestamp_millis() as f64
    }

    #[prop("toDateString")]
    pub fn to_date_string(&self) -> String {
        self.date().format("%a %b %d %Y").to_string()
    }

    #[prop("toISOString")]
    pub fn to_iso_string(&self) -> String {
        self.date().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> String {
        self.to_iso_string()
    }

    // #[prop("toLocaleDateString")]
    // pub fn to_locale_date_string(&self) -> String {
    //     self.date().format("%Y-%m-%d").to_string()
    // }

    // #[prop("toLocaleString")]
    // pub fn to_locale_string(&self) -> String {
    //     self.date().format("%Y-%m-%d %H:%M:%S").to_string()
    // }

    // #[prop("toLocaleTimeString")]
    // pub fn to_locale_time_string(&self) -> String {
    //     self.date().format("%H:%M:%S").to_string()
    // }

    #[prop("toString")]
    pub fn to_string_js(&self) -> String {
        self.date().to_string()
    }

    #[prop("toTimeString")]
    pub fn to_time_string(&self) -> String {
        self.date().format("%H:%M:%S").to_string()
    }

    #[prop("toUTCString")]
    pub fn to_utc_string(&self) -> String {
        self.date().to_utc().to_string()
    }

    #[prop("valueOf")]
    pub fn value_of(&self) -> f64 {
        self.date().timestamp_millis() as f64
    }

    #[prop(Symbol::TO_PRIMITIVE)]
    pub fn to_primitive(&self, hint: &str) -> ValueResult {
        match hint {
            "number" => Ok(self.value_of().into()),
            _ => Ok(self.to_string_js().into()),
        }
    }
}

impl Date {
    fn date(&self) -> DateTime<Local> {
        self.inner.borrow().date
    }

    #[must_use]
    pub fn new(date: DateTime<Local>, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableDate {
                object: MutObject::with_proto(realm.intrinsics.date.clone().into()),
                date,
            }),
        }
    }

    pub fn js_construct(args: &[Value], realm: &mut Realm) -> Res<Self> {
        let date = match args.len() {
            0 => Local::now(),
            1 => {
                let arg = &args[0];

                match arg {
                    Value::String(s) => DateTime::from_str(s).unwrap_or(Local::now()),
                    Value::Number(time) => {
                        let time = *time;
                        let secs = time.div_euclid(1000.0) as i64;
                        let nsec = time.rem_euclid(1000.0) as u32 * 1_000_000;

                        DateTime::from_timestamp(secs, nsec)
                            .unwrap_or_default()
                            .into()
                    }

                    Value::Object(obj) => {
                        if let Ok(date) = <&Self>::from_value_out(obj.clone().into()) {
                            date.inner.borrow().date
                        } else {
                            let str = obj.to_string(realm)?;

                            DateTime::from_str(&str).unwrap_or(Local::now())
                        }
                    }

                    _ => Local::now(),
                }
            }
            _ => {
                let mut get = |idx: usize, def: i32| -> i32 {
                    args.get(idx)
                        .map_or(def, |v| v.to_number(realm).map(|x| x as i32).unwrap_or(def))
                };

                let year = get(0, 0);
                let month = get(1, 0);
                let day = get(2, 1) - 1;
                let hour = get(3, 0);
                let minute = get(4, 0);
                let second = get(5, 0);
                let ms = get(6, 0);

                let (ms, second) = fixup(ms, 1000, second);
                let (second, minute) = fixup(second, 60, minute);
                let (minute, hour) = fixup(minute, 60, hour);
                let (hour, day) = fixup(hour, 24, day);
                let (day, month) = fixup(day, 31, month);
                let (month, year) = fixup(month, 12, year);

                let month = month + 1;
                let day = day + 1;

                let time = match Local.with_ymd_and_hms(year, month, day, hour, minute, second) {
                    LocalResult::Single(date) => date,
                    LocalResult::Ambiguous(e, _) => e,
                    LocalResult::None => Local::now(),
                };

                time.with_nanosecond(ms * 1_000_000).unwrap_or(time)
            }
        };

        Ok(Self::new(date, realm))
    }
}

#[object(constructor, function)]
#[derive(Debug)]
pub struct DateConstructor {}

impl DateConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableDateConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

impl Constructor<Realm> for DateConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        Date::js_construct(&args, realm).map(Obj::into_value)
    }
}

impl Func<Realm> for DateConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        Date::js_construct(&args, realm).map(|date| {
            let inner = date.inner.borrow();

            inner.date.to_string().into()
        })
    }
}

#[properties_new(raw)]
impl DateConstructor {
    pub fn now(&self) -> ValueResult {
        Ok(Local::now().timestamp_millis().into())
    }

    pub fn parse(&self, s: &str) -> ValueResult {
        let date = DateTime::from_str(s).unwrap_or(Local::now());
        Ok(date.timestamp_millis().into())
    }

    #[prop("UTC")]
    pub fn utc(&self, args: &[Value], #[realm] realm: &mut Realm) -> ValueResult {
        if args.is_empty() {
            return Ok(f64::NAN.into());
        }

        let mut get = |idx: usize, def: i32| -> i32 {
            args.get(idx)
                .map_or(def, |v| v.to_number(realm).map(|x| x as i32).unwrap_or(def))
        };

        let year = get(0, 0);
        let month = get(1, 0);
        let day = get(2, 1) - 1;
        let hour = get(3, 0);
        let minute = get(4, 0);
        let second = get(5, 0);
        let ms = get(6, 0);

        let (ms, second) = fixup(ms, 1000, second);
        let (second, minute) = fixup(second, 60, minute);
        let (minute, hour) = fixup(minute, 60, hour);
        let (hour, day) = fixup(hour, 24, day);
        let (day, month) = fixup(day, 31, month);
        let (month, year) = fixup(month, 12, year);

        let month = month + 1;
        let day = day + 1;

        let time = match Local.with_ymd_and_hms(year, month, day, hour, minute, second) {
            LocalResult::Single(date) => date,
            LocalResult::Ambiguous(e, _) => e,
            LocalResult::None => Local::now(),
        };

        let time = time.with_nanosecond(ms * 1_000_000).unwrap_or(time);

        Ok(time.timestamp_millis().into())
    }
}

fn fixup(val: i32, max: i32, mut larger: i32) -> (u32, i32) {
    let val = if val.is_negative() {
        larger -= 1;
        max + val
    } else {
        val
    };

    let larger_diff = val / max;
    let val = val.rem(max);

    (val as u32, larger - larger_diff)
}


impl PrettyObjectOverride for Date {
    fn pretty_inline(&self, _obj: &yavashark_value::Object<Realm>, _not: &mut Vec<usize>) -> Option<String> {
        Some(self.date().format("%Y-%m-%d %H:%M:%S").to_string())
    }
}