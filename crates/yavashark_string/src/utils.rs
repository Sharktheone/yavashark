use std::mem;
use std::mem::MaybeUninit;
use std::rc::Rc;

pub fn units_to_ascii_rc(
    units: &[u16]
) -> Option<Rc<str>> {
    if !units.iter().all(|&u| u < 128) {
        return None
    }

    let mut ptr = Rc::into_raw(Rc::<[u8]>::new_uninit_slice(units.len()));

    let mut mut_slice = unsafe { std::slice::from_raw_parts_mut::<MaybeUninit<u8>>(ptr as *mut _, units.len()) };

    for (i, unit) in units.iter().enumerate() {
        mut_slice[i].write(*unit as u8);
    }

    unsafe {
        mut_slice.assume_init_mut();
    }


    Some(unsafe {
        Rc::from_raw(ptr as *const [u8] as *const str)
    })
}


pub fn units_iter_to_rc(iter: impl ExactSizeIterator<Item = u16>) -> Rc<[u16]> {
    let len = iter.len();

    let mut ptr = Rc::into_raw(Rc::<[u16]>::new_uninit_slice(len));

    let mut mut_slice = unsafe { &mut *ptr.cast_mut() };

    for (i, unit) in iter.enumerate() {
        mut_slice[i].write(unit);
    }


    unsafe {
        mut_slice.assume_init_mut();
    }


    unsafe {
        Rc::from_raw(ptr as *const [u16])
    }
}


pub enum TwoIter<T> {
    Two(T, T),
    One(T),
    None
}

impl<T: Default> Iterator for TwoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Two(a, b) => {
                let a = mem::take(a);
                *self = Self::One(mem::take(b));
                Some(a)
            }
            Self::One(a) => {
                let a = mem::take(a);
                *self = Self::None;
                Some(a)
            }
            Self::None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T: Default> ExactSizeIterator for TwoIter<T> {
    fn len(&self) -> usize {
        match self {
            Self::Two(_, _) => 2,
            Self::One(_) => 1,
            Self::None => 0,
        }
    }

}