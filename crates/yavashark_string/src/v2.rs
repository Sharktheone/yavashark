mod heap;
mod inline;
mod rope;
mod small_pointer;
mod reference;

use crate::v2::heap::HeapString;
use crate::v2::inline::{InlineAscii, InlineWtf16};
use crate::v2::rope::{RopeString, RopeStringRef};
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::mem::{size_of, ManuallyDrop};
use std::ptr;
use crate::v2::reference::YSStringRef;
use crate::v2::small_pointer::Gc;

pub struct YSString {
    inner: UnsafeCell<Inner>,
}

impl Debug for YSString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = unsafe { &*self.inner.get() };

        match inner {
            Inner::Heap(heap) => heap.fmt(f),
            Inner::InlineAscii(inline) => inline.fmt(f),
            Inner::InlineWtf16(inline) => inline.fmt(f),
            Inner::Rope(rope) => rope.fmt(f),
        }
    }
}

const _: [(); 24] = [(); size_of::<YSString>()];

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

impl Default for YSString {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for YSString {
    fn clone(&self) -> Self {
        let inner = unsafe { &*self.inner.get() };

        match inner {
            Inner::Heap(heap) => heap.clone().into(),
            Inner::InlineAscii(inline) => (*inline).into(),
            Inner::InlineWtf16(inline) => (*inline).into(),
            Inner::Rope(rope) => rope.clone().into(),
        }
    }
}

impl YSString {
    fn new() -> Self {
        InlineAscii::new().into()
    }

    const fn from_inner(inner: Inner) -> Self {
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
        InlineAscii::try_from_str(s)
            .map_or_else(
                || HeapString::from_str(s).into(),
                Into::into
            )
    }

    pub fn from_wtf16(s: &[u16]) -> Self {
        InlineWtf16::try_from_slice(s)
            .map_or_else(
                || HeapString::from_wtf16(s).into(),
                Into::into
            )
    }

    pub fn len(&self) -> u32 {
        let inner = unsafe { &*self.inner.get() };

        match inner {
            Inner::Heap(heap) => heap.len(),
            Inner::InlineAscii(inline) => inline.len(),
            Inner::InlineWtf16(inline) => inline.len(),
            Inner::Rope(rope) => rope.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn to_ref(&self) -> YSStringRef {
        let copied = unsafe {
            ManuallyDrop::new(
                ptr::read(self)
            )
        };

        YSStringRef {
            inner: copied,
            _marker: std::marker::PhantomData,
        }
    }

    fn as_rope_ref(&self) -> RopableStringRef {
        let inner = unsafe { &*self.inner.get() };

        match inner {
            Inner::Heap(heap) => heap.as_rope_ref(),
            Inner::InlineAscii(inline) => RopableStringRef::Ascii(inline.as_ref()),
            Inner::InlineWtf16(inline) => RopableStringRef::Wtf16(&[]), //TODO
            Inner::Rope(rope) => RopableStringRef::Rope(RopeStringRef::new(rope)),
        }
    }

    pub fn slice(self, start: u32, end: u32) -> Option<Self> {
        let inner = self.inner.into_inner();

        match inner {
            Inner::Heap(heap) => heap.slice(start, end).ok().map(Into::into),
            Inner::InlineAscii(inline) => inline.slice(start, end).map(Into::into),
            Inner::InlineWtf16(inline) => inline.slice(start, end).map(Into::into),
            Inner::Rope(rope) => rope.slice(start, end)
                .map(|r| match r {
                        Ok(rope) => rope.into(),
                        Err(string) => string,
                    }
                )
        }
    }

    pub fn concat(a: Gc<Self>, b: Gc<Self>) -> Self {
        RopeString::new(a, b).into()
    }


}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Ascii,
    Wtf16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringRef<'a> {
    Ascii(&'a str),
    Wtf16(&'a [u16]),
}

#[derive(Debug, Clone, Copy)]
enum RopableStringRef<'a> {
    Ascii(&'a str),
    Wtf16(&'a [u16]),
    Rope(RopeStringRef<'a>),
}
