use crate::uz::{DoubleU4, UsizeSmall, UZ_BYTES};

pub struct SmallVecLenCap {
    len: UsizeSmall,
    shared: DoubleU4,
    cap: UsizeSmall,
}

impl SmallVecLenCap {
    pub fn new(len: usize, cap: usize) -> Option<Self> {
        if len > 0x7F_FF_FF_FF || cap > 0x7F_FF_FF_FF {
            return None;
        }

        let (len, len_shared) = uz_to_bytes(len);
        let (cap, cap_shared) = uz_to_bytes(cap);

        let shared = DoubleU4::new(len_shared, cap_shared)?;

        Some(Self {
            len: UsizeSmall::from_le_bytes(len),
            shared,
            cap: UsizeSmall::from_le_bytes(cap),
        })
    }

    pub fn len(&self) -> usize {
        let without_shared = self.len.to_usize();

        let shared = self.shared.first();

        without_shared | (shared as usize) << (8 * UZ_BYTES)
    }

    pub fn cap(&self) -> usize {
        let without_shared = self.cap.to_usize();

        let shared = self.shared.second();

        without_shared | (shared as usize) << (8 * UZ_BYTES)
    }
}

pub fn uz_to_bytes(uz: usize) -> ([u8; UZ_BYTES], u8) {
    let bytes_full = uz.to_le_bytes();

    let most_worth = bytes_full[UZ_BYTES];

    let mut res = [0; UZ_BYTES];

    for i in 0..UZ_BYTES {
        res[i] = bytes_full[UZ_BYTES - 1 - i];
    }

    (res, most_worth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_vec_len_cap() {
        let small_vec_len_cap = SmallVecLenCap::new(0, 0).unwrap();

        assert_eq!(small_vec_len_cap.len(), 0);
        assert_eq!(small_vec_len_cap.cap(), 0);

        let small_vec_len_cap = SmallVecLenCap::new(1, 1).unwrap();

        assert_eq!(small_vec_len_cap.len(), 1);
        assert_eq!(small_vec_len_cap.cap(), 1);

        let small_vec_len_cap = SmallVecLenCap::new(0x7F_FF_FF_FF, 0x7F_FF_FF_FF).unwrap();

        assert_eq!(small_vec_len_cap.len(), 0x7F_FF_FF_FF);
        assert_eq!(small_vec_len_cap.cap(), 0x7F_FF_FF_FF);
    }
}
