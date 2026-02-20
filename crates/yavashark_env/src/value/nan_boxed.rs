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

pub struct ValueInner(u64);

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
