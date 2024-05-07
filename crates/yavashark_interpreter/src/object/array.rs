use yavashark_macro::{object, properties};
use crate::{Context, Variable, Value};
use crate::object::Object;
use crate::Symbol;



#[derive(Debug)]
#[object(direct(iter(Symbol::ITERATOR)))]
struct Array {}




#[properties]
impl Array {
    
    #[prop]
    fn length(&self) -> Value {
        todo!()
    }
    
}

#[derive(Debug)]
#[object]
struct ArrayIterator {}