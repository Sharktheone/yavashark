use std::ptr::NonNull;

use crate::v2::YSString;


type Gc<T> = NonNull<T>;

pub struct RopeString {
    from: u32,
    to: u32,
    a: Gc<YSString>,
    b: Gc<YSString>,
}

impl RopeString {
    pub fn len(&self) -> u32 {
        self.to - self.from
    }

    pub fn slice(mut self, start: u32, end: u32) -> Option<Self> {
        if start > end || end > self.len() {
            return None;
        }

        self.from += start;
        self.to = self.from + (end - start);

        Some(self)
    }
}
