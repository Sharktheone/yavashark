#![allow(dead_code)]

use std::ptr::NonNull;

/// Int32      0111 1111 1111 1001 0000 0000 0000 0000 IIII .. IIII
///
/// Imm        0111 1111 1111 1010 0000 0000 0000 0000 iiii .. iiii
/// False      0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0000
/// True       0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0001
/// Null       0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0010
/// Undefined  0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0011
/// TheHole    0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0100
///
/// Unused     0111 1111 1111 1011 0000 0000 0000 0000 0000 .. 0000
///
/// InlineStr  0111 1111 1111 1100 DDDD DDDD DDDD DDDD DDDD .. DDDD => Inline marker first bit and second + third bit in the 4th group
/// BigInt48   0111 1111 1111 1101 BBBB BBBB BBBB BBBB BBBB .. BBBB
///
/// Object     1111 1111 1111 1001 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// Symbol     1111 1111 1111 1010 PPPP PPPP PPPP PPPP PPPP .. PPPP
///
/// Unused     1111 1111 1111 1011 0000 0000 0000 0000 0000 .. 0000
///
/// String     1111 1111 1111 1100 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// BigInt     1111 1111 1111 1110 PPPP PPPP PPPP PPPP PPPP .. PPPP
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ValueInner {
    val: u64,
}

mod bits {
    use std::num::NonZeroUsize;
    use std::ptr::NonNull;

    // 0xFFFF FFFF FFFF FFFF
    pub const INT32_TAG: u64 = 0x7FF9_0000_0000_0000;

    pub const IMM_TAG: u64 = 0x7FFA_0000_0000_0000;
    pub const BOOL_TAG: u64 = IMM_TAG;

    pub const FALSE: u64 = 0x0 | IMM_TAG;
    pub const TRUE: u64 = 0x1 | IMM_TAG;
    pub const NULL: u64 = 0x2 | IMM_TAG;
    pub const UNDEFINED: u64 = 0x3 | IMM_TAG;
    pub const THE_HOLE: u64 = 0x4 | IMM_TAG;

    pub const INLINE_STRING_TAG: u64 = 0x7FFC_0000_0000_0000;
    pub const INLINE_BIGINT_TAG: u64 = 0x7FFD_0000_0000_0000;

    pub const OBJECT_TAG: u64 = 0xFFF9_0000_0000_0000;
    pub const SYMBOL_TAG: u64 = 0xFFFA_0000_0000_0000;

    pub const HEAP_STRING_TAG: u64 = 0xFFFC_0000_0000_0000;
    pub const HEAP_BIGINT_TAG: u64 = 0xFFFD_0000_0000_0000;

    pub const MASK_NAN: u64 = 0x7FF8_0000_0000_0000;
    pub const MASK_KIND: u64 = MASK_NAN | 0xF_0000_0000_0000;
    pub const TAG_INF: u64 = 0x0_0000_0000_0000;
    pub const TAG_NAN: u64 = 0x8_0000_0000_0000;

    pub const VALUE_48BIT_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

    pub const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

    pub const IGNORE_INLINE_MASK: u64 = 0x7FFF_0000_0000_0000;

    const _INLINE_STRING_CHECK: () =
        assert!(INLINE_STRING_TAG & IGNORE_INLINE_MASK == HEAP_STRING_TAG & IGNORE_INLINE_MASK);
    const _INLINE_BIGINT_CHECK: () =
        assert!(INLINE_BIGINT_TAG & IGNORE_INLINE_MASK == HEAP_BIGINT_TAG & IGNORE_INLINE_MASK);


    pub const fn is_f64(val: u64) -> bool {
        (val & MASK_NAN) != MASK_NAN
            || (val & MASK_KIND) == TAG_INF
            || (val & MASK_KIND) == TAG_NAN
    }

    pub const fn is_int32(val: u64) -> bool {
        tag(val) == INT32_TAG
    }

    pub const fn is_imm(val: u64) -> bool {
        val == IMM_TAG
    }

    pub const fn is_bool(val: u64) -> bool {
        val == TRUE || val == FALSE
    }

    pub const fn is_null(val: u64) -> bool {
        val == NULL
    }

    pub const fn is_undefined(val: u64) -> bool {
        val == UNDEFINED
    }

    pub const fn is_the_hole(val: u64) -> bool {
        val == THE_HOLE
    }

    pub const fn is_inline_string(val: u64) -> bool {
        tag(val) == INLINE_STRING_TAG
    }

    pub const fn is_inline_big_int(val: u64) -> bool {
        tag(val) == INLINE_BIGINT_TAG
    }

    pub const fn is_object(val: u64) -> bool {
        tag(val) == OBJECT_TAG
    }

    pub const fn is_symbol(val: u64) -> bool {
        tag(val) == SYMBOL_TAG
    }

    pub const fn is_heap_string(val: u64) -> bool {
        tag(val) == HEAP_STRING_TAG
    }

    pub const fn is_heap_big_int(val: u64) -> bool {
        tag(val) == HEAP_BIGINT_TAG
    }

    pub const fn is_string(val: u64) -> bool {
        ignore_inline_tag(val) == HEAP_STRING_TAG
    }

    pub const fn is_big_int(val: u64) -> bool {
        ignore_inline_tag(val) == HEAP_BIGINT_TAG
    }

    pub const fn tag(val: u64) -> u64 {
        val & TAG_MASK
    }

    pub const fn ignore_inline_tag(val: u64) -> u64 {
        val & IGNORE_INLINE_MASK
    }

    pub const fn val(val: u64) -> u64 {
        val & VALUE_48BIT_MASK
    }

    pub const fn box_f64(val: f64) -> u64 {
        if val.is_nan() {
            f64::NAN.to_bits()
        } else {
            val.to_bits()
        }
    }

    pub fn box_ptr<const TAG: u64>(ptr: NonNull<()>) -> u64 {
        ptr.expose_provenance().get() as u64 | TAG
    }


    pub const fn box_int32(val: i32) -> u64 {
        (val as u64) | INT32_TAG
    }

    pub const fn box_bool(val: bool) -> u64 {
        (val as u64) | BOOL_TAG
    }

    pub const fn null() -> u64 {
        NULL
    }

    pub const fn undefined() -> u64 {
        UNDEFINED
    }

    pub const fn the_hole() -> u64 {
        THE_HOLE
    }

    pub fn box_inline_string(val: [u8; 6]) -> u64 {
        let mut expanded = [0u8; 8];
        expanded[0..6].copy_from_slice(&val);
        u64::from_le_bytes(expanded) | INLINE_STRING_TAG
    }

    pub const fn box_inline_big_int(val: i64) -> u64 {
        (val as u64 & VALUE_48BIT_MASK) | INLINE_BIGINT_TAG
    }

    pub fn box_object(ptr: NonNull<()>) -> u64 {
        box_ptr::<OBJECT_TAG>(ptr)
    }

    pub fn box_symbol(ptr: NonNull<()>) -> u64 {
        box_ptr::<SYMBOL_TAG>(ptr)
    }

    pub fn box_heap_string(ptr: NonNull<()>) -> u64 {
        box_ptr::<HEAP_STRING_TAG>(ptr)
    }

    pub fn box_heap_big_int(ptr: NonNull<()>) -> u64 {
        box_ptr::<HEAP_BIGINT_TAG>(ptr)
    }


    pub const unsafe fn unbox_f64(val: u64) -> f64 {
        debug_assert!(is_f64(val));
        f64::from_bits(val)
    }

    pub const unsafe fn unbox_int32(val: u64) -> i32 {
        debug_assert!(is_int32(val));
        val as i32
    }

    pub const unsafe fn unbox_bool(val: u64) -> bool {
        debug_assert!(is_bool(val));

        val == TRUE
    }

    pub unsafe fn unbox_inline_string(val: u64) -> [u8; 6] {
        debug_assert!(is_inline_string(val));

        let bytes = (val & VALUE_48BIT_MASK).to_le_bytes();
        let mut result = [0u8; 6];
        result.copy_from_slice(&bytes[0..6]);
        result
    }

    pub unsafe fn unbox_inline_big_int(val: u64) -> i64 {
        debug_assert!(is_inline_big_int(val));

        (((val & VALUE_48BIT_MASK) << 16) as i64) >> 16
    }

    pub unsafe fn unbox_ptr(val: u64) -> NonNull<()> {
        unsafe {
            let ptr_val = NonZeroUsize::new_unchecked((val & VALUE_48BIT_MASK) as usize);
            NonNull::with_exposed_provenance(ptr_val)
        }
    }

    pub unsafe fn unbox_object(val: u64) -> NonNull<()> {
        debug_assert!(is_object(val));

        unsafe { unbox_ptr(val) }
    }

    pub unsafe fn unbox_symbol(val: u64) -> NonNull<()> {
        debug_assert!(is_symbol(val));

        unsafe { unbox_ptr(val) }
    }

    pub unsafe fn unbox_heap_string(val: u64) -> NonNull<()> {
        debug_assert!(is_heap_string(val));

        unsafe { unbox_ptr(val) }
    }

    pub unsafe fn unbox_heap_big_int(val: u64) -> NonNull<()> {
        debug_assert!(is_heap_big_int(val));

        unsafe { unbox_ptr(val) }
    }
}

impl ValueInner {
    pub const unsafe fn from_bits(bits: u64) -> Self {
        Self { val: bits }
    }

    pub const fn from_f64(val: f64) -> Self {
        Self {
            val: bits::box_f64(val)
        }
    }

    pub const fn from_int32(val: i32) -> Self {
        Self {
            val: bits::box_int32(val),
        }
    }

    pub const fn from_bool(val: bool) -> Self {
        Self {
            val: bits::box_bool(val),
        }
    }

    pub const fn null() -> Self {
        Self { val: bits::null() }
    }

    pub const fn undefined() -> Self {
        Self {
            val: bits::undefined(),
        }
    }

    pub const unsafe fn the_hole() -> Self {
        Self {
            val: bits::the_hole(),
        }
    }

    pub fn from_inline_string(val: [u8; 6]) -> Self {
        Self {
            val: bits::box_inline_string(val),
        }
    }

    pub const fn from_inline_big_int(val: i64) -> Self {
        Self {
            val: bits::box_inline_big_int(val),
        }
    }

    pub fn from_object(ptr: NonNull<()>) -> Self {
        Self {
            val: bits::box_object(ptr),
        }
    }

    pub fn from_symbol(ptr: NonNull<()>) -> Self {
        Self {
            val: bits::box_symbol(ptr),
        }
    }

    pub fn from_heap_string(ptr: NonNull<()>) -> Self {
        Self {
            val: bits::box_heap_string(ptr),
        }
    }

    pub fn from_heap_big_int(ptr: NonNull<()>) -> Self {
        Self {
            val: bits::box_heap_big_int(ptr),
        }
    }

    pub fn from_string(val: JSString) -> Self {
        match val {
            JSString::Inline(bytes) => Self::from_inline_string(bytes),
            JSString::Heap(ptr) => Self::from_heap_string(ptr),
        }
    }

    pub fn from_big_int(val: JSBigInt) -> Self {
        match val {
            JSBigInt::Inline(int) => Self::from_inline_big_int(int),
            JSBigInt::Heap(ptr) => Self::from_heap_big_int(ptr),
        }
    }

    pub const fn to_bits(self) -> u64 {
        self.val
    }

    pub const fn is_f64(self) -> bool {
        bits::is_f64(self.val)
    }

    pub const fn is_int32(self) -> bool {
        bits::is_int32(self.val)
    }

    pub const fn is_imm(self) -> bool {
        bits::is_imm(self.val)
    }

    pub const fn is_bool(self) -> bool {
        bits::is_bool(self.val)
    }

    pub const fn is_null(self) -> bool {
        bits::is_null(self.val)
    }

    pub const fn is_undefined(self) -> bool {
        bits::is_undefined(self.val)
    }

    pub const fn is_the_hole(self) -> bool {
        bits::is_the_hole(self.val)
    }

    pub const fn is_inline_string(self) -> bool {
        bits::is_inline_string(self.val)
    }

    pub const fn is_inline_big_int(self) -> bool {
        bits::is_inline_big_int(self.val)
    }

    pub const fn is_object(self) -> bool {
        bits::is_object(self.val)
    }

    pub const fn is_symbol(self) -> bool {
        bits::is_symbol(self.val)
    }

    pub const fn is_heap_string(self) -> bool {
        bits::is_heap_string(self.val)
    }

    pub const fn is_heap_big_int(self) -> bool {
        bits::is_heap_big_int(self.val)
    }

    pub const fn is_string(self) -> bool {
        bits::is_string(self.val)
    }

    pub const fn is_big_int(self) -> bool {
        bits::is_big_int(self.val)
    }

    pub const fn as_f64(self) -> Option<f64> {
        if self.is_f64() {
            unsafe { Some(bits::unbox_f64(self.val)) }
        } else {
            None
        }
    }

    pub const fn as_int32(self) -> Option<i32> {
        if self.is_int32() {
            unsafe { Some(bits::unbox_int32(self.val)) }
        } else {
            None
        }
    }

    pub const fn as_bool(self) -> Option<bool> {
        if self.is_bool() {
            unsafe { Some(bits::unbox_bool(self.val)) }
        } else {
            None
        }
    }

    pub fn as_inline_string(self) -> Option<[u8; 6]> {
        if self.is_inline_string() {
            unsafe { Some(bits::unbox_inline_string(self.val)) }
        } else {
            None
        }
    }

    pub fn as_inline_big_int(self) -> Option<i64> {
        if self.is_inline_big_int() {
            unsafe { Some(bits::unbox_inline_big_int(self.val)) }
        } else {
            None
        }
    }

    pub fn as_object(self) -> Option<NonNull<()>> {
        if self.is_object() {
            unsafe { Some(bits::unbox_object(self.val)) }
        } else {
            None
        }
    }

    pub fn as_symbol(self) -> Option<NonNull<()>> {
        if self.is_symbol() {
            unsafe { Some(bits::unbox_symbol(self.val)) }
        } else {
            None
        }
    }

    pub fn as_heap_string(self) -> Option<NonNull<()>> {
        if self.is_heap_string() {
            unsafe { Some(bits::unbox_heap_string(self.val)) }
        } else {
            None
        }
    }

    pub fn as_heap_big_int(self) -> Option<NonNull<()>> {
        if self.is_heap_big_int() {
            unsafe { Some(bits::unbox_heap_big_int(self.val)) }
        } else {
            None
        }
    }

    pub fn as_string(self) -> Option<JSString> {
        if self.is_heap_string() {
            unsafe {
                let ptr = bits::unbox_heap_string(self.val);
                Some(JSString::Heap(ptr))
            }
        } else if self.is_inline_string() {
            unsafe {
                let bytes = bits::unbox_inline_string(self.val);
                Some(JSString::Inline(bytes))
            }
        } else {
            None
        }
    }

    pub fn as_big_int(self) -> Option<JSBigInt> {
        if self.is_heap_big_int() {
            unsafe {
                let ptr = bits::unbox_heap_big_int(self.val);
                Some(JSBigInt::Heap(ptr))
            }
        } else if self.is_inline_big_int() {
            unsafe {
                let val = bits::unbox_inline_big_int(self.val);
                Some(JSBigInt::Inline(val))
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JSString {
    Inline([u8; 6]),
    Heap(NonNull<()>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JSBigInt {
    Inline(i64),
    Heap(NonNull<()>),
}



#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_f64_val() {
        let value = ValueInner::from_f64(42.42);

        assert_eq!(value.to_bits(), 42.42f64.to_bits());
        assert_eq!(value.as_f64(), Some(42.42));
    }


    #[test]
    fn test_f64_inf() {
        let inf = ValueInner::from_f64(f64::INFINITY);
        let neg_inf = ValueInner::from_f64(f64::NEG_INFINITY);

        assert_eq!(inf.to_bits(), f64::INFINITY.to_bits());
        assert_eq!(inf.as_f64(), Some(f64::INFINITY));

        assert_eq!(neg_inf.to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(neg_inf.as_f64(), Some(f64::NEG_INFINITY));
    }

    #[test]
    fn test_f64_nan() {
        let nan = ValueInner::from_f64(f64::NAN);

        println!("{:#b}", nan.to_bits());
        println!("{:#x}", nan.to_bits());

        assert_eq!(nan.to_bits(), f64::NAN.to_bits());
        assert!(f64::from_bits(nan.to_bits()).is_nan());
        assert_eq!(nan.as_f64(), Some(f64::NAN));
        assert!(nan.as_f64().unwrap().is_nan());
    }

    #[test]
    fn test_int32() {
        let value = ValueInner::from_int32(42);
        assert!(value.is_int32());
        assert_eq!(value.as_int32(), Some(42));
    }

    #[test]
    fn test_bool() {
        let true_value = ValueInner::from_bool(true);
        let false_value = ValueInner::from_bool(false);

        assert!(true_value.is_bool());
        assert!(false_value.is_bool());

        assert_eq!(true_value.as_bool(), Some(true));
        assert_eq!(false_value.as_bool(), Some(false));
    }

    #[test]
    fn test_null_undefined() {
        let null_value = ValueInner::null();
        let undefined_value = ValueInner::undefined();

        assert!(null_value.is_null());
        assert!(undefined_value.is_undefined());
    }

    #[test]
    fn test_inline_string() {
        let string_value = ValueInner::from_inline_string(*b"luna!?");
        assert!(string_value.is_inline_string());
        assert_eq!(string_value.as_inline_string(), Some(*b"luna!?"));
    }

    #[test]
    fn test_heap_string() {
        let heap_string_ptr = NonNull::new(0x1234 as *mut ()).unwrap();
        let string_value = ValueInner::from_heap_string(heap_string_ptr);

        assert!(string_value.is_heap_string());
        assert_eq!(string_value.as_heap_string(), Some(heap_string_ptr));
    }

    #[test]
    fn test_object() {
        let object_ptr = NonNull::new(0x4242 as *mut ()).unwrap();
        let object_value = ValueInner::from_object(object_ptr);

        assert!(object_value.is_object());
        assert_eq!(object_value.as_object(), Some(object_ptr));
    }

    #[test]
    fn test_symbol() {
        let symbol_ptr = NonNull::new(0x1337 as *mut ()).unwrap();
        let symbol_value = ValueInner::from_symbol(symbol_ptr);

        assert!(symbol_value.is_symbol());
        assert_eq!(symbol_value.as_symbol(), Some(symbol_ptr));
    }

    #[test]
    fn test_inline_big_int() {
        let big_int_value = ValueInner::from_inline_big_int(42_1337_6967);
        assert!(big_int_value.is_inline_big_int());
        assert_eq!(big_int_value.as_inline_big_int(), Some(42_1337_6967));
    }

    #[test]
    fn test_inline_neg_big_int() {
        let big_int_value = ValueInner::from_inline_big_int(-42_1337_6967);
        assert!(big_int_value.is_inline_big_int());
        assert_eq!(big_int_value.as_inline_big_int(), Some(-42_1337_6967));
    }

    #[test]
    fn test_heap_big_int() {
        let heap_big_int_ptr = NonNull::new(0x6969 as *mut ()).unwrap();
        let big_int_value = ValueInner::from_heap_big_int(heap_big_int_ptr);

        assert!(big_int_value.is_heap_big_int());
        assert_eq!(big_int_value.as_heap_big_int(), Some(heap_big_int_ptr));
    }

    #[test]
    fn test_string_enum() {
        let inline_string = JSString::Inline(*b"luna!?");
        let heap_string_ptr = NonNull::new(0x6767 as *mut ()).unwrap();
        let heap_string = JSString::Heap(heap_string_ptr);

        let inline_value = ValueInner::from_string(inline_string);
        let heap_value = ValueInner::from_string(heap_string);

        assert!(inline_value.is_inline_string());
        assert!(heap_value.is_heap_string());

        assert_eq!(inline_value.as_string(), Some(inline_string));
        assert_eq!(heap_value.as_string(), Some(heap_string));
    }

    #[test]
    fn test_big_int_enum() {
        let inline_big_int = JSBigInt::Inline(42_1337_6967);
        let inline_neg_big_int = JSBigInt::Inline(-42_1337_6967);
        let heap_big_int_ptr = NonNull::new(0x4200 as *mut ()).unwrap();
        let heap_big_int = JSBigInt::Heap(heap_big_int_ptr);

        let inline_value = ValueInner::from_big_int(inline_big_int);
        let inline_neg_value = ValueInner::from_big_int(inline_neg_big_int);
        let heap_value = ValueInner::from_big_int(heap_big_int);

        assert!(inline_value.is_inline_big_int());
        assert!(inline_neg_value.is_inline_big_int());
        assert!(heap_value.is_heap_big_int());

        assert_eq!(inline_value.as_big_int(), Some(inline_big_int));
        assert_eq!(inline_neg_value.as_big_int(), Some(inline_neg_big_int));
        assert_eq!(heap_value.as_big_int(), Some(heap_big_int));
    }
}
