use std::collections::HashMap;
use yavashark_value::Value;
use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;


pub struct Scope {
    scope: Rc<NonNull<UnsafeScope>>,
}


struct UnsafeScope {
    parent: Option<Rc<NonNull<UnsafeScope>>>,
    variables: HashMap<String, Value>,
}

impl UnsafeScope {
    pub fn new() -> NonNull<Self>{
        let ptr = Box::new(Self {
            parent: None,
            variables: HashMap::new(),
        });

        //Box::into_raw always returns a non-null pointer (except it was created unsafely, which is not the case here)
        NonNull::new(Box::into_raw(ptr)).expect("inserted known good pointer into NonNull but it apparently was null")
    }
    
    pub fn with_parent(parent: Rc<NonNull<UnsafeScope>>) -> NonNull<Self> {
        let ptr = Box::new(Self {
            parent: Some(parent),
            variables: HashMap::new(),
        });
        
        //Box::into_raw always returns a non-null pointer (except it was created unsafely, which is not the case here)
        NonNull::new(Box::into_raw(ptr)).expect("inserted known good pointer into NonNull but it apparently was null")
    }

}


impl Scope {
    pub fn new() -> Self {
        Self {
            scope: Rc::new(UnsafeScope::new()),
        }
    }
    
    pub fn with_parent(parent: &Scope) -> Self {
        Self {
            scope: Rc::new(UnsafeScope::with_parent(Rc::clone(&parent.scope))),
        }
    }
    
    pub fn clone(scope: &Self) -> Self {
        Self {
            scope: Rc::clone(&scope.scope),
        }
    }
}