#![allow(unused)]

mod console;
pub mod context;
mod function;
pub mod scope;
pub mod statement;
mod value;
pub mod variable;

pub use function::*;
pub use value::*;

use swc_ecma_ast::{Script, Stmt};
use yavashark_value::error::Error;

pub enum ControlFlow {
    Continue(Option<String>),
    Break(Option<String>),
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
    
    fn syntax_error(e: &str) -> Self {
        ControlFlow::Error(Error::syntax(e))
    }
    
    fn get_error(self) -> std::result::Result<Error, ControlFlow> {
        match self {
            ControlFlow::Error(e) => Ok(e),
            (e) => Err(e),
        }
   
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
            _ => Error::new("Incorrect ControlFlow".to_string()),
        }
    }
}

pub struct Interpreter {
    script: Vec<Stmt>,
}

impl Interpreter {
    pub fn new(script: Vec<Stmt>) -> Self {
        Self { script }
    }

    pub fn run(&self) -> ValueResult {
        let mut context = context::Context::new();
        let mut scope = scope::Scope::new();

        context
            .run_statements(&self.script, &mut scope)
            .or_else(|e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_common::input::StringInput;
    use swc_common::BytePos;
    use swc_ecma_parser::{Parser, Syntax};

    #[test]
    fn math() {
        let src = r#"

        let x = 1 + 2

        let y = x + true

        let k = x + y


        function hello(a, b) {
            return a + b
        }

        if (k > 0) {
            var z = 1337
        } else {
            var z = 42
        }

        console.log(3, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
        console.log("3+4 is", hello(3, 4))


        let yyy = 1 + 2

        switch (yyy) {
            case 1:
                console.log("one")
                break
            case 2:
                console.log("two")
                break
            case 3:
                console.log("three")
            case 4:
                console.log("four")
            case 5:
                console.log("five")
            default:
                console.log("default")
        }

        z
        "#;

        let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();

        let mut p = Parser::new(Syntax::Es(c), input, None);
        let script = p.parse_script().unwrap();

        let interpreter = Interpreter::new(script.body);
        let result = interpreter.run().unwrap();
        println!("{:?}", result);
    }
}
