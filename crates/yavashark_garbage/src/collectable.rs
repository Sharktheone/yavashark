use std::cell::RefCell;
use super::{Collectable, Gc};
use std::sync::{RwLock as StdRwLock, Mutex as StdMutex};
use parking_lot::{Mutex, RwLock};

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



/// # Safety
/// The implementer must guarantee that all references are valid and all references are returned by `get_refs`
pub unsafe trait CellCollectable<T: Collectable> {
    fn get_refs(&self) -> Vec<Gc<T>>;


    /// (removed, added)
    fn get_refs_diff(&self, old: &[Gc<T>]) -> (Vec<Gc<T>>, Vec<Gc<T>>);
}



macro_rules! cell {
    ($ty:ident,$lock:ident) => {
        unsafe impl<T: CellCollectable<Self>> Collectable for $ty<T> {
            fn get_refs(&self) -> Vec<Gc<Self>> {
                self.$lock().map(|x| x.get_refs()).unwrap_or_default()
            }

            fn get_refs_diff(&self, old: &[Gc<Self>]) -> (Vec<Gc<Self>>, Vec<Gc<Self>>) {
                self.$lock().map(|x| x.get_refs_diff(old)).unwrap_or_default()
            }
        }
    };
}


cell!(RefCell, try_borrow);
cell!(StdRwLock, read);
cell!(RwLock, try_read);
cell!(StdMutex, lock);
cell!(Mutex, try_lock);