#![allow(warnings)]

// Testing ground for new YS to improve perf!

// We need to test:
// - new Object API with shapes
// - NaN boxing model
// - Realm and Scope APIs
// - Ability for async Scopes
// - How we handle escaping variables
// - Small micro benches to compare against different engines and current Yavashark

mod object;

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
    kind: ObjectKind,
}

#[derive(Clone, Copy)]
#[repr(usize)]
enum ObjectKind {
    Ordinary,
    Array,
    TypedArray,
    Proxy,
}

// Keep the shared header at the same offsets for every native payload.
#[repr(C)]
struct Object<Native = ()> {
    shape: NonNull<Shape>,
    kind: ObjectKind,
    props: [Value; 4],
    native: Native,
}

// ObjectOps:
// - We'll have two kinds Object Offsets: regular and native
//      - The regular ones are just a offset in the properties table
//      - The native are just for identification for the native part when matching against special properties such as `length`
//      - An Object might decide to give no regular ones out if it chooses to (proxy)
//      - Lazy Properties are initialized the property is requested in the shape, then they become just regular old properties

struct ObjectOps {
    set: fn(NonNull<Object>, u32, Value),
    get: fn(NonNull<Object>, u32) -> Value,
    set_opt: Option<fn(NonNull<Object>, u32, Value)>,
    get_opt: Option<fn(NonNull<Object>, u32) -> Value>,
}

#[inline(always)]
fn get(obj: NonNull<Object>, slot: u32) -> Value {
    let obj = unsafe { obj.as_ref() };

    obj.props[slot as usize - 2] // Just to keep the cost of the two function variants exactly the same
}

fn set(mut obj: NonNull<Object>, slot: u32, val: Value) {
    let obj = unsafe { obj.as_mut() };

    obj.props[slot as usize - 2] = val;
}

#[inline(always)]
fn get_cursed(obj: NonNull<Object>, slot: u32) -> Value {
    let obj = unsafe { obj.as_ref() };

    obj.props[slot as usize - 42]
}

fn set_cursed(mut obj: NonNull<Object>, slot: u32, val: Value) {
    let obj = unsafe { obj.as_mut() };

    obj.props[slot as usize - 42] = val;
}

// Slot 0 stands in for an array's length property in this mock.
#[inline(always)]
fn get_array(obj: NonNull<Object>, slot: u32) -> Value {
    let obj = unsafe { obj.as_ref() };
    if slot == 0 {
        obj.props.len() as Value
    } else {
        obj.props[slot as usize - 2]
    }
}

#[inline(always)]
fn get_proxy(obj: NonNull<Object>, slot: u32) -> Value {
    let obj = unsafe { obj.as_ref() };
    obj.props[slot as usize - 82]
}

// Closed set of kinds dispatched through function pointers.
static GET_BY_KIND: [fn(NonNull<Object>, u32) -> Value; 4] =
    [get, get_array, get_cursed, get_proxy];

impl<Native> Object<Native> {
    const fn shape(&self) -> &Shape {
        unsafe { self.shape.as_ref() }
    }

    fn set1(&mut self, slot: u32, val: Value) {
        let ptr = NonNull::from_ref(self).cast();

        (self.shape().ops.set)(ptr, slot, val);
    }

    fn get1(&self, slot: u32) -> Value {
        let ptr = NonNull::from_ref(self).cast();

        (self.shape().ops.get)(ptr, slot)
    }

    fn set2(&mut self, slot: u32, val: Value) {
        let ptr = NonNull::from_ref(self).cast();

        if let Some(set) = self.shape().ops.set_opt {
            set(ptr, slot, val)
        } else {
            self.props[slot as usize - 2] = val;
        }
    }

    fn get2(&self, slot: u32) -> Value {
        let ptr = NonNull::from_ref(self).cast();

        if let Some(get) = self.shape().ops.get_opt {
            get(ptr, slot)
        } else {
            self.props[slot as usize - 2]
        }
    }

    fn get3(&self, slot: u32) -> Value {
        if let ObjectKind::Ordinary = self.shape().kind {
            self.props[slot as usize - 2]
        } else {
            (self.shape().ops.get)(NonNull::from_ref(self).cast(), slot)
        }
    }

    fn get4(&self, slot: u32) -> Value {
        match self.shape().kind {
            ObjectKind::Ordinary => self.props[slot as usize - 2],
            ObjectKind::Array => {
                if slot == 0 {
                    self.props.len() as Value
                } else {
                    self.props[slot as usize - 2]
                }
            }
            _ => (self.shape().ops.get)(NonNull::from_ref(self).cast(), slot),
        }
    }

    // Closed set of kinds, with every handler inlined into the dispatch.
    #[inline(always)]
    fn get5(&self, slot: u32) -> Value {
        let ptr = NonNull::from_ref(self).cast();
        match self.shape().kind {
            ObjectKind::Ordinary => get(ptr, slot),
            ObjectKind::Array => get_array(ptr, slot),
            ObjectKind::TypedArray => get_cursed(ptr, slot),
            ObjectKind::Proxy => get_proxy(ptr, slot),
        }
    }

    fn get6(&self, slot: u32) -> Value {
        GET_BY_KIND[self.shape().kind as usize](NonNull::from_ref(self).cast(), slot)
    }

    // Same fast paths as get4, but the tag needs no shape dereference.
    fn get7(&self, slot: u32) -> Value {
        match self.kind {
            ObjectKind::Ordinary => self.props[slot as usize - 2],
            ObjectKind::Array => {
                if slot == 0 {
                    self.props.len() as Value
                } else {
                    self.props[slot as usize - 2]
                }
            }
            _ => (self.shape().ops.get)(NonNull::from_ref(self).cast(), slot),
        }
    }
}
