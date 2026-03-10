#![allow(unused)]

use std::ptr;
use std::ptr::NonNull;

/// we have 1 sign bit and 52 exponent bits to store data.
/// We need to have the following values:
/// f64 - default
/// null
/// undefined
/// true
/// false
/// inline string (6bytes)
/// string (ptr)
/// symbol (ptr)
/// object (ptr)
/// bit int (ptr)
/// int32
///            0000 0000 0000 0000 0000 0000 0000 0000 0000 .. 0000
/// +inf       0111 1111 1111 0000 0000 0000 0000 0000 0000 .. 0000
/// -inf       1111 1111 1111 0000 0000 0000 0000 0000 0000 .. 0000
/// NaN        0111 1111 1111 1000 0000 0000 0000 0000 0000 .. 0000
/// Int32      0111 1111 1111 1001 0000 0000 0000 0000 IIII .. IIII
/// False      0111 1111 1111 1001 0100 0000 0000 0000 0000 .. 0000
/// True       0111 1111 1111 1001 0100 0000 0000 0000 0000 .. 0001
/// Null       0111 1111 1111 1001 1000 0000 0000 0000 0000 .. 0000
/// Undefined  0111 1111 1111 1001 1000 0000 0000 0000 0000 .. 0001
/// String     0111 1111 1111 1010 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// InlineStr  0111 1111 1111 1011 DDDD DDDD DDDD DDDD DDDD .. DDDD
/// Object     0111 1111 1111 1100 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// Symbol     0111 1111 1111 1101 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// BigInt     0111 1111 1111 1110 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// Float64    Any other value.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ValueInner {
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
    half: u32,
    #[cfg(target_pointer_width = "16")]
    ptr_pad: u16,
    ptr: *const (),
}

pub enum ValueVariant {
    Number(f64),
    Null,
    Undefined,
    Bool(bool),
    InlineString([u8; 6]),
    String(NonNull<()>),
    Symbol(NonNull<()>),
    Object(NonNull<()>),
    BigInt(NonNull<()>),
    Integer(i32),
}

mod bits {
    use std::ptr::NonNull;

    const MASK_NAN: u64 = 0x7FF8_0000_0000_0000;

    const MASK_KIND: u64 = MASK_NAN | 0xF_0000_0000_0000;
    const MASK_KIND_OTHER: u64 = MASK_NAN | 0xF_C000_0000_0000;
    const MASK_STRING: u64 = MASK_NAN | 0xE_0000_0000_0000;

    const TAG_INF: u64 = 0x0_0000_0000_0000;
    const TAG_NAN: u64 = 0x8_0000_0000_0000;
    const TAG_INT32: u64 = 0x9_0000_0000_0000;
    const TAG_BOOLEAN: u64 = 0x9_4000_0000_0000;
    const TAG_NULL_UNDEF: u64 = 0x9_8000_0000_0000;
    const TAG_INLINE_STRING: u64 = 0xA_0000_0000_0000;
    const TAG_STRING_OWNED: u64 = 0xB_0000_0000_0000;
    const TAG_OBJECT: u64 = 0xC_0000_0000_0000;
    const TAG_SYMBOL: u64 = 0xD_0000_0000_0000;
    const TAG_BIGINT: u64 = 0xE_0000_0000_0000;

    const MASK_INT32: u64 = MASK_NAN | TAG_INT32;
    const MASK_BOOLEAN: u64 = MASK_NAN | TAG_BOOLEAN;
    const MASK_NULL_UNDEF: u64 = MASK_NAN | TAG_NULL_UNDEF;
    const MASK_INLINE_STRING: u64 = MASK_NAN | TAG_INLINE_STRING;
    const MASK_STRING_OWNED: u64 = MASK_NAN | TAG_STRING_OWNED;
    const MASK_OBJECT: u64 = MASK_NAN | TAG_OBJECT;
    const MASK_SYMBOL: u64 = MASK_NAN | TAG_SYMBOL;
    const MASK_BIGINT: u64 = MASK_NAN | TAG_BIGINT;

    const MASK_INT32_VALUE: u64 = 0x0000_0000_FFFF_FFFF;
    const MASK_BOOLEAN_VALUE: u64 = 0x0000_0000_0000_0001;
    const MASK_48BIT_VALUE: u64 = 0x0000_FFFF_FFFF_FFFF;

    pub(crate) const VALUE_NULL: u64 = TAG_NULL_UNDEF;
    pub(crate) const VALUE_UNDEFINED: u64 = TAG_NULL_UNDEF | 0x1;
    const VALUE_FALSE: u64 = TAG_BOOLEAN;
    const VALUE_TRUE: u64 = TAG_BOOLEAN | 0x1;

    pub const fn is_int32(value: u64) -> bool {
        (value & MASK_KIND_OTHER) == TAG_INT32
    }

    pub const fn is_boolean(value: u64) -> bool {
        (value & MASK_BOOLEAN) == TAG_BOOLEAN
    }

    pub const fn is_null_or_undefined(value: u64) -> bool {
        (value & MASK_NULL_UNDEF) == TAG_NULL_UNDEF
    }

    pub const fn is_null(value: u64) -> bool {
        value == VALUE_NULL
    }

    pub const fn is_undefined(value: u64) -> bool {
        value == VALUE_UNDEFINED
    }

    pub const fn is_inline_string(value: u64) -> bool {
        (value & MASK_KIND) == TAG_INLINE_STRING
    }

    pub const fn is_string_owned(value: u64) -> bool {
        (value & MASK_KIND) == TAG_STRING_OWNED
    }

    pub const fn is_string(value: u64) -> bool {
        (value & MASK_STRING) == TAG_INLINE_STRING
    }

    pub const fn is_object(value: u64) -> bool {
        (value & MASK_KIND) == TAG_OBJECT
    }

    pub const fn is_symbol(value: u64) -> bool {
        (value & MASK_KIND) == TAG_SYMBOL
    }

    pub const fn is_bigint(value: u64) -> bool {
        (value & MASK_KIND) == TAG_BIGINT
    }

    pub const fn is_number(value: u64) -> bool {
        (value & MASK_NAN) != MASK_NAN
            || (value & MASK_KIND) == TAG_INF
            || (value & MASK_KIND) == TAG_NAN
    }

    pub const fn encode_int32(value: i32) -> u64 {
        (value as u64) | TAG_INT32
    }

    pub const fn encode_boolean(value: bool) -> u64 {
        value as u64 | TAG_BOOLEAN
    }

    pub const fn encode_f64(value: f64) -> u64 {
        if value.is_nan() {
            f64::NAN.to_bits()
        } else {
            value.to_bits()
        }
    }

    pub fn encode_pointer(ptr: NonNull<()>, tag: u64) -> u64 {
        let value = ptr.addr().get() as u64;
        (value & MASK_48BIT_VALUE) | tag
    }

    pub fn encode_inline_string(bytes: [u8; 6]) -> u64 {
        let mut padded = [0; 8];

        padded[2..8].copy_from_slice(&bytes);

        let value = u64::from_le_bytes(padded);

        value | TAG_INLINE_STRING
    }

    pub const fn decode_pointer(value: u64) -> usize {
        (value & MASK_48BIT_VALUE) as usize
    }

    pub const fn decode_int32(value: u64) -> i32 {
        value as i32
    }

    pub const fn decode_boolean(value: u64) -> bool {
        (value & MASK_BOOLEAN_VALUE) != 0
    }
}

impl ValueInner {
    const fn from_bits(bits: u64) -> Self {
        Self {
            #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
            half: (bits >> 32) as u32,
            #[cfg(target_pointer_width = "16")]
            ptr_pad: (bits >> 48) as u16,
            ptr: ptr::without_provenance_mut(bits as usize),
        }
    }

    fn with_provenance(ptr: NonNull<()>, addr: u64) -> Self {
        Self {
            #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
            half: (addr >> 32) as u32,
            #[cfg(target_pointer_width = "16")]
            ptr_pad: (addr >> 48) as u16,
            ptr: ptr.cast::<()>().as_ptr().with_addr(addr as usize),
        }
    }

    fn value(self) -> u64 {
        //TODO: should this be a transmute?
        let mut bits = self.ptr.addr() as u64;

        #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
        {
            bits |= (self.half as u64) << 32;
        }

        #[cfg(target_pointer_width = "16")]
        {
            bits |= (self.ptr_pad as u64) << 48;
        }

        bits
    }

    pub const fn from_f64(value: f64) -> Self {
        let bits = bits::encode_f64(value);

        Self::from_bits(bits)
    }

    pub const fn from_int32(value: i32) -> Self {
        let bits = bits::encode_int32(value);

        Self::from_bits(bits)
    }

    pub const fn from_bool(value: bool) -> Self {
        let bits = bits::encode_boolean(value);

        Self::from_bits(bits)
    }

    pub const fn null() -> Self {
        Self::from_bits(bits::VALUE_NULL)
    }

    pub const fn undefined() -> Self {
        Self::from_bits(bits::VALUE_UNDEFINED)
    }

    pub fn from_inline_string(bytes: [u8; 6]) -> Self {
        let bits = bits::encode_inline_string(bytes);

        Self::from_bits(bits)
    }

    pub unsafe fn from_ptr(ptr: NonNull<()>) -> Self {
        Self {
            #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
            half: 0,
            #[cfg(target_pointer_width = "16")]
            ptr_pad: 0,
            ptr: ptr.as_ptr(),
        }
    }

    pub fn is_f64(self) -> bool {
        bits::is_number(self.value())
    }

    pub fn is_int32(self) -> bool {
        bits::is_int32(self.value())
    }

    pub fn is_bool(self) -> bool {
        bits::is_boolean(self.value())
    }

    pub fn is_null(self) -> bool {
        bits::is_null(self.value())
    }

    pub fn is_undefined(self) -> bool {
        bits::is_undefined(self.value())
    }

    pub fn is_inline_string(self) -> bool {
        bits::is_inline_string(self.value())
    }

    pub fn is_string_owned(self) -> bool {
        bits::is_string_owned(self.value())
    }

    pub fn is_string(self) -> bool {
        bits::is_string(self.value())
    }

    pub fn is_object(self) -> bool {
        bits::is_object(self.value())
    }

    pub fn is_symbol(self) -> bool {
        bits::is_symbol(self.value())
    }

    pub fn is_bigint(self) -> bool {
        bits::is_bigint(self.value())
    }

    pub fn as_f64(self) -> Option<f64> {
        if self.is_f64() {
            Some(f64::from_bits(self.value()))
        } else {
            None
        }
    }

    pub fn as_int32(self) -> Option<i32> {
        if self.is_int32() {
            Some(bits::decode_int32(self.value()))
        } else {
            None
        }
    }

    pub fn as_bool(self) -> Option<bool> {
        if self.is_bool() {
            Some(bits::decode_boolean(self.value()))
        } else {
            None
        }
    }

    pub unsafe fn unsafe_assume_pointer(self) -> NonNull<()> {
        NonNull::new_unchecked(self.ptr as *mut ())
    }

    pub unsafe fn as_pointer_unchecked(self) -> NonNull<()> {
        let addr = bits::decode_pointer(self.value());
        unsafe {
            NonNull::new_unchecked(self.ptr.with_addr(addr) as *mut _)
        }
    }

    pub fn as_inline_string(self) -> Option<[u8; 6]> {
        if self.is_inline_string() {
            let bits = self.value();
            let bytes = bits.to_le_bytes();
            let mut result = [0; 6];
            result.copy_from_slice(&bytes[2..8]);
            Some(result)
        } else {
            None
        }
    }

    pub fn as_string_owned(self) -> Option<NonNull<()>> {
        if self.is_string_owned() {
            unsafe { Some(self.as_pointer_unchecked()) }
        } else {
            None
        }
    }

    pub fn as_object(self) -> Option<NonNull<()>> {
        if self.is_object() {
            unsafe { Some(self.as_pointer_unchecked()) }
        } else {
            None
        }
    }

    pub fn as_symbol(self) -> Option<NonNull<()>> {
        if self.is_symbol() {
            unsafe { Some(self.as_pointer_unchecked()) }
        } else {
            None
        }
    }

    pub fn as_bigint(self) -> Option<NonNull<()>> {
        if self.is_bigint() {
            unsafe { Some(self.as_pointer_unchecked()) }
        } else {
            None
        }
    }

}
