use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::{Mutex as StdMutex, RwLock as StdRwLock};

use parking_lot::{Mutex, RwLock};

use super::{Collectable, Gc, GcBox};

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



pub struct GcRefCellGuard<'a, T: CellCollectable<RefCell<T>>> {
    value: Ref<'a, T>,
    gc: NonNull<GcBox<RefCell<T>>>,
}

impl<T: CellCollectable<RefCell<T>>> Drop for GcRefCellGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            GcBox::update_refs(self.gc);
        }
    }
}

impl<'a, T: CellCollectable<RefCell<T>>> Deref for GcRefCellGuard<'a, T> {
    type Target = Ref<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}


pub struct GcMutRefCellGuard<'a, T: CellCollectable<RefCell<T>>> {
    value: RefMut<'a, T>,
    gc: NonNull<GcBox<RefCell<T>>>,
}

impl<T: CellCollectable<RefCell<T>>> Drop for GcMutRefCellGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            GcBox::update_refs(self.gc);
        }
    }
}


impl<'a, T: CellCollectable<RefCell<T>>> Deref for GcMutRefCellGuard<'a, T> {
    type Target = RefMut<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T: CellCollectable<RefCell<T>>> DerefMut for GcMutRefCellGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}


impl<T: CellCollectable<RefCell<T>>> Gc<RefCell<T>> {
    pub fn borrow(&self) -> Result<GcRefCellGuard<T>, BorrowError> {
        unsafe {
            let value = (*(*self.inner.as_ptr()).value.as_ptr()).try_borrow()?;

            Ok(GcRefCellGuard {
                value,
                gc: self.inner,
            })
        }
    }
    
    pub fn borrow_mut(&self) -> Result<GcMutRefCellGuard<T>, BorrowMutError> {
        unsafe {
            let value = (*(*self.inner.as_ptr()).value.as_ptr()).try_borrow_mut()?;

            Ok(GcMutRefCellGuard {
                value,
                gc: self.inner,
            })
        }
    }
}

