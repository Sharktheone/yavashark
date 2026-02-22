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
    const MASK_NAN: u64 = 0x7FF8000000000000;

    const MASK_KIND: u64 = MASK_NAN | 0xF_0000_0000_0000;
    const MASK_KIND_OTHER: u64 = MASK_KIND | 0xF_C000_0000_0000;
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


}


fn a() {
    let a = Some(0);

    if let Some(a) = a {
        println!("a is {}", a);
    } else {
        println!("a is None");
    }
}