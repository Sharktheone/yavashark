use yavashark_macro::{object, properties};

use crate::{Context, Error, NativeFunction, Value, ValueResult, Variable, ObjectHandle};
use crate::Symbol;
use crate::object::Object;

#[derive(Debug)]
#[object(direct(iter(Symbol::ITERATOR)))]
struct Array {}



#[properties]
impl Array {
    #[prop]
    fn length(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }
    
    #[prop(Symbol::ITERATOR)]
    fn iterator(&self, args: Vec<Value>) -> ValueResult {
        
        todo!()
    }
    
    
    #[constructor]
    fn construct(&self, args: Vec<Value>) -> ValueResult {
        todo!()
    }
}




#[derive(Debug)]
#[object]
struct ArrayIterator {}