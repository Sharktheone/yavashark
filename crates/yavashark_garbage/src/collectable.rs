use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::{Mutex as StdMutex, RwLock as StdRwLock};

use parking_lot::{Mutex, RwLock};

use super::{Collectable, Gc, GcBox, GcRef};

macro_rules! collect {
    ($ty:ty) => {
        unsafe impl Collectable for $ty {
            fn get_refs(&self) -> Vec<GcRef<Self>> {
                Vec::new()
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

///This trait optimizes the usage of Cells like `RefCell`s and others in the Gc
///they do not need to use a typeless gc, but instead can just return also RefCell-References
/// # Safety
/// The implementer must guarantee that all references are valid and all references are returned by `get_refs`
pub unsafe trait CellCollectable<T: Collectable> {
    fn get_refs(&self) -> Vec<GcRef<T>>;

    #[cfg(feature = "easy_debug")]
    #[must_use]
    fn trace_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

macro_rules! cell {
    ($ty:ident,$lock:ident) => {
        unsafe impl<T: CellCollectable<Self>> Collectable for $ty<T> {
            fn get_refs(&self) -> Vec<GcRef<Self>> {
                self.$lock().map(|x| x.get_refs()).unwrap_or_default()
            }

            #[cfg(feature = "easy_debug")]
            fn trace_name(&self) -> &'static str {
                self.$lock().map(|x| x.trace_name()).unwrap_or("<unknown>")
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
    /// # Safety
    /// This value should only be set None when the guard is dropped
    value: Option<RefMut<'a, T>>,
    gc: NonNull<GcBox<RefCell<T>>>,
}

impl<T: CellCollectable<RefCell<T>>> Drop for GcMutRefCellGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            drop(self.value.take());
            GcBox::update_refs(self.gc);
        }
    }
}

impl<'a, T: CellCollectable<RefCell<T>>> Deref for GcMutRefCellGuard<'a, T> {
    type Target = RefMut<'a, T>;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::unwrap_used)]
        self.value.as_ref().unwrap() // this can only be None if the guard is dropped
    }
}

impl<'a, T: CellCollectable<RefCell<T>>> DerefMut for GcMutRefCellGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(clippy::unwrap_used)]
        self.value.as_mut().unwrap() // this can only be None if the guard is dropped
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
            let value = Some((*(*self.inner.as_ptr()).value.as_ptr()).try_borrow_mut()?);

            Ok(GcMutRefCellGuard {
                value,
                gc: self.inner,
            })
        }
    }
}
