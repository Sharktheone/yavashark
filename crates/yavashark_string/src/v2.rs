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


impl From<HeapString> for YSString {
    fn from(heap: HeapString) -> Self {
        Self::from_inner(Inner::Heap(heap))
    }
}

impl From<InlineAscii> for YSString {
    fn from(inline: InlineAscii) -> Self {
        Self::from_inner(Inner::InlineAscii(inline))
    }
}


impl From<InlineWtf16> for YSString {
    fn from(inline: InlineWtf16) -> Self {
        Self::from_inner(Inner::InlineWtf16(inline))
    }
}

impl From<RopeString> for YSString {
    fn from(rope: RopeString) -> Self {
        Self::from_inner(Inner::Rope(rope))
    }
}



impl YSString {
    fn from_inner(inner: Inner) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }
    
    pub fn from_static_ascii(s: &'static str) -> Self {
        HeapString::from_static_ascii(s).into()
    }
    
    pub fn from_static_wtf16(s: &'static [u16]) -> Self {
        HeapString::from_static_wtf16(s).into()
    }
    
    pub fn from_ascii(s: &str) -> Self {
        if let Some(inline) = InlineAscii::try_from_str(s) {
            inline.into()
        } else {
            HeapString::from_str(s).into()
        }
    }

    pub fn from_wtf16(s: &[u16]) -> Self {
        if let Some(inline) = InlineWtf16::try_from_slice(s) {
            inline.into()
        } else {
            HeapString::from_wtf16(s).into()
        }
    }
    
    
    pub fn slice(self, start: u32, end: u32) -> Option<Self> {
        let inner = self.inner.into_inner();
        
        match inner {
            Inner::Heap(heap) => heap.slice(start, end).ok().map(Into::into),
            Inner::InlineAscii(inline) => inline.slice(start, end).map(Into::into),
            Inner::InlineWtf16(inline) => inline.slice(start, end).map(Into::into),
            Inner::Rope(rope) => rope.slice(start, end).map(Into::into),
        }
    }
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

