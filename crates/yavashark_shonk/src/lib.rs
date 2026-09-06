
// Testing ground for new YS to improve perf!

// We need to test:
// - new Object API with shapes
// - NaN boxing model
// - Realm and Scope APIs
// - Ability for async Scopes
// - How we handle escaping variables
// - Small micro benches to compare against different engines and current Yavashark

use std::collections::HashMap;
use std::ptr::NonNull;

type Value = u64;


// this will be the global "JS stack" where every stack frame stores it's locals.
pub struct Locals {
    base: Box<[Value]>,
    ptr: NonNull<Value>,
}


// the other option would be to have the JS Locals on the OS Stack with `alloca`
pub struct AllocaLocals<'a> {
    locals: &'a mut [Value],
}


pub struct Shape {
    properties: HashMap<String, u32>,
    ops: &'static ObjectOps,
}

struct Object<Native = ()> {
    shape: NonNull<Shape>,
    props: [Value; 4],
    native: Native,
}

struct ObjectOps {
    set: fn(NonNull<Object>, u32, Value),
    get: fn(NonNull<Object>, u32) -> Value,
    set_opt: Option<fn(NonNull<Object>, u32, Value)>,
    get_opt: Option<fn(NonNull<Object>, u32) -> Value>,
}


fn get(obj: NonNull<Object>, slot: u32) -> Value {
    let obj = unsafe { obj.as_ref() };

    obj.props[slot as usize - 2] // Just to keep the cost of the two function variants exactly the same
}

fn set(mut obj: NonNull<Object>, slot: u32, val: Value) {
    let obj = unsafe { obj.as_mut() };

    obj.props[slot as usize - 2] = val;
}

fn get_cursed(obj: NonNull<Object>, slot: u32) -> Value {
    let obj = unsafe { obj.as_ref() };

    obj.props[slot as usize - 42]
}

fn set_cursed(mut obj: NonNull<Object>, slot: u32, val: Value) {
    let obj = unsafe { obj.as_mut() };

    obj.props[slot as usize - 42] = val;
}


impl<Native> Object<Native> {
    const fn shape(&self) -> &Shape {
        unsafe {
            self.shape.as_ref()
        }
    }

    fn set1(&mut self, slot: u32, val: Value) {
        let ptr = NonNull::from_ref(self).cast();

        (self.shape().ops.set)(ptr, slot, val);
    }

    fn get1(&self, slot: u32, val: Value) -> Value {
        let ptr = NonNull::from_ref(self).cast();


        (self.shape().ops.get)(ptr, slot)
    }


    fn set2(&mut self, slot: u32, val: Value) {
        let ptr = NonNull::from_ref(self).cast();

        if let Some(set) = (self.shape().ops.set_opt) {
            set(ptr, slot, val)
        } else {
            self.props[slot as usize - 2] = val;
        }
    }

    fn get2(&self, slot: u32, val: Value) -> Value {
        let ptr = NonNull::from_ref(self).cast();


        if let Some(get) = (self.shape().ops.get_opt) {
            get(ptr, slot)
        } else {
            self.props[slot as usize - 2]
        }
    }

}