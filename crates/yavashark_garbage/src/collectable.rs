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

pub struct GcRefCellGuard<'a, T: CellCollectable<RefCell<T>>, V = T> {
    // this is always Some, except when the destructor runs
    value: Option<Ref<'a, V>>,
    gc: NonNull<GcBox<RefCell<T>>>,
}

impl<T: CellCollectable<RefCell<T>>, V> Drop for GcRefCellGuard<'_, T, V> {
    fn drop(&mut self) {
        unsafe {
            if self.value.is_some() {
                GcBox::update_refs(self.gc);
            }
        }
    }
}

impl<'a, T: CellCollectable<RefCell<T>>> Deref for GcRefCellGuard<'a, T> {
    type Target = Ref<'a, T>;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::expect_used)]
        self.value.as_ref().expect("unreachable")
    }
}

impl<'a, T: CellCollectable<RefCell<T>>, V> GcRefCellGuard<'a, T, V> {
    #[allow(clippy::missing_panics_doc)]
    pub fn map<R, F: FnOnce(&V) -> &R>(mut self, f: F) -> GcRefCellGuard<'a, T, R> {
        #[allow(clippy::expect_used)]
        let value = Ref::map(self.value.take().expect("unreachable"), f);

        GcRefCellGuard {
            value: Some(value),
            gc: self.gc,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn maybe_map<R, F: FnOnce(&V) -> Option<&R>>(
        mut self,
        f: F,
    ) -> Result<GcRefCellGuard<'a, T, R>, Self> {
        #[allow(clippy::expect_used)]
        let value = self.value.take().expect("unreachable");

        let value = match Ref::filter_map(value, f) {
            Ok(v) => v,
            Err(v) => {
                self.value = Some(v);

                return Err(self);
            }
        };

        Ok(GcRefCellGuard {
            value: Some(value),
            gc: self.gc,
        })
    }
}

// I hope this doesn't bite me in the ass if so, the person finding it can say "well..."
pub struct OwningGcRefCellGuard<'a, T: CellCollectable<RefCell<T>>, V = T> {
    // this is always Some, except when the destructor runs
    value: Option<Ref<'a, V>>,
    gc: Gc<RefCell<T>>, // this is used to keep the Gc alive
}

impl<T: CellCollectable<RefCell<T>>, V> Drop for OwningGcRefCellGuard<'_, T, V> {
    fn drop(&mut self) {
        drop(self.value.take());
    }
}

impl<T: CellCollectable<RefCell<T>>, V> Deref for OwningGcRefCellGuard<'_, T, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::expect_used)]
        self.value.as_ref().expect("unreachable")
    }
}

impl<'a, T: CellCollectable<RefCell<T>>, V> OwningGcRefCellGuard<'a, T, V> {
    #[allow(clippy::missing_panics_doc)]
    pub fn map<R, F: FnOnce(&V) -> &R>(mut self, f: F) -> OwningGcRefCellGuard<'a, T, R> {
        #[allow(clippy::expect_used)]
        let value = Ref::map(self.value.take().expect("unreachable"), f);

        OwningGcRefCellGuard {
            value: Some(value),
            gc: self.gc.clone(),
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn maybe_map<R, F: FnOnce(&V) -> Option<&R>>(
        mut self,
        f: F,
    ) -> Result<OwningGcRefCellGuard<'a, T, R>, Self> {
        #[allow(clippy::expect_used)]
        let value = self.value.take().expect("unreachable");

        let value = match Ref::filter_map(value, f) {
            Ok(v) => v,
            Err(v) => {
                self.value = Some(v);

                return Err(self);
            }
        };

        Ok(OwningGcRefCellGuard {
            value: Some(value),
            gc: self.gc.clone(),
        })
    }
}

// I hope this doesn't bite me in the ass if so, the person finding it can say "well..."
pub struct GcMutRefCellGuard<'a, T: CellCollectable<RefCell<T>>, V = T> {
    /// # Safety
    /// This value should only be set None when the guard is dropped
    value: Option<RefMut<'a, V>>,
    gc: NonNull<GcBox<RefCell<T>>>,
}

impl<T: CellCollectable<RefCell<T>>, V> Drop for GcMutRefCellGuard<'_, T, V> {
    fn drop(&mut self) {
        unsafe {
            if self.value.is_some() {
                drop(self.value.take());
                GcBox::update_refs(self.gc);
            }
        }
    }
}

impl<'a, T: CellCollectable<RefCell<T>>, V> Deref for GcMutRefCellGuard<'a, T, V> {
    type Target = RefMut<'a, V>;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::expect_used)]
        self.value.as_ref().expect("unreachable") // this can only be None if the guard is dropped
    }
}

impl<T: CellCollectable<RefCell<T>>, V> DerefMut for GcMutRefCellGuard<'_, T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(clippy::expect_used)]
        self.value.as_mut().expect("unreachable") // this can only be None if the guard is dropped
    }
}

pub struct OwningGcMutRefCellGuard<'a, T: CellCollectable<RefCell<T>>, V = T> {
    /// # Safety
    /// This value should only be set None when the guard is dropped
    value: Option<RefMut<'a, V>>,
    gc: Gc<RefCell<T>>,
}

impl<T: CellCollectable<RefCell<T>>, V> Drop for OwningGcMutRefCellGuard<'_, T, V> {
    fn drop(&mut self) {
        drop(self.value.take());
    }
}

impl<T: CellCollectable<RefCell<T>>, V> Deref for OwningGcMutRefCellGuard<'_, T, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::expect_used)]
        self.value.as_ref().expect("unreachable") // this can only be None if the guard is dropped
    }
}

impl<T: CellCollectable<RefCell<T>>, V> DerefMut for OwningGcMutRefCellGuard<'_, T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(clippy::expect_used)]
        self.value.as_mut().expect("unreachable") // this can only be None if the guard is dropped
    }
}

impl<'a, T: CellCollectable<RefCell<T>>, V> OwningGcMutRefCellGuard<'a, T, V> {
    #[allow(clippy::missing_panics_doc)]
    pub fn map<R, F: FnOnce(&mut V) -> &mut R>(
        mut self,
        f: F,
    ) -> OwningGcMutRefCellGuard<'a, T, R> {
        #[allow(clippy::expect_used)]
        let value = RefMut::map(self.value.take().expect("unreachable"), f);

        OwningGcMutRefCellGuard {
            value: Some(value),
            gc: self.gc.clone(),
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn maybe_map<R, F: FnOnce(&mut V) -> Option<&mut R>>(
        mut self,
        f: F,
    ) -> Result<OwningGcMutRefCellGuard<'a, T, R>, Self> {
        #[allow(clippy::expect_used)]
        let value = self.value.take().expect("unreachable");

        let value = match RefMut::filter_map(value, f) {
            Ok(v) => v,
            Err(v) => {
                self.value = Some(v);

                return Err(self);
            }
        };

        Ok(OwningGcMutRefCellGuard {
            value: Some(value),
            gc: self.gc.clone(),
        })
    }
}

impl<T: CellCollectable<RefCell<T>>> Gc<RefCell<T>> {
    pub fn borrow(&self) -> Result<GcRefCellGuard<T>, BorrowError> {
        unsafe {
            let value = (*(*self.inner.as_ptr()).value.as_ptr()).try_borrow()?;

            Ok(GcRefCellGuard {
                value: Some(value),
                gc: self.inner,
            })
        }
    }

    pub fn own<'b>(&self) -> Result<OwningGcRefCellGuard<'b, T>, BorrowError> {
        unsafe {
            let value = (*(*self.inner.as_ptr()).value.as_ptr()).try_borrow()?;

            Ok(OwningGcRefCellGuard {
                value: Some(value),
                gc: self.clone(),
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

    pub fn own_mut<'b>(&self) -> Result<OwningGcMutRefCellGuard<'b, T>, BorrowMutError> {
        unsafe {
            let value = Some((*(*self.inner.as_ptr()).value.as_ptr()).try_borrow_mut()?);

            Ok(OwningGcMutRefCellGuard {
                value,
                gc: self.clone(),
            })
        }
    }
}

impl<'a, T: CellCollectable<RefCell<T>>, V> GcMutRefCellGuard<'a, T, V> {
    #[allow(clippy::missing_panics_doc)]
    pub fn map<R, F: FnOnce(&mut V) -> &mut R>(mut self, f: F) -> GcMutRefCellGuard<'a, T, R> {
        #[allow(clippy::expect_used)]
        let value = self.value.take().expect("unreachable");

        let value = RefMut::map(value, f);

        GcMutRefCellGuard {
            value: Some(value),
            gc: self.gc,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn maybe_map<R, F: FnOnce(&mut V) -> Option<&mut R>>(
        mut self,
        f: F,
    ) -> Result<GcMutRefCellGuard<'a, T, R>, Self> {
        #[allow(clippy::expect_used)]
        let value = self.value.take().expect("unreachable");

        let value = match RefMut::filter_map(value, f) {
            Ok(v) => v,
            Err(v) => {
                self.value = Some(v);

                return Err(self);
            }
        };

        Ok(GcMutRefCellGuard {
            value: Some(value),
            gc: self.gc,
        })
    }
}
