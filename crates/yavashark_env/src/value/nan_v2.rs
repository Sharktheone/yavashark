
/// Int32      0111 1111 1111 1001 0000 0000 0000 0000 IIII .. IIII
///
/// Imm        0111 1111 1111 1010 0000 0000 0000 0000 iiii .. iiii
/// False      0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0000
/// True       0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0001
/// Null       0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0010
/// Undefined  0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0011
/// TheHole    0111 1111 1111 1010 0000 0000 0000 0000 0000 .. 0100
///
/// String     1111 1111 1111 1000 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// Object     1111 1111 1111 1001 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// Symbol     1111 1111 1111 1010 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// BigInt     1111 1111 1111 1011 PPPP PPPP PPPP PPPP PPPP .. PPPP
/// InlineStr  1111 1111 1111 1100 DDDD DDDD DDDD DDDD DDDD .. DDDD
/// BigInt48   1111 1111 1111 1101 BBBB BBBB BBBB BBBB BBBB .. BBBB
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ValueInner {
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
    half: u32,
    #[cfg(target_pointer_width = "16")]
    ptr_pad: u16,
    ptr: *const (),
}



