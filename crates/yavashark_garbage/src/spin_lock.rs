use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) trait SpinLock<'a, T> {
    type WriteTarget<W: 'a> where Self: 'a;
    type ReadTarget<R: 'a> where Self: 'a;

    fn spin_write(&'a self) -> Option<Self::WriteTarget<T>>;
    fn spin_read(&'a self) -> Option<Self::ReadTarget<T>>;
}

impl<'a, T> SpinLock<'a, T> for RwLock<T> {
    type WriteTarget<W: 'a> = RwLockWriteGuard<'a, W> where Self: 'a;
    type ReadTarget<R: 'a> = RwLockReadGuard<'a, R> where Self: 'a;

    fn spin_write(&'a self) -> Option<Self::WriteTarget<T>> {
        let mut retries = 500;

        loop {
            let Ok(write) = self.try_write() else {
                // Theoretically we should never get here...  and have the loop looping - TODO: use a cfg attr to remove this code on size opt levels
                std::hint::spin_loop();
                retries -= 1;

                if retries == 0 {
                    //TODO: warn that we have leaked a references
                    break None;
                }

                continue;
            };

            return Some(write);
        }
    }

    fn spin_read(&'a self) -> Option<Self::ReadTarget<T>> {
        let mut retries = 500;

        loop {
            let Ok(read) = self.try_read() else {
                // Theoretically we should never get here...  and have the loop looping - TODO: use a cfg attr to remove this code on size opt levels
                std::hint::spin_loop();
                retries -= 1;

                if retries == 0 {
                    //TODO: warn that we have leaked a references
                    break None;
                }

                continue;
            };

            return Some(read);
        }
    }
}
