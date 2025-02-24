#![allow(unused)]

pub(crate) mod uz;
pub(crate) mod smallvec;
mod smallstring;

use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::cmp::min;
use std::num::{NonZero, NonZeroI32};
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;
use crate::smallstring::SmallString;
use crate::smallvec::SmallVecLenCap;
use crate::uz::{UsizeSmall, UZ_BYTES};

pub struct YSString {
    inner: UnsafeCell<InnerString>
}


enum InnerString {
    Inline(InlineString),
    Static(StaticStr),
    Owned(SmallString),
    Rc(RcStr),
    Rope(RopeStr),
}

#[repr(packed)]
struct StaticStr {
    str: &'static str,
    begin: [u8; UZ_BYTES],
}

struct RcStr {
    rc: Rc<str>,
}

#[repr(packed)]
pub struct InlineString {
    len: InlineLen,
    data: [u8; 23],
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InlineLen {
    Empty = 0,
    Len1,
    Len2,
    Len3,
    Len4,
    Len5,
    Len6,
    Len7,
    Len8,
    Len9,
    Len10,
    Len11,
    Len12,
    Len13,
    Len14,
    Len15,
    Len16,
    Len17,
    Len18,
    Len19,
    Len20,
    Len21,
    Len22,
    Len23,
}

struct RopeStr {
    left: Rc<YSString>,
    right: Rc<YSString>,
}



impl StaticStr {
    const fn begin(&self) -> [u8; UZ_BYTES] {
        self.begin
    }
    
    const fn len(&self) -> usize {
        self.str.len()
    }
    
    const fn as_str(&self) -> &str {
        self.str
    }
}

impl RcStr {
    fn begin(&self) -> [u8; UZ_BYTES] {
        let mut begin = [0; UZ_BYTES];
        
        begin.copy_from_slice(&self.rc.as_bytes()[0..min(UZ_BYTES, self.rc.len())]);
        
        begin
    }
    
    fn len(&self) -> usize {
        self.rc.len()
    }
    
    fn as_str(&self) -> &str {
        &self.rc
    }
}

impl InlineString {
    const fn begin(&self) -> [u8; UZ_BYTES] {
        let mut begin = [0; UZ_BYTES];
        
        begin[0] = self.len as u8;
        
        begin
    }
    
    const fn len(&self) -> usize {
        self.len as usize
    }
    
    fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.data[0..self.len()])
        }
    }
}


impl InlineLen {
    
    #[must_use] 
    pub const fn from_usize(len: usize) -> Option<Self> {
        Some(match len {
            0 => Self::Empty,
            1 => Self::Len1,
            2 => Self::Len2,
            3 => Self::Len3,
            4 => Self::Len4,
            5 => Self::Len5,
            6 => Self::Len6,
            7 => Self::Len7,
            8 => Self::Len8,
            9 => Self::Len9,
            10 => Self::Len10,
            11 => Self::Len11,
            12 => Self::Len12,
            13 => Self::Len13,
            14 => Self::Len14,
            15 => Self::Len15,
            16 => Self::Len16,
            17 => Self::Len17,
            18 => Self::Len18,
            19 => Self::Len19,
            20 => Self::Len20,
            21 => Self::Len21,
            22 => Self::Len22,
            23 => Self::Len23,
            _ => return None,
        })
    }
    
}

impl RopeStr {
    // fn begin(&self) -> [u8; UZ_BYTES] {
    //     self.begin
    // }
    
    fn len(&self) -> usize {
        self.left.len() + self.right.len()
    }
    
    fn as_string_opt_rope(&self) -> String {
        let mut str = String::with_capacity(self.len());
        
        if Rc::strong_count(&self.left) == 1 {
            str.push_str(&self.left.as_str_no_rope_fix()); // if we only have one reference, we can avoid cloning
        } else {
            str.push_str(self.left.as_str());
        }
        
        if Rc::strong_count(&self.right) == 1 {
            str.push_str(&self.right.as_str_no_rope_fix()); // if we only have one reference, we can avoid cloning
        } else {
            str.push_str(self.right.as_str());
        }
        
        
        str
    }
    
    fn as_string(&self) -> String {
        let mut str = String::with_capacity(self.len());
        
        str.push_str(self.left.as_str());
        str.push_str(self.right.as_str());
        
        str
    }
}


impl YSString {
    fn inner(&self) -> &InnerString {
        unsafe {
            &*self.inner.get()
        }
    }
    
    #[allow(clippy::mut_from_ref)]
    unsafe fn inner_mut(&self) -> &mut InnerString {
        unsafe {
            &mut *self.inner.get()
        }
    }
    
    pub fn len(&self) -> usize {
        match self.inner() {
            InnerString::Inline(inline) => inline.len(),
            InnerString::Static(static_str) => static_str.len(),
            InnerString::Owned(owned) => owned.len(),
            InnerString::Rc(rc) => rc.len(),
            InnerString::Rope(rope) => rope.len(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn as_str(&self) -> &str {
        match self.inner() {
            InnerString::Inline(inline) => inline.as_str(),
            InnerString::Static(static_str) => static_str.as_str(),
            InnerString::Owned(owned) => owned,
            InnerString::Rc(rc) => rc.as_str(),
            InnerString::Rope(rope) => {
                let str = rope.as_string_opt_rope(); // since we drop the RopeStr afterward we don't need to fix the rope
                
                let inner = unsafe { self.inner_mut() };
                
                let str = SmallString::from_string(str).unwrap_or_default();
                
                *inner = InnerString::Owned(str);
                
                let InnerString::Owned(str) = inner else { 
                    return "" // unreachable, but don't crash
                };
                
                str
            },
        }
    }
    
    fn as_str_no_rope_fix(&self) -> Cow<str> {
        Cow::Borrowed(match self.inner() {
            InnerString::Inline(inline) => inline.as_str(),
            InnerString::Static(static_str) => static_str.as_str(),
            InnerString::Owned(owned) => owned,
            InnerString::Rc(rc) => rc.as_str(),
            InnerString::Rope(rope) => return Cow::Owned(rope.as_string_opt_rope()) // we don't need to fix the rope here
        })
    }
}




#[test]
fn str_size() {
    dbg!(size_of::<InnerString>());
    dbg!(align_of::<InnerString>());
    dbg!(size_of::<SmallVecLenCap>());
    dbg!(size_of::<SmallString>());

    dbg!(size_of::<String>());
    dbg!(size_of::<Option<String>>());
    
    let str = "Hello, World!".to_owned();
    
    let str = str.into_boxed_str();
    
    let rcd: Rc<str> = Rc::from(str);
    
    let rcd2 = rcd.clone();
    
    dbg!(rcd, rcd2);
    
    
    // assert_eq!(std::mem::size_of::<InnerString>(), std::mem::size_of::<String>());
}