mod heap;
mod inline;
mod rope;

use std::cell::UnsafeCell;
use crate::v2::heap::HeapString;
use crate::v2::inline::{InlineAscii, InlineWtf16};
use crate::v2::rope::RopeString;

pub struct YSString {
    inner: UnsafeCell<Inner>,
}

enum Inner {
    Heap(HeapString),
    InlineAscii(InlineAscii),
    InlineWtf16(InlineWtf16),
    Rope(RopeString),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Ascii,
    Wtf16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringRef<'a> {
    Ascii(&'a str),
    Wtf16(&'a [u16]),
}

