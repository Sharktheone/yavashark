#![allow(dead_code)]

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


    pub const VALUE_48BIT_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

    pub const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

    pub const IGNORE_INLINE_MASK: u64 = 0x7FFF_0000_0000_0000;

    const _INLINE_STRING_CHECK: () = assert!(INLINE_STRING_TAG & IGNORE_INLINE_MASK == HEAP_STRING_TAG & IGNORE_INLINE_MASK);
    const _INLINE_BIGINT_CHECK: () = assert!(INLINE_BIGINT_TAG & IGNORE_INLINE_MASK == HEAP_BIGINT_TAG & IGNORE_INLINE_MASK);



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
}
