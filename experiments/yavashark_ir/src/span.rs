use std::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Span(pub u32);

impl Span {
    const fn new(span: u32) -> Self {
        Span(span)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(C, packed(4))]
pub struct Spanned<'a, T>(Span, &'a T);

const _: () = {
    assert!(size_of::<Spanned<'static, ()>>() == 12, "Spanned must be 12 bytes");
    assert!(size_of::<Spanned<'static, ()>>() == size_of::<Option<Spanned<'static, ()>>>());
};


impl<'a, T> Spanned<'a, T> {
    const fn new(span: Span, value: &'a T) -> Self {
        Spanned(span, value)
    }
}

impl<T> Deref for Spanned<'_, T> {
    type Target = Span;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Spanned<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}