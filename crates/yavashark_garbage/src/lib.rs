use std::ptr::NonNull;

pub struct Gc<T> {
    inner: NonNull<InnerGc<T>>,
}

struct InnerGc<T> {
    value: T,
    rc: usize,
    reference: NonNull<GcRef>,
}



struct GcRef {
    prev: Vec<MaybeNull<Self>>,
    next: Vec<MaybeNull<Self>>,
}



///Wrapper around a raw ptr that may be null (just to make it more explicit)
struct MaybeNull<T> (NonNull<T>);