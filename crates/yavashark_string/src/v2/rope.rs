use std::fmt::Debug;
use std::ops::Deref;
use std::ptr::NonNull;
use crate::v2::small_pointer::{Gc, SmallPointer};
use crate::v2::YSString;





#[derive(Clone)]
pub struct RopeString {
    from: u32,
    to: u32,
    a: Gc<YSString>,
    b: Gc<YSString>,
}

impl Debug for RopeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = self.a.deref();
        let b = self.b.deref();

        let a = a.to_ref();
        let b = b.to_ref();



        Ok(())
    }
}

impl RopeString {
    pub fn new(a: Gc<YSString>, b: Gc<YSString>) -> Self {
        let a_len = a.len();
        let b_len = b.len();

        Self {
            from: 0,
            to: a_len + b_len,
            a,
            b,
        }
    }


    pub const fn len(&self) -> u32 {
        self.to - self.from
    }

    pub fn raw_len(&self) -> u32 {
        self.a.len() + self.b.len()
    }

    pub fn slice(mut self, start: u32, end: u32) -> Option<Result<Self, YSString>> {
        if start > end || end > self.len() {
            return None;
        }

        self.from += start;
        self.to = self.from + (end - start);

        Some(self.shake())
    }


    pub fn shake(mut self) -> Result<Self, YSString> {
        if self.from == 0 && self.to == self.raw_len() {
            return Ok(self);
        }

        if self.from == self.to {
            return Err(YSString::new())
        }

        if self.from > self.a.len() {
            // we can safely drop a, since it's not used at all (we return a slice of b)

            let start = self.from - self.a.len();
            let end = self.to - self.a.len();

            let b = (*self.b).clone();

            return Err(b.slice(start, end).expect("can't ever happen"));
        }

        if self.to <= self.a.len() {
            // we can safely drop b, since it's not used at all (we return a slice of a)

            let start = self.from;
            let end = self.to;

            let a = (*self.a).clone();

            return Err(a.slice(start, end).expect("can't ever happen"));
        }

        Ok(self)

    }


}
