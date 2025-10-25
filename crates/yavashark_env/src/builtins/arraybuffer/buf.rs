use std::cell::RefCell;
use std::rc::Rc;

pub struct ResizableBuffer {
    pub buf: RefCell<Option<Vec<u8>>>,
    pub max_byte_length: Option<usize>,
}

pub struct DefaultBuffer {
    buf: RefCell<[u8]>,
}

pub struct ImmutableBuffer {
    buf: [u8],
}

pub struct SharedArrayBuffer {
    pub buf: RefCell<Vec<u8>>,
    pub max_byte_length: Option<usize>,
}

pub enum ArrayBuf {
    Resizable(Rc<ResizableBuffer>),
    Default(Rc<DefaultBuffer>),
    Immutable(Rc<ImmutableBuffer>),
    Shared(Rc<SharedArrayBuffer>),
}