#![feature(unboxed_closures)]
#![allow(unused)]


pub mod context;
pub mod scope;
pub mod statement;
pub mod variable;
mod value;
mod function;

pub use value::*;
pub use function::*;

use swc_ecma_ast::{Script, Stmt};
use yavashark_value::error::Error;


pub enum ControlFlow {
    Continue,
    Break,
    Return(Value),
    Error(Error),
}

impl ControlFlow {
    fn error(e: String) -> Self {
        ControlFlow::Error(Error::new(e))
    }

    fn error_reference(e: String) -> Self {
        ControlFlow::Error(Error::reference(e))
    }
}

type ValueResult = std::result::Result<Value, Error>;

type Result<T> = std::result::Result<T, Error>;

type Res = Result<()>;

type RuntimeResult = std::result::Result<Value, ControlFlow>;


impl From<Error> for ControlFlow {
    fn from(e: Error) -> Self {
        ControlFlow::Error(e)
    }
}

impl From<ControlFlow> for Error {
    fn from(e: ControlFlow) -> Self {
        match e {
            ControlFlow::Error(e) => e,
            _ => Error::new("Incorrect ControlFlow".to_string())
        }
    }
}



pub struct Interpreter {
    script: Vec<Stmt>,
}


impl Interpreter {
    pub fn new(script: Vec<Stmt>) -> Self {
        Self {
            script,
        }
    }

    pub fn run(&self) -> ValueResult {
        let mut context = context::Context::new();
        let mut scope = scope::Scope::new();
        
        context.run_statements(&self.script, &mut scope).or_else(|e| {
            match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            }
        })
    }
}



#[cfg(test)]
mod tests {
    use swc_common::BytePos;
    use swc_common::input::StringInput;
    use swc_ecma_parser::{Parser, Syntax};
    use super::*;
    
    #[test]
    fn math() {
        
        let src = r#"

        let x = 1 + 2

        let y = x + true

        let k = x + y


        if (k > 0) {
            var z = 1337
        } else {
            var z = 42
        }

        log(3)

        z
        "#;
        
        let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();
        
        let mut p = Parser::new(Syntax::Es(c),input, None);
        let script = p.parse_script().unwrap();
        
        let interpreter = Interpreter::new(script.body);
        let result = interpreter.run().unwrap();
        println!("{:?}", result);
    }
    
}


