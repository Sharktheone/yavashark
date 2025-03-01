pub trait FromBytes: Sized {
    const N_BYTES: usize = size_of::<Self>();
    type Bytes: for<'a> TryFrom<&'a [u8]> + AsRef<[u8]>;
    fn from_bytes(bytes: Self::Bytes, le: bool) -> Self;

    fn to_bytes(self, le: bool) -> Self::Bytes;
}

macro_rules! impl_from_bytes {
    ($($t:ty),*) => {
        $(
        impl FromBytes for $t {
            type Bytes = [u8; size_of::<Self>()];

            fn from_bytes(bytes: Self::Bytes, le: bool) -> Self {
                if le {
                    <$t>::from_le_bytes(bytes)
                } else {
                    <$t>::from_be_bytes(bytes)
                }
            }

            fn to_bytes(self, le: bool) -> Self::Bytes {
                if le {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                }
            }
        }
        )*
    };
}

impl_from_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
