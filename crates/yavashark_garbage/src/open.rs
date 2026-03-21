use std::ptr::NonNull;
use std::rc::Rc;

pub struct Agent;

pub trait OpenHandle<'a, R: 'a> {
    fn open(self, agent: &'a Agent) -> R;
}

pub trait OpenHandleMut<T> {
    fn open_mut(self, agent: &mut Agent) -> &mut T;
}

impl<'a, T> OpenHandle<'a, &'a T> for Rc<T> {
    fn open(self, _agent: &'a Agent) -> &'a T {
        todo!()
    }
}

impl<'a, T> OpenHandle<'a, Self> for Rc<T>
where
    T: 'a,
{
    fn open(self, _agent: &'a Agent) -> Self {
        todo!()
    }
}

impl<T> OpenHandleMut<T> for Rc<T> {
    fn open_mut(self, _agent: &mut Agent) -> &mut T {
        todo!()
    }
}

pub struct Handle<T> {
    inner: NonNull<T>,
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}


impl<'a, T> OpenHandle<'a, &'a T> for Handle<T> {
    fn open(self, _agent: &'a Agent) -> &'a T {
        todo!()
    }
}

impl<'a, T> OpenHandle<'a, Self> for Handle<T>
where
    T: 'a,
{
    fn open(self, _agent: &'a Agent) -> Self {
        todo!()
    }
}

impl<T> OpenHandleMut<T> for Handle<T> {
    fn open_mut(self, _agent: &mut Agent) -> &mut T {
        todo!()
    }
}

pub struct Consumer;

impl Consumer {
    #[must_use]
    pub const fn a(&self) -> i32 {
        42
    }

    #[must_use]
    pub const fn b(&self, agent: &Agent) -> i32 {
        24
    }

    pub fn c(&mut self) -> i32 {
        12
    }

    pub fn d(self: Rc<Self>, agent: &mut Agent) -> i32 {
        6
    }

    pub fn e(this: Handle<Self>, agent: &mut Agent) -> i32 {
        let inner = &raw const this;

        3
    }

    // pub fn f(self: *mut Self, agent: &mut Agent) -> i32 {
    //     1
    // }
}

fn test(handle: Rc<Consumer>, agent: &mut Agent) {
    let consumer = handle.clone().open(agent);
    println!("a: {}", Consumer::a(consumer));

    let consumer = handle.clone().open(agent);
    println!("b: {}", Consumer::b(consumer, agent));

    let consumer = handle.clone().open_mut(agent);
    println!("c: {}", Consumer::c(consumer));

    let consumer = handle.open(agent);
    println!("d: {}", Consumer::d(consumer, agent));
}


fn test_handle(handle: Handle<Consumer>, agent: &mut Agent) {
    let consumer = handle.open(agent);
    println!("a: {}", Consumer::a(consumer));

    let consumer = handle.open(agent);
    println!("b: {}", Consumer::b(consumer, agent));

    let consumer = handle.open_mut(agent);
    println!("c: {}", Consumer::c(consumer));

    let consumer = handle.open(agent);
    println!("d: {}", Consumer::e(consumer, agent));
}
