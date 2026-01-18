use crate::conversion::downcast_obj;
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::Hint;
use crate::value::Obj;
use crate::{MutObject, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use chrono::{DateTime, Datelike, Local, LocalResult, Offset, TimeZone, Timelike, Utc};
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
    #[length(7)]
    pub fn utc(args: &[Value], #[realm] realm: &mut Realm) -> ValueResult {
        if args.is_empty() {
            return Ok(f64::NAN.into());
        }

        // Helper to convert arg to number, returning None if NaN/Inf
        let get_num = |idx: usize, args: &[Value], realm: &mut Realm| -> Res<Option<f64>> {
            if let Some(v) = args.get(idx) {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    Ok(None)
                } else {
                    Ok(Some(n))
                }
            } else {
                Ok(None)
            }
        };

        // Step 1-7: Get arguments as numbers
        let year = match get_num(0, args, realm)? {
            Some(n) => n,
            None => return Ok(f64::NAN.into()),
        };

        let month = match args.get(1) {
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    return Ok(f64::NAN.into());
                }
                n
            }
            None => 0.0, // Default month if not provided
        };

        let date = match args.get(2) {
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    return Ok(f64::NAN.into());
                }
                n
            }
            None => 1.0,
        };

        let hours = match args.get(3) {
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    return Ok(f64::NAN.into());
                }
                n
            }
            None => 0.0,
        };

        let minutes = match args.get(4) {
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    return Ok(f64::NAN.into());
                }
                n
            }
            None => 0.0,
        };

        let seconds = match args.get(5) {
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    return Ok(f64::NAN.into());
                }
                n
            }
            None => 0.0,
        };

        let ms = match args.get(6) {
            Some(v) => {
                let n = v.to_number(realm)?;
                if n.is_nan() || n.is_infinite() {
                    return Ok(f64::NAN.into());
                }
                n
            }
            None => 0.0,
        };

        // Step 8: If 0 ≤ ToInteger(y) ≤ 99, add 1900
        // ToInteger truncates toward zero
        let year_int = year.trunc();
        let yr = if (0.0..=99.0).contains(&year_int) {
            1900.0 + year_int
        } else {
            year
        };

        // Step 9: MakeDay, MakeTime, MakeDate, TimeClip
        let day = make_day(yr, month, date);
        let time = make_time(hours, minutes, seconds, ms);

        let result = if let (Some(d), Some(t)) = (day, time) {
            make_date(Some(d), t).and_then(time_clip)
        } else {
            None
        };

        Ok(result.unwrap_or(f64::NAN).into())
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
        self.date()
            .map_or(f64::NAN, |d| d.timestamp_millis() as f64)
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
        self.date().map_or(f64::NAN, |d| {
            d.to_utc().weekday().num_days_from_sunday() as f64
        })
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
        self.date().map_or(f64::NAN, |d| {
            f64::from(d.to_utc().nanosecond()) / 1_000_000.0
        })
    }

    #[prop("getUTCMinutes")]
    pub fn get_utc_minutes(&self) -> f64 {
        self.date().map_or(f64::NAN, |d| d.to_utc().minute() as f64)
    }

    #[prop("getUTCMonth")]
    pub fn get_utc_month(&self) -> f64 {
        self.date()
            .map_or(f64::NAN, |d| (d.to_utc().month() - 1) as f64)
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
    pub fn set_date(&self, dt: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4: Let dt be ? ToNumber(date)
        let dt = dt.to_number(realm)?;

        // Step 5: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        if !dt.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        // Step 6: Set t to LocalTime(t)
        let local_t = utc_to_local(t_val);

        let new_date = make_date(
            make_day(
                year_from_time(local_t) as f64,
                month_from_time(local_t) as f64,
                dt,
            ),
            time_within_day(local_t),
        );

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setFullYear")]
    #[length(3)]
    pub fn set_full_year(
        &self,
        year: Value,
        month: Option<Value>,
        date: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4: Let y be ? ToNumber(year)
        let y = year.to_number(realm)?;

        // Step 5: If t is NaN, set t to +0; otherwise set t to LocalTime(t)
        let local_t = match t {
            Some(t_val) => utc_to_local(t_val),
            None => 0.0,
        };

        // Step 6-7: Handle optional month and date parameters
        let m = if let Some(m_val) = month {
            m_val.to_number(realm)?
        } else {
            month_from_time(local_t) as f64
        };

        let dt = if let Some(d_val) = date {
            d_val.to_number(realm)?
        } else {
            date_from_time(local_t) as f64
        };

        if !y.is_finite() || !m.is_finite() || !dt.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let new_date = make_date(make_day(y, m, dt), time_within_day(local_t));

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setHours")]
    #[length(4)]
    pub fn set_hours(
        &self,
        hour: Value,
        min: Option<Value>,
        sec: Option<Value>,
        ms: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-7: ToNumber on arguments
        let h = hour.to_number(realm)?;
        let m = if let Some(v) = min {
            Some(v.to_number(realm)?)
        } else {
            None
        };
        let s = if let Some(v) = sec {
            Some(v.to_number(realm)?)
        } else {
            None
        };
        let milli = if let Some(v) = ms {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 8: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 9: Set t to LocalTime(t)
        let local_t = utc_to_local(t_val);

        // Steps 10-12: Use existing values if not provided
        let m = m.unwrap_or_else(|| minutes_from_time(local_t) as f64);
        let s = s.unwrap_or_else(|| seconds_from_time(local_t) as f64);
        let milli = milli.unwrap_or_else(|| ms_from_time(local_t) as f64);

        // Check for NaN in any argument
        if !h.is_finite() || !m.is_finite() || !s.is_finite() || !milli.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (local_t / MS_PER_DAY as f64).floor();
        let new_time = make_time(h, m, s, milli);

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setMilliseconds")]
    pub fn set_milliseconds(&self, ms: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4: Let ms be ? ToNumber(ms)
        let ms = ms.to_number(realm)?;

        // Step 5: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 6: Set t to LocalTime(t)
        let local_t = utc_to_local(t_val);

        if !ms.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (local_t / MS_PER_DAY as f64).floor();
        let new_time = make_time(
            hours_from_time(local_t) as f64,
            minutes_from_time(local_t) as f64,
            seconds_from_time(local_t) as f64,
            ms,
        );

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setMinutes")]
    #[length(3)]
    pub fn set_minutes(
        &self,
        min: Value,
        sec: Option<Value>,
        ms: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-6: ToNumber on arguments
        let min = min.to_number(realm)?;
        let s = if let Some(v) = sec {
            Some(v.to_number(realm)?)
        } else {
            None
        };
        let milli = if let Some(v) = ms {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 7: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 8: Set t to LocalTime(t)
        let local_t = utc_to_local(t_val);

        // Steps 9-10: Use existing values if not provided
        let s = s.unwrap_or_else(|| seconds_from_time(local_t) as f64);
        let milli = milli.unwrap_or_else(|| ms_from_time(local_t) as f64);

        // Check for NaN in any argument
        if !min.is_finite() || !s.is_finite() || !milli.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (local_t / MS_PER_DAY as f64).floor();
        let new_time = make_time(hours_from_time(local_t) as f64, min, s, milli);

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setMonth")]
    #[length(2)]
    pub fn set_month(
        &self,
        month: Value,
        date: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-5: ToNumber on arguments
        let month = month.to_number(realm)?;
        let dt = if let Some(v) = date {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 6: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 7: Set t to LocalTime(t)
        let local_t = utc_to_local(t_val);

        // Step 8: Use existing date if not provided
        let dt = dt.unwrap_or_else(|| date_from_time(local_t) as f64);

        if !month.is_finite() || !dt.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let new_date = make_date(
            make_day(year_from_time(local_t) as f64, month, dt),
            time_within_day(local_t),
        );

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setSeconds")]
    #[length(2)]
    pub fn set_seconds(
        &self,
        sec: Value,
        ms: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-5: ToNumber on arguments
        let sec = sec.to_number(realm)?;
        let milli = if let Some(v) = ms {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 6: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 7: Set t to LocalTime(t)
        let local_t = utc_to_local(t_val);

        // Step 8: Use existing ms if not provided
        let milli = milli.unwrap_or_else(|| ms_from_time(local_t) as f64);

        // Check for NaN in any argument
        if !sec.is_finite() || !milli.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (local_t / MS_PER_DAY as f64).floor();
        let new_time = make_time(
            hours_from_time(local_t) as f64,
            minutes_from_time(local_t) as f64,
            sec,
            milli,
        );

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setTime")]
    pub fn set_time(&self, time: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // Step 3: Let t be ? ToNumber(time)
        let time = time.to_number(realm)?;

        if !time.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        // Step 4: Let v be TimeClip(t)
        let v = time_clip(time);

        // Step 5: Set dateObject.[[DateValue]] to v
        self.inner.borrow_mut().date = v.and_then(ms_to_local);

        // Step 6: Return v
        Ok(v.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCDate")]
    pub fn set_utc_date(&self, dt: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4: Let dt be ? ToNumber(date)
        let dt = dt.to_number(realm)?;

        // Step 5: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        if !dt.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let new_date = make_date(
            make_day(
                utc_year_from_time(t_val) as f64,
                utc_month_from_time(t_val) as f64,
                dt,
            ),
            time_within_day(t_val),
        );

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCFullYear")]
    #[length(3)]
    pub fn set_utc_full_year(
        &self,
        year: Value,
        month: Option<Value>,
        date: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]. If t is NaN, set t to +0
        let t = self
            .inner
            .borrow()
            .date
            .map_or(0.0, |d| d.timestamp_millis() as f64);

        // Step 4: Let y be ? ToNumber(year)
        let y = year.to_number(realm)?;

        // Step 5-6: Handle optional month and date parameters
        let m = if let Some(m_val) = month {
            m_val.to_number(realm)?
        } else {
            utc_month_from_time(t) as f64
        };

        let dt = if let Some(d_val) = date {
            d_val.to_number(realm)?
        } else {
            utc_date_from_time(t) as f64
        };

        if !y.is_finite() || !m.is_finite() || !dt.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let new_date = make_date(make_day(y, m, dt), time_within_day(t));

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCHours")]
    #[length(4)]
    pub fn set_utc_hours(
        &self,
        hour: Value,
        min: Option<Value>,
        sec: Option<Value>,
        ms: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-7: ToNumber on arguments
        let h = hour.to_number(realm)?;
        let m = if let Some(v) = min {
            Some(v.to_number(realm)?)
        } else {
            None
        };
        let s = if let Some(v) = sec {
            Some(v.to_number(realm)?)
        } else {
            None
        };
        let milli = if let Some(v) = ms {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 8: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Steps 9-11: Use existing values if not provided
        let m = m.unwrap_or_else(|| utc_minutes_from_time(t_val) as f64);
        let s = s.unwrap_or_else(|| utc_seconds_from_time(t_val) as f64);
        let milli = milli.unwrap_or_else(|| utc_ms_from_time(t_val) as f64);

        // Check for NaN in any argument
        if !h.is_finite() || !m.is_finite() || !s.is_finite() || !milli.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (t_val / MS_PER_DAY as f64).floor();
        let new_time = make_time(h, m, s, milli);

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCMilliseconds")]
    pub fn set_utc_milliseconds(&self, ms: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4: Let milli be ? ToNumber(ms)
        let ms = ms.to_number(realm)?;

        // Step 5: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        if !ms.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (t_val / MS_PER_DAY as f64).floor();
        let new_time = make_time(
            utc_hours_from_time(t_val) as f64,
            utc_minutes_from_time(t_val) as f64,
            utc_seconds_from_time(t_val) as f64,
            ms,
        );

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCMinutes")]
    #[length(3)]
    pub fn set_utc_minutes(
        &self,
        min: Value,
        sec: Option<Value>,
        ms: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-6: ToNumber on arguments
        let min = min.to_number(realm)?;
        let s = if let Some(v) = sec {
            Some(v.to_number(realm)?)
        } else {
            None
        };
        let milli = if let Some(v) = ms {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 7: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Steps 8-9: Use existing values if not provided
        let s = s.unwrap_or_else(|| utc_seconds_from_time(t_val) as f64);
        let milli = milli.unwrap_or_else(|| utc_ms_from_time(t_val) as f64);

        // Check for NaN in any argument
        if !min.is_finite() || !s.is_finite() || !milli.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (t_val / MS_PER_DAY as f64).floor();
        let new_time = make_time(utc_hours_from_time(t_val) as f64, min, s, milli);

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCMonth")]
    #[length(2)]
    pub fn set_utc_month(
        &self,
        month: Value,
        date: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-5: ToNumber on arguments
        let month = month.to_number(realm)?;
        let dt = if let Some(v) = date {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 6: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 7: Use existing date if not provided
        let dt = dt.unwrap_or_else(|| utc_date_from_time(t_val) as f64);

        if !month.is_finite() || !dt.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let new_date = make_date(
            make_day(utc_year_from_time(t_val) as f64, month, dt),
            time_within_day(t_val),
        );

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setUTCSeconds")]
    #[length(2)]
    pub fn set_utc_seconds(
        &self,
        sec: Value,
        ms: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> ValueResult {
        // Step 3: Let t be dateObject.[[DateValue]]
        let t = self
            .inner
            .borrow()
            .date
            .map(|d| d.timestamp_millis() as f64);

        // Step 4-5: ToNumber on arguments
        let sec = sec.to_number(realm)?;
        let milli = if let Some(v) = ms {
            Some(v.to_number(realm)?)
        } else {
            None
        };

        // Step 6: If t is NaN, return NaN
        let Some(t_val) = t else {
            return Ok(f64::NAN.into());
        };

        // Step 7: Use existing ms if not provided
        let milli = milli.unwrap_or_else(|| utc_ms_from_time(t_val) as f64);

        // Check for NaN in any argument
        if !sec.is_finite() || !milli.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        let day = (t_val / MS_PER_DAY as f64).floor();
        let new_time = make_time(
            utc_hours_from_time(t_val) as f64,
            utc_minutes_from_time(t_val) as f64,
            sec,
            milli,
        );

        let new_date = make_date(Some(day), new_time.unwrap_or(f64::NAN));

        let result = new_date.and_then(time_clip);

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("setYear")]
    pub fn set_year(&self, year: Value, #[realm] realm: &mut Realm) -> ValueResult {
        // Per spec: If t is NaN, let t be +0; otherwise, let t be LocalTime(t)
        let local_t = if let Some(d) = self.inner.borrow().date {
            let utc_t = d.timestamp_millis() as f64;
            utc_to_local(utc_t)
        } else {
            // NaN case: use +0 directly (no conversion)
            0.0
        };

        let year = year.to_number(realm)?;

        if !year.is_finite() {
            self.inner.borrow_mut().date = None;
            return Ok(f64::NAN.into());
        }

        // setYear interprets values 0-99 as 1900+year
        let year = if year >= 0.0 && year <= 99.0 {
            1900.0 + year.trunc()
        } else {
            year.trunc()
        };

        let new_date = make_date(
            make_day(
                year,
                month_from_time(local_t) as f64,
                date_from_time(local_t) as f64,
            ),
            time_within_day(local_t),
        );

        let result = new_date.and_then(|d| time_clip(local_to_utc(d)));

        self.inner.borrow_mut().date = result.and_then(ms_to_local);

        Ok(result.unwrap_or(f64::NAN).into())
    }

    #[prop("toDateString")]
    pub fn to_date_string(&self) -> String {
        self.date().map_or("Invalid Date".to_string(), |d| {
            d.format("%a %b %d %Y").to_string()
        })
    }

    #[prop("toISOString")]
    pub fn to_iso_string(&self) -> ValueResult {
        match self.date() {
            Some(d) => Ok(d
                .to_utc()
                .format("%Y-%m-%dT%H:%M:%S.%3fZ")
                .to_string()
                .into()),
            None => Err(crate::Error::range("Invalid time value")),
        }
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> ValueResult {
        match self.date() {
            Some(d) => Ok(d
                .to_utc()
                .format("%Y-%m-%dT%H:%M:%S.%3fZ")
                .to_string()
                .into()),
            None => Ok(Value::Null),
        }
    }

    #[prop("toLocaleDateString")]
    pub fn to_locale_date_string(&self) -> String {
        self.date().map_or("Invalid Date".to_string(), |d| {
            d.format("%m/%d/%Y").to_string()
        })
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self) -> String {
        self.date().map_or("Invalid Date".to_string(), |d| {
            d.format("%m/%d/%Y, %I:%M:%S %p").to_string()
        })
    }

    #[prop("toLocaleTimeString")]
    pub fn to_locale_time_string(&self) -> String {
        self.date().map_or("Invalid Date".to_string(), |d| {
            d.format("%I:%M:%S %p").to_string()
        })
    }

    #[prop("toString")]
    pub fn to_string_js(&self) -> String {
        self.date()
            .map_or("Invalid Date".to_string(), |d| d.to_string())
    }

    #[prop("toTimeString")]
    pub fn to_time_string(&self) -> String {
        self.date().map_or("Invalid Date".to_string(), |d| {
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
        self.date()
            .map_or(f64::NAN, |d| d.timestamp_millis() as f64)
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
                    Value::String(s) => match parse_date_string(&s.as_str_lossy()) {
                        ParseResult::Parsed(dt) => Some(dt),
                        ParseResult::Invalid => None,
                        ParseResult::NotMatched => DateTime::from_str(&s.as_str_lossy()).ok(),
                    },
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
                                Value::String(s) => match parse_date_string(&s.as_str_lossy()) {
                                    ParseResult::Parsed(dt) => Some(dt),
                                    ParseResult::Invalid => None,
                                    ParseResult::NotMatched => {
                                        DateTime::from_str(&s.as_str_lossy()).ok()
                                    }
                                },
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
        (
            val,
            new_larger.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        )
    } else if val >= max {
        let carry = (val / max) as i64;
        let new_larger = (larger as i64).saturating_add(carry);
        let val = val % max;
        (
            val as u32,
            new_larger.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        )
    } else {
        (val as u32, larger)
    }
}

// Constants for date calculations (all in milliseconds)
const MS_PER_SECOND: i64 = 1000;
const MS_PER_MINUTE: i64 = 60 * MS_PER_SECOND;
const MS_PER_HOUR: i64 = 60 * MS_PER_MINUTE;
const MS_PER_DAY: i64 = 24 * MS_PER_HOUR;

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn make_day(year: f64, month: f64, date: f64) -> Option<f64> {
    if !year.is_finite() || !month.is_finite() || !date.is_finite() {
        return None;
    }

    let year = year as i64;
    let month = month as i64;
    let date = date as i64;

    // Adjust year for month overflow
    let year = year + month.div_euclid(12);
    let month = month.rem_euclid(12) as u32;

    // Calculate days from epoch to start of year
    // Using a reference: 1970-01-01 is day 0
    let mut days: i64 = 0;

    if year >= 1970 {
        for y in 1970..year {
            days += if is_leap_year(y as i32) { 366 } else { 365 };
        }
    } else {
        for y in year..1970 {
            days -= if is_leap_year(y as i32) { 366 } else { 365 };
        }
    }

    // Add days for months in the target year
    let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 0..month {
        days += days_per_month[m as usize] as i64;
        if m == 1 && is_leap_year(year as i32) {
            days += 1;
        }
    }

    // Add the day of month (date is 1-indexed, so subtract 1)
    days += date - 1;

    Some(days as f64)
}

/// MakeDate per ECMA-262: combines day and time
fn make_date(day: Option<f64>, time: f64) -> Option<f64> {
    let day = day?;
    if !day.is_finite() || !time.is_finite() {
        return None;
    }
    Some(day * MS_PER_DAY as f64 + time)
}

/// MakeTime per ECMA-262
const fn make_time(hour: f64, min: f64, sec: f64, ms: f64) -> Option<f64> {
    if !hour.is_finite() || !min.is_finite() || !sec.is_finite() || !ms.is_finite() {
        return None;
    }
    let h = hour as i64;
    let m = min as i64;
    let s = sec as i64;
    let ms = ms as i64;
    Some((h * MS_PER_HOUR + m * MS_PER_MINUTE + s * MS_PER_SECOND + ms) as f64)
}

fn time_clip(time: f64) -> Option<f64> {
    if !time.is_finite() {
        return None;
    }
    // Max date value is 8.64e15 ms
    if time.abs() > 8.64e15 {
        return None;
    }
    Some(time.trunc())
}

fn ms_to_local(ms: f64) -> Option<DateTime<Local>> {
    if !ms.is_finite() {
        return None;
    }
    let secs = (ms / 1000.0).floor() as i64;
    let nsecs = ((ms % 1000.0) * 1_000_000.0) as u32;
    DateTime::from_timestamp(secs, nsecs).map(|d| d.with_timezone(&Local))
}

fn time_within_day(t: f64) -> f64 {
    t.rem_euclid(MS_PER_DAY as f64)
}

fn year_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(1970, |d| d.year())
}

fn month_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(0, |d| d.month() as i32 - 1)
}

fn date_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(1, |d| d.day() as i32)
}

fn hours_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(0, |d| d.hour() as i32)
}

fn minutes_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(0, |d| d.minute() as i32)
}

fn seconds_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(0, |d| d.second() as i32)
}

fn ms_from_time(t: f64) -> i32 {
    ms_to_local(t).map_or(0, |d| (d.nanosecond() / 1_000_000) as i32)
}

fn utc_year_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(1970, |d| d.year())
}

fn utc_month_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(0, |d| d.month() as i32 - 1)
}

fn utc_date_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(1, |d| d.day() as i32)
}

fn utc_hours_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(0, |d| d.hour() as i32)
}

fn utc_minutes_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(0, |d| d.minute() as i32)
}

fn utc_seconds_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(0, |d| d.second() as i32)
}

fn utc_ms_from_time(t: f64) -> i32 {
    DateTime::from_timestamp_millis(t as i64).map_or(0, |d| (d.nanosecond() / 1_000_000) as i32)
}

fn local_to_utc(t: f64) -> f64 {
    // Get the timezone offset at time t
    if let Some(local) = ms_to_local(t) {
        let offset_secs = local.offset().fix().local_minus_utc() as f64;
        t - offset_secs * 1000.0
    } else {
        t
    }
}

fn utc_to_local(t: f64) -> f64 {
    if let Some(utc) = DateTime::from_timestamp_millis(t as i64) {
        let local = utc.with_timezone(&Local);
        let offset_secs = local.offset().fix().local_minus_utc() as f64;
        t + offset_secs * 1000.0
    } else {
        t
    }
}

impl PrettyObjectOverride for Date {
    fn pretty_inline(
        &self,
        obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self.date().map_or("Invalid Date".to_string(), |d| {
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

/// Result of parsing an ECMAScript Date Time String Format string
enum ParseResult {
    /// Successfully parsed to a datetime
    Parsed(DateTime<Local>),
    /// Matched the format but was invalid (e.g., -000000)
    Invalid,
    /// Did not match the ES6 format at all
    NotMatched,
}

/// Parse an ECMAScript Date Time String Format string
/// Formats supported:
/// - YYYY (year only)
/// - YYYY-MM (year and month)
/// - YYYY-MM-DD (full date)
/// - Plus time forms: THH:mm, THH:mm:ss, THH:mm:ss.sss
/// - With optional timezone: Z or +HH:mm or -HH:mm
/// - Expanded years: +YYYYYY or -YYYYYY (6 digits with sign)
fn parse_date_string(s: &str) -> ParseResult {
    let s = s.trim();

    // Check for expanded year format (starts with + or -)
    let (year_str, rest, is_expanded) = if s.starts_with('+') || s.starts_with('-') {
        // Expanded year format: +YYYYYY or -YYYYYY
        if s.len() < 7 {
            return ParseResult::NotMatched;
        }
        let sign = &s[0..1];
        let year_part = &s[1..7];

        // All 6 chars must be digits
        if !year_part.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::NotMatched;
        }

        // Check for invalid -000000 (negative zero)
        if sign == "-" && year_part == "000000" {
            return ParseResult::Invalid;
        }

        (&s[0..7], &s[7..], true)
    } else {
        // Regular 4-digit year
        if s.len() < 4 {
            return ParseResult::NotMatched;
        }
        let year_part = &s[0..4];
        if !year_part.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::NotMatched;
        }
        (year_part, &s[4..], false)
    };

    // Parse year
    let year: i32 = if is_expanded {
        let sign = if year_str.starts_with('-') { -1 } else { 1 };
        let abs_year: i32 = match year_str[1..].parse() {
            Ok(y) => y,
            Err(_) => return ParseResult::Invalid,
        };
        sign * abs_year
    } else {
        match year_str.parse() {
            Ok(y) => y,
            Err(_) => return ParseResult::Invalid,
        }
    };

    // Default values
    let mut month: u32 = 1;
    let mut day: u32 = 1;
    let mut hour: u32 = 0;
    let mut minute: u32 = 0;
    let mut second: u32 = 0;
    let mut millisecond: u32 = 0;
    let mut has_timezone = false;
    let mut tz_offset_minutes: i32 = 0;

    let mut rest = rest;

    // Parse month if present: -MM
    if rest.starts_with('-') {
        if rest.len() < 3 {
            return ParseResult::Invalid;
        }
        let month_str = &rest[1..3];
        if !month_str.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::Invalid;
        }
        month = match month_str.parse() {
            Ok(m) => m,
            Err(_) => return ParseResult::Invalid,
        };
        if month < 1 || month > 12 {
            return ParseResult::Invalid;
        }
        rest = &rest[3..];

        // Parse day if present: -DD
        if rest.starts_with('-') {
            if rest.len() < 3 {
                return ParseResult::Invalid;
            }
            let day_str = &rest[1..3];
            if !day_str.chars().all(|c| c.is_ascii_digit()) {
                return ParseResult::Invalid;
            }
            day = match day_str.parse() {
                Ok(d) => d,
                Err(_) => return ParseResult::Invalid,
            };
            if day < 1 || day > 31 {
                return ParseResult::Invalid;
            }
            rest = &rest[3..];
        }
    }

    // Parse time if present: THH:mm or THH:mm:ss or THH:mm:ss.sss
    if rest.starts_with('T') {
        rest = &rest[1..];

        // HH:mm is required
        if rest.len() < 5 {
            return ParseResult::Invalid;
        }
        let hour_str = &rest[0..2];
        if !hour_str.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::Invalid;
        }
        hour = match hour_str.parse() {
            Ok(h) => h,
            Err(_) => return ParseResult::Invalid,
        };
        if hour > 24 {
            return ParseResult::Invalid;
        }

        if &rest[2..3] != ":" {
            return ParseResult::Invalid;
        }

        let min_str = &rest[3..5];
        if !min_str.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::Invalid;
        }
        minute = match min_str.parse() {
            Ok(m) => m,
            Err(_) => return ParseResult::Invalid,
        };
        if minute > 59 {
            return ParseResult::Invalid;
        }

        rest = &rest[5..];

        // Parse seconds if present: :ss
        if rest.starts_with(':') {
            if rest.len() < 3 {
                return ParseResult::Invalid;
            }
            let sec_str = &rest[1..3];
            if !sec_str.chars().all(|c| c.is_ascii_digit()) {
                return ParseResult::Invalid;
            }
            second = match sec_str.parse() {
                Ok(s) => s,
                Err(_) => return ParseResult::Invalid,
            };
            if second > 59 {
                return ParseResult::Invalid;
            }
            rest = &rest[3..];

            // Parse milliseconds if present: .sss
            if rest.starts_with('.') {
                if rest.len() < 4 {
                    return ParseResult::Invalid;
                }
                let ms_str = &rest[1..4];
                if !ms_str.chars().all(|c| c.is_ascii_digit()) {
                    return ParseResult::Invalid;
                }
                millisecond = match ms_str.parse() {
                    Ok(m) => m,
                    Err(_) => return ParseResult::Invalid,
                };
                rest = &rest[4..];
            }
        }
    }

    // Parse timezone if present: Z or +HH:mm or -HH:mm
    if rest.starts_with('Z') {
        has_timezone = true;
        tz_offset_minutes = 0;
        rest = &rest[1..];
    } else if rest.starts_with('+') || rest.starts_with('-') {
        has_timezone = true;
        let sign = if rest.starts_with('-') { -1 } else { 1 };
        rest = &rest[1..];

        if rest.len() < 5 {
            return ParseResult::Invalid;
        }
        let tz_hour_str = &rest[0..2];
        if !tz_hour_str.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::Invalid;
        }
        let tz_hour: i32 = match tz_hour_str.parse() {
            Ok(h) => h,
            Err(_) => return ParseResult::Invalid,
        };

        if &rest[2..3] != ":" {
            return ParseResult::Invalid;
        }

        let tz_min_str = &rest[3..5];
        if !tz_min_str.chars().all(|c| c.is_ascii_digit()) {
            return ParseResult::Invalid;
        }
        let tz_min: i32 = match tz_min_str.parse() {
            Ok(m) => m,
            Err(_) => return ParseResult::Invalid,
        };

        tz_offset_minutes = sign * (tz_hour * 60 + tz_min);
        rest = &rest[5..];
    }

    // Remaining string should be empty
    if !rest.is_empty() {
        return ParseResult::Invalid;
    }

    // Handle hour 24 special case (means midnight of next day)
    if hour == 24 {
        if minute != 0 || second != 0 || millisecond != 0 {
            return ParseResult::Invalid;
        }
        // We'll handle this by adding a day after construction
    }

    // Construct the datetime
    let hour_for_construction = if hour == 24 { 0 } else { hour };

    // Date-only forms (no T) are interpreted as UTC
    // Date-time forms without timezone are interpreted as local time
    let has_time = s.contains('T');
    let treat_as_utc = !has_time || has_timezone;

    if treat_as_utc {
        // Build in UTC
        let utc_result =
            Utc.with_ymd_and_hms(year, month, day, hour_for_construction, minute, second);
        let utc_dt = match utc_result {
            LocalResult::Single(dt) => dt,
            LocalResult::Ambiguous(dt, _) => dt,
            LocalResult::None => return ParseResult::Invalid,
        };

        let utc_dt = match utc_dt.with_nanosecond(millisecond * 1_000_000) {
            Some(dt) => dt,
            None => return ParseResult::Invalid,
        };

        // Handle hour 24 by adding a day
        let utc_dt = if hour == 24 {
            utc_dt + chrono::Duration::days(1)
        } else {
            utc_dt
        };

        // Apply timezone offset (subtract it since offset is "local - UTC")
        let utc_dt = utc_dt - chrono::Duration::minutes(tz_offset_minutes as i64);

        ParseResult::Parsed(utc_dt.with_timezone(&Local))
    } else {
        // Build in local time
        let local_result =
            Local.with_ymd_and_hms(year, month, day, hour_for_construction, minute, second);
        let local_dt = match local_result {
            LocalResult::Single(dt) => dt,
            LocalResult::Ambiguous(dt, _) => dt,
            LocalResult::None => return ParseResult::Invalid,
        };

        let local_dt = match local_dt.with_nanosecond(millisecond * 1_000_000) {
            Some(dt) => dt,
            None => return ParseResult::Invalid,
        };

        // Handle hour 24 by adding a day
        let local_dt = if hour == 24 {
            local_dt + chrono::Duration::days(1)
        } else {
            local_dt
        };

        ParseResult::Parsed(local_dt)
    }
}
