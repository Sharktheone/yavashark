#![allow(unused)]

pub(crate) mod uz;
pub(crate) mod smallvec;

use std::cell::UnsafeCell;
use std::num::{NonZero, NonZeroI32};
use std::ptr::NonNull;
use std::rc::Rc;

pub struct YSString {
    inner: UnsafeCell<InnerString>
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SmallLen {
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

enum InnerString {
    Inline {
        len: SmallLen,
        data: [u8; 23],
    },
    Static(&'static str),
    Owned(OwnedString),
    Rc(Rc<str>),
    Rope(RopeStr),
}







enum InnerRopeStr {
    // Static(&'static str),
    // Rc(Rc<str>),
    Rope(Rc<RopeStr>),
}

struct RopeStr {
    left: InnerRopeStr,
    // right: InnerRopeStr,
}

struct OwnedString {
    // inner: SmallerVec<u8>,
    // data: RawVec<u8>,
}




// impl SmallVecLenCap {
//     pub fn new(len: usize, cap: usize) -> Option<Self> {
//         if len > 0x7F_FF_FF_FF || cap > 0x7F_FF_FF_FF {
//             return None;
//         }
//         
//         
//         let len: [u8; 8] = len.to_be_bytes();
//         let cap: [u8; 8] = cap.to_be_bytes();
//         
//         Self {
//             len,
//             cap,
//         }
//     }
//     
//     pub fn len(&self) -> usize {
//         usize::from_be_bytes(self.len)
//     }
//     
//     pub fn cap(&self) -> usize {
//         usize::from_be_bytes(self.cap)
//     }
//     
// }



#[repr(u8)]
enum Signed {
    Positive,
    NoSign,
    Negative,
}


#[repr(packed)]
struct BigUInt {
    inner: Vec<u32>,
}


#[repr(packed)]
struct BigInt {
    inner: BigUInt,
    sign: Signed,
}

pub enum Value {
    Null,
    Undefined,
    Number(f64),
    String(YSString),
    Boolean(bool),
    Object(Box<String>),
    Symbol(Box<String>),
    BigInt(BigInt),
}

enum StrStr {
    One(String),
    Two(String),
}


#[test]
fn str_size() {
    dbg!(std::mem::size_of::<Value>());
    
    dbg!(std::mem::size_of::<BigInt>());
    dbg!(std::mem::size_of::<BigUInt>());
    dbg!(std::mem::size_of::<String>());
    dbg!(std::mem::size_of::<Option<String>>());
    dbg!(std::mem::size_of::<StrStr>());
    
    let str = "Hello, World!".to_owned();
    
    let str = str.into_boxed_str();
    
    let rcd: Rc<str> = Rc::from(str);
    
    let rcd2 = rcd.clone();
    
    dbg!(rcd, rcd2);
    
    
    // assert_eq!(std::mem::size_of::<InnerString>(), std::mem::size_of::<String>());
}