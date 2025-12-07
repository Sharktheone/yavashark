use crate::conversion::downcast_obj;
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::Hint;
use crate::value::Obj;
use crate::{MutObject, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use chrono::{DateTime, Datelike, Local, LocalResult, Offset, TimeZone, Timelike};
use std::cell::RefCell;
use std::str::FromStr;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Date {
    #[mutable]
    date: Option<DateTime<Local>>,
}

#[props(intrinsic_name = date)]
impl Date {
    #[constructor]
    #[length(7)]
    fn construct(realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        Date::js_construct(&args, realm).map(Obj::into_object)
    }

    #[call_constructor]
    fn js_call(_realm: &mut Realm, _args: Vec<Value>) -> ValueResult {
        Ok(Local::now().to_string().into())
    }

    pub fn now() -> ValueResult {
        Ok(Local::now().timestamp_millis().into())
    }

    pub fn parse(s: &str) -> ValueResult {
        let date = DateTime::from_str(s).unwrap_or(Local::now());
        Ok(date.timestamp_millis().into())
    }

    #[prop("UTC")]
    pub fn utc(args: &[Value], #[realm] realm: &mut Realm) -> ValueResult {
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

    #[prop("getDate")]
    pub fn get_date(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.day() as f64)
    }

    #[prop("getDay")]
    pub fn get_day(&self) -> f64 {
        self.date()
            .map_or(f64::NAN, |d| d.weekday().num_days_from_sunday() as f64)
    }

    #[prop("getFullYear")]
    pub fn get_full_year(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.year() as f64)
    }

    #[prop("getHours")]
    pub fn get_hours(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.hour() as f64)
    }

    #[prop("getMilliseconds")]
    pub fn get_milliseconds(&self) -> f64 {
        self.date()
            .map_or(f64::NAN, |d| f64::from(d.nanosecond()) / 1_000_000.0)
    }

    #[prop("getMinutes")]
    pub fn get_minutes(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.minute() as f64)
    }

    #[prop("getMonth")]
    pub fn get_month(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| (d.month() - 1) as f64)
    }

    #[prop("getSeconds")]
    pub fn get_seconds(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.second() as f64)
    }

    #[prop("getTime")]
    pub fn get_time(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.timestamp_millis() as f64)
    }

    #[prop("getTimezoneOffset")]
    pub fn get_timezone_offset(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| {
            let secs = d.offset().fix().utc_minus_local();
            f64::from(secs) / 60.0
        })
    }

    #[prop("getUTCDate")]
    pub fn get_utc_date(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.to_utc().day() as f64)
    }

    #[prop("getUTCDay")]
    pub fn get_utc_day(&self) -> f64 {
        self.date()
            .map_or(f64::NAN, |d| d.to_utc().weekday().num_days_from_sunday() as f64)
    }

    #[prop("getUTCFullYear")]
    pub fn get_utc_full_year(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.to_utc().year() as f64)
    }

    #[prop("getUTCHours")]
    pub fn get_utc_hours(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.to_utc().hour() as f64)
    }

    #[prop("getUTCMilliseconds")]
    pub fn get_utc_milliseconds(&self) -> f64 {
        self.date()
            .map_or(f64::NAN, |d| f64::from(d.to_utc().nanosecond()) / 1_000_000.0)
    }

    #[prop("getUTCMinutes")]
    pub fn get_utc_minutes(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.to_utc().minute() as f64)
    }

    #[prop("getUTCMonth")]
    pub fn get_utc_month(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| (d.to_utc().month() - 1) as f64)
    }

    #[prop("getUTCSeconds")]
    pub fn get_utc_seconds(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.to_utc().second() as f64)
    }

    #[prop("getYear")]
    pub fn get_year(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| (d.year() - 1900) as f64)
    }

    #[prop("setDate")]
    pub fn set_date(&self, date_day: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let date = date.with_day(date_day).unwrap_or(date);

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("setFullYear")]
    pub fn set_full_year(&self, year: i32, month: Option<u32>, day: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let mut date = inner.date.unwrap_or_else(|| {
            DateTime::from_timestamp(0, 0)
                .unwrap_or_default()
                .with_timezone(&Local)
        });

        let month = month.map(|m| m + 1);
        let day = day;

        date = date.with_year(year).unwrap_or(date);
        if let Some(m) = month {
            date = date.with_month(m).unwrap_or(date);
        }
        if let Some(d) = day {
            date = date.with_day(d).unwrap_or(date);
        }

        inner.date = Some(date);

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
        let mut inner = self.inner.borrow_mut();

        let Some(mut date) = inner.date else {
            return f64::NAN;
        };

        date = date.with_hour(hours).unwrap_or(date);
        if let Some(m) = minutes {
            date = date.with_minute(m).unwrap_or(date);
        }
        if let Some(s) = seconds {
            date = date.with_second(s).unwrap_or(date);
        }
        if let Some(ms_val) = ms {
            date = date.with_nanosecond(ms_val * 1_000_000).unwrap_or(date);
        }

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("setMilliseconds")]
    pub fn set_milliseconds(&self, ms: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let date = date.with_nanosecond(ms * 1_000_000).unwrap_or(date);

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("setMinutes")]
    pub fn set_minutes(&self, minutes: u32, seconds: Option<u32>, ms: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(mut date) = inner.date else {
            return f64::NAN;
        };

        date = date.with_minute(minutes).unwrap_or(date);
        if let Some(s) = seconds {
            date = date.with_second(s).unwrap_or(date);
        }
        if let Some(ms_val) = ms {
            date = date.with_nanosecond(ms_val * 1_000_000).unwrap_or(date);
        }

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("setMonth")]
    pub fn set_month(&self, month: u32, day: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(mut date) = inner.date else {
            return f64::NAN;
        };

        date = date.with_month(month + 1).unwrap_or(date);
        if let Some(d) = day {
            date = date.with_day(d).unwrap_or(date);
        }

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("setSeconds")]
    pub fn set_seconds(&self, seconds: u32, ms: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(mut date) = inner.date else {
            return f64::NAN;
        };

        date = date.with_second(seconds).unwrap_or(date);
        if let Some(ms_val) = ms {
            date = date.with_nanosecond(ms_val * 1_000_000).unwrap_or(date);
        }

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("setTime")]
    pub fn set_time(&self, time: f64) -> f64 {
        let mut inner = self.inner.borrow_mut();

        if !time.is_finite() {
            inner.date = None;
            return f64::NAN;
        }

        let secs = time.div_euclid(1000.0) as i64;
        let nsec = time.rem_euclid(1000.0) as u32 * 1_000_000;

        let date = DateTime::from_timestamp(secs, nsec)
            .map(|d| d.with_timezone(&Local));

        inner.date = date;

        date.map_or(f64::NAN, |d| d.timestamp_millis() as f64)
    }

    #[prop("setUTCDate")]
    pub fn set_utc_date(&self, date_day: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let date = date.to_utc().with_day(date_day).unwrap_or(date.to_utc());

        inner.date = Some(date.with_timezone(&Local));

        date.timestamp_millis() as f64
    }

    #[prop("setUTCFullYear")]
    pub fn set_utc_full_year(&self, year: i32, month: Option<u32>, day: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let date = inner
            .date
            .map(|d| d.to_utc())
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap_or_default());

        let month = month.map(|m| m + 1);
        let day = day;

        let mut date = date.with_year(year).unwrap_or(date);
        if let Some(m) = month {
            date = date.with_month(m).unwrap_or(date);
        }
        if let Some(d) = day {
            date = date.with_day(d).unwrap_or(date);
        }

        inner.date = Some(date.with_timezone(&Local));

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
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let mut date = date.to_utc().with_hour(hours).unwrap_or(date.to_utc());
        if let Some(m) = minutes {
            date = date.with_minute(m).unwrap_or(date);
        }
        if let Some(s) = seconds {
            date = date.with_second(s).unwrap_or(date);
        }
        if let Some(ms_val) = ms {
            date = date.with_nanosecond(ms_val * 1_000_000).unwrap_or(date);
        }

        inner.date = Some(date.with_timezone(&Local));

        date.timestamp_millis() as f64
    }

    #[prop("setUTCMilliseconds")]
    pub fn set_utc_milliseconds(&self, ms: u32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let date = date
            .to_utc()
            .with_nanosecond(ms * 1_000_000)
            .unwrap_or(date.to_utc());

        inner.date = Some(date.with_timezone(&Local));

        date.timestamp_millis() as f64
    }

    #[prop("setUTCMinutes")]
    pub fn set_utc_minutes(&self, minutes: u32, seconds: Option<u32>, ms: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let mut date = date.to_utc().with_minute(minutes).unwrap_or(date.to_utc());
        if let Some(s) = seconds {
            date = date.with_second(s).unwrap_or(date);
        }
        if let Some(ms_val) = ms {
            date = date.with_nanosecond(ms_val * 1_000_000).unwrap_or(date);
        }

        inner.date = Some(date.with_timezone(&Local));

        date.timestamp_millis() as f64
    }

    #[prop("setUTCMonth")]
    pub fn set_utc_month(&self, month: u32, day: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let mut date = date.to_utc().with_month(month + 1).unwrap_or(date.to_utc());
        if let Some(d) = day {
            date = date.with_day(d).unwrap_or(date);
        }

        inner.date = Some(date.with_timezone(&Local));

        date.timestamp_millis() as f64
    }

    #[prop("setUTCSeconds")]
    pub fn set_utc_seconds(&self, seconds: u32, ms: Option<u32>) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let mut date = date.to_utc().with_second(seconds).unwrap_or(date.to_utc());
        if let Some(ms_val) = ms {
            date = date.with_nanosecond(ms_val * 1_000_000).unwrap_or(date);
        }

        inner.date = Some(date.with_timezone(&Local));

        date.timestamp_millis() as f64
    }

    #[prop("setYear")]
    pub fn set_year(&self, year: i32) -> f64 {
        let mut inner = self.inner.borrow_mut();

        let Some(date) = inner.date else {
            return f64::NAN;
        };

        let date = date.with_year(year + 1900).unwrap_or(date);

        inner.date = Some(date);

        date.timestamp_millis() as f64
    }

    #[prop("toDateString")]
    pub fn to_date_string(&self) -> String {
        self.date()
            .map_or("Invalid Date".to_string(), |d| {
                d.format("%a %b %d %Y").to_string()
            })
    }

    #[prop("toISOString")]
    pub fn to_iso_string(&self) -> ValueResult {
        match self.date() {
            Some(d) => Ok(d.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string().into()),
            None => Err(crate::Error::range("Invalid time value")),
        }
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> ValueResult {
        match self.date() {
            Some(d) => Ok(d.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string().into()),
            None => Ok(Value::Null),
        }
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
        self.date()
            .map_or("Invalid Date".to_string(), |d| d.to_string())
    }

    #[prop("toTimeString")]
    pub fn to_time_string(&self) -> String {
        self.date()
            .map_or("Invalid Date".to_string(), |d| {
                d.format("%H:%M:%S").to_string()
            })
    }

    #[prop("toUTCString")]
    pub fn to_utc_string(&self) -> String {
        self.date()
            .map_or("Invalid Date".to_string(), |d| d.to_utc().to_string())
    }

    #[prop("valueOf")]
    pub fn value_of(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.timestamp_millis() as f64)
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
    fn date(&self) -> Option<DateTime<Local>> {
        self.inner.borrow().date
    }

    pub fn new(date: Option<DateTime<Local>>, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableDate {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().date.get(realm)?.clone(),
                ),
                date,
            }),
        })
    }

    pub fn js_construct(args: &[Value], realm: &mut Realm) -> Res<Self> {
        let date = match args.len() {
            0 => Some(Local::now()),
            1 => {
                let arg = &args[0];

                match arg {
                    Value::String(s) => DateTime::from_str(s).ok(),
                    Value::Number(time) => {
                        let time = *time;
                        if !time.is_finite() {
                            None
                        } else {
                            let secs = time.div_euclid(1000.0) as i64;
                            let nsec = time.rem_euclid(1000.0) as u32 * 1_000_000;

                            DateTime::from_timestamp(secs, nsec).map(|d| d.with_timezone(&Local))
                        }
                    }

                    Value::Object(obj) => {
                        if let Ok(date) = downcast_obj::<Self>(obj.clone().into()) {
                            date.inner.borrow().date
                        } else {
                            let prim = obj.to_primitive(Hint::None, realm)?;

                            match prim {
                                Value::String(s) => DateTime::from_str(&s).ok(),
                                Value::Symbol(_) => {
                                    return Err(crate::Error::ty(
                                        "Cannot convert a Symbol value to a number",
                                    ));
                                }
                                other => {
                                    let time = other.to_number(realm)?;
                                    if !time.is_finite() {
                                        None
                                    } else {
                                        let secs = time.div_euclid(1000.0) as i64;
                                        let nsec = time.rem_euclid(1000.0) as u32 * 1_000_000;

                                        DateTime::from_timestamp(secs, nsec)
                                            .map(|d| d.with_timezone(&Local))
                                    }
                                }
                            }
                        }
                    }

                    _ => None,
                }
            }
            _ => {
                let get = |v: &Value, realm: &mut Realm| -> Res<Option<i32>> {
                    let num = v.to_number(realm)?;
                    if num.is_finite() {
                        Ok(Some(num as i32))
                    } else {
                        Ok(None)
                    }
                };

                let year = if let Some(v) = args.first() {
                    match get(v, realm)? {
                        Some(y) => y,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let year = if (0..=99).contains(&year) {
                    1900 + year
                } else {
                    year
                };

                let month = if let Some(v) = args.get(1) {
                    match get(v, realm)? {
                        Some(m) => m,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let day = if let Some(v) = args.get(2) {
                    match get(v, realm)? {
                        Some(d) => d - 1,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let hour = if let Some(v) = args.get(3) {
                    match get(v, realm)? {
                        Some(h) => h,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let minute = if let Some(v) = args.get(4) {
                    match get(v, realm)? {
                        Some(m) => m,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let second = if let Some(v) = args.get(5) {
                    match get(v, realm)? {
                        Some(s) => s,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let ms = if let Some(v) = args.get(6) {
                    match get(v, realm)? {
                        Some(m) => m,
                        None => return Self::new(None, realm),
                    }
                } else {
                    0
                };

                let (ms, second) = fixup(ms, 1000, second);
                let (second, minute) = fixup(second, 60, minute);
                let (minute, hour) = fixup(minute, 60, hour);
                let (hour, day) = fixup(hour, 24, day);
                let (day, month) = fixup(day, 31, month);
                let (month, year) = fixup(month, 12, year);

                let month = month + 1;
                let day = day + 1;

                let time = match Local.with_ymd_and_hms(year, month, day, hour, minute, second) {
                    LocalResult::Single(date) => Some(date),
                    LocalResult::Ambiguous(e, _) => Some(e),
                    LocalResult::None => None,
                };

                time.and_then(|t| t.with_nanosecond(ms * 1_000_000))
            }
        };

        Self::new(date, realm)
    }
}

fn fixup(val: i32, max: i32, larger: i32) -> (u32, i32) {
    if val.is_negative() {
        let abs_val = (val as i64).wrapping_neg();
        let max_64 = max as i64;
        let borrow = (abs_val + max_64 - 1) / max_64;
        let new_larger = (larger as i64).saturating_sub(borrow);
        let val = ((max_64 - (abs_val % max_64)) % max_64) as u32;
        (val, new_larger.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
    } else if val >= max {
        let carry = (val / max) as i64;
        let new_larger = (larger as i64).saturating_add(carry);
        let val = val % max;
        (val as u32, new_larger.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
    } else {
        (val as u32, larger)
    }
}

impl PrettyObjectOverride for Date {
    fn pretty_inline(
        &self,
        obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self
            .date()
            .map_or("Invalid Date".to_string(), |d| {
                d.format("%Y-%m-%d %H:%M:%S").to_string()
            });

        fmt_properties_to(obj, &mut s, not, realm);

        Some(s)
    }

    fn pretty_multiline(
        &self,
        obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        self.pretty_inline(obj, not, realm)
    }
}
