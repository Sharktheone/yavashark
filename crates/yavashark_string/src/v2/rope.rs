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

