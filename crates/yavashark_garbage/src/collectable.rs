use super::{Collectable, Gc};


macro_rules! collect {
    ($ty:ty) => {
        unsafe impl Collectable for $ty {
            fn get_refs(&self) -> Vec<Gc<Self>> {
                Vec::new()
            }

            fn get_refs_diff(&self, old: &[Gc<Self>]) -> (Vec<Gc<Self>>, Vec<Gc<Self>>) {
                (old.to_vec(), Vec::new())
            }
        }
    };
}


collect!(());
collect!(bool);
collect!(char);
collect!(f32);
collect!(f64);
collect!(i8);
collect!(i16);
collect!(i32);
collect!(i64);
collect!(i128);
collect!(isize);
collect!(u8);
collect!(u16);
collect!(u32);
collect!(u64);
collect!(u128);
collect!(usize);

collect!(String);
