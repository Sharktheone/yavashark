use crate::v2::small_pointer::{Gc, SmallPointer};
use crate::v2::{RopableStringRef, StringRef, Type, YSString};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[derive(Copy, Clone)]
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
    
    pub fn get_type(&self) -> Type {
        let a_ty = self.a.deref().get_type();
        let b_ty = self.b.deref().get_type();

        if a_ty == Type::Wtf16 || b_ty == Type::Wtf16 {
            Type::Wtf16
        } else {
            Type::Ascii
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
            return Err(YSString::new());
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

    pub fn for_each_elem<'a, F>(
        &'a self,
        f: &mut impl FnMut(StringRef<'a>) -> Option<F>,
    ) -> Option<F> {
        let a_len = self.a.len();

        if self.from < a_len {
            //TODO: should we make this check? (If not we have to change some things in the RopePath below
            match self.a.as_rope_ref() {
                RopableStringRef::Ascii(a) => {
                    let start = self.from as usize;
                    let end = self.to.min(a_len) as usize;

                    if let Some(result) = f(StringRef::Ascii(&a[start..end])) {
                        return Some(result);
                    }
                }
                RopableStringRef::Wtf16(w) => {
                    let start = self.from as usize;
                    let end = self.to.min(a_len) as usize;

                    if let Some(result) = f(StringRef::Wtf16(&w[start..end])) {
                        return Some(result);
                    }
                }
                RopableStringRef::Rope(mut rope) => {
                    rope.from += self.from;
                    rope.to = rope.from + (self.to - self.from).min(a_len - self.from);

                    if let Some(result) = rope.as_ref().for_each_elem(f) {
                        return Some(result);
                    }
                }
            }
        }

        if self.to > a_len {
            match self.b.as_rope_ref() {
                RopableStringRef::Ascii(b) => {
                    let start = (self.from - a_len) as usize;
                    let end = (self.to - a_len) as usize;

                    if let Some(result) = f(StringRef::Ascii(&b[start..end])) {
                        return Some(result);
                    }
                }
                RopableStringRef::Wtf16(w) => {
                    let start = (self.from - a_len) as usize;
                    let end = (self.to - a_len) as usize;

                    if let Some(result) = f(StringRef::Wtf16(&w[start..end])) {
                        return Some(result);
                    }
                }
                RopableStringRef::Rope(mut rope) => {
                    rope.from += self.from - a_len;
                    rope.to =
                        rope.from + (self.to - self.from).min(self.b.len() - (self.from - a_len));

                    if let Some(result) = rope.as_ref().for_each_elem(f) {
                        return Some(result);
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RopeStringRef<'a> {
    rope: RopeString,
    _marker: PhantomData<&'a RopeString>,
}

impl<'a> RopeStringRef<'a> {
    pub const fn new(rope: &'a RopeString) -> Self {
        Self {
            rope: *rope,
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> &'a RopeString {
        unsafe { mem::transmute::<&RopeString, &'a RopeString>(&self.rope) }
    }
    
    
    pub fn write_to_utf16_buffer(&self, buffer: &mut [u16], mut offset: usize) {
        self.as_ref().for_each_elem(&mut |elem| {
            match elem {
                StringRef::Ascii(s) => {
                    for (i, &b) in s.as_bytes().iter().enumerate() {
                        buffer[offset + i] = b as u16;
                    }
                    
                    offset += s.len();
                }
                StringRef::Wtf16(w) => {
                    for (i, &b) in w.iter().enumerate() {
                        buffer[offset + i] = b;
                    }
                    
                    offset += w.len();
                }
            }

            Some(())
        });
    }
    
    pub fn write_to_ascii_buffer(&self, buffer: &mut [u8], mut offset: usize) {
        self.as_ref().for_each_elem(&mut |elem| {
            match elem {
                StringRef::Ascii(s) => {
                    for (i, &b) in s.as_bytes().iter().enumerate() {
                        buffer[offset + i] = b;
                    }
                    
                    offset += s.len();
                }
                StringRef::Wtf16(w) => {
                    for (i, &b) in w.iter().enumerate() {
                        buffer[offset + i] = b as u8;
                    }
                    
                    offset += w.len();
                }
            }

            Some(())
        });
    }
}

impl Deref for RopeStringRef<'_> {
    type Target = RopeString;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for RopeStringRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { mem::transmute::<&mut RopeString, &mut RopeString>(&mut self.rope) }
    }
}
