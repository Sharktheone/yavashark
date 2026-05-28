use crate::v2::YSString;

pub struct RopeString {
    from: u32,
    to: u32,
    inner: Box<RopeStringInner>,
}

struct RopeStringInner {
    a: YSString,
    b: YSString,
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
