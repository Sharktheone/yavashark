pub const UZ_BYTES: usize = size_of::<usize>() - 1;

#[repr(Rust, packed)]
#[derive(Copy, Clone)]
pub struct UsizeSmall {
    bytes: [u8; UZ_BYTES],
}

impl UsizeSmall {
    pub fn new(val: usize) -> Option<Self> {
        let bytes = val.to_le_bytes();
        let last = bytes[UZ_BYTES];
        if last != 0 {
            return None;
        }

        let mut res = [0; UZ_BYTES];

        res.copy_from_slice(&bytes[..UZ_BYTES]);

        Some(Self { bytes: res })
    }

    pub fn from_le_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != UZ_BYTES {
            return None;
        }

        let mut res = [0; UZ_BYTES];

        res.copy_from_slice(slice);

        Some(Self { bytes: res })
    }

    pub const fn from_le_bytes(bytes: [u8; UZ_BYTES]) -> Self {
        Self { bytes }
    }

    pub fn to_usize(self) -> usize {
        let mut buf = [0; size_of::<usize>()];
        buf.copy_from_slice(&self.bytes);

        usize::from_le_bytes(buf)
    }

    pub const fn to_le_bytes_small(self) -> [u8; UZ_BYTES] {
        self.bytes
    }

    pub fn to_le_bytes(self) -> [u8; size_of::<usize>()] {
        let mut res = [0; size_of::<usize>()];

        res[..UZ_BYTES].copy_from_slice(&self.bytes[..UZ_BYTES]);

        res
    }
}

#[repr(Rust, packed)]
#[derive(Copy, Clone)]
pub struct DoubleU4(u8);

impl DoubleU4 {
    pub const fn new(first: u8, second: u8) -> Option<Self> {
        if first > 0xF || second > 0xF {
            return None;
        }
        Some(Self((first << 4) | second))
    }

    pub const fn from_u8(val: u8) -> Self {
        Self(val)
    }

    pub const fn to_u8(self) -> u8 {
        self.0
    }

    pub const fn to_u4(self) -> (u8, u8) {
        let val = self.0;
        (val >> 4, val & 0xF)
    }

    pub const fn first(self) -> u8 {
        self.0 >> 4
    }

    pub const fn second(self) -> u8 {
        self.0 & 0xF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usize_small() {
        let val = 0x1234_5678;
        let small = UsizeSmall::new(val).unwrap();
        assert_eq!(small.to_usize(), val);
        assert_eq!(small.to_le_bytes(), val.to_le_bytes());
    }

    #[test]
    fn test_usize_small_fail() {
        let val = 0x1234_5678_1234_5678;
        assert!(UsizeSmall::new(val).is_none());
    }

    #[test]
    fn test_double_u4_from_u8() {
        let val = 0x12;
        let double = DoubleU4::from_u8(val);
        assert_eq!(double.to_u8(), val);
        assert_eq!(double.to_u4(), (1, 2));
    }

    #[test]
    fn test_double_u4_new() {
        let first = 0x1;
        let second = 0x2;
        let double = DoubleU4::new(first, second).unwrap();
        assert_eq!(double.to_u8(), 0x12);
        assert_eq!(double.to_u4(), (first, second));

        let first = 0xF;
        let second = 0xF;
        let double = DoubleU4::new(first, second).unwrap();
        assert_eq!(double.to_u8(), 0xFF);
        assert_eq!(double.to_u4(), (first, second));
    }
}
