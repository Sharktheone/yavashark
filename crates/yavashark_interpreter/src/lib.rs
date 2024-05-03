#![allow(unused)]

use swc_ecma_ast::Stmt;

use crate::context::Context;
pub use function::*;

mod console;
pub mod context;
mod error;
mod function;
mod object;
pub mod scope;
pub mod statement;
pub mod variable;

type Value = yavashark_value::Value<Context>;
type Error = yavashark_value::Error<Context>;
type Function = yavashark_value::Function<Context>;
type Object = yavashark_value::Object<Context>;

pub enum ControlFlow {
    Continue(Option<String>),
    Break(Option<String>),
    Return(Value),
    Error(Error),
}

impl ControlFlow {
    fn error(e: String) -> Self {
        ControlFlow::Error(Error::new_error(e))
    }

    fn error_reference(e: String) -> Self {
        ControlFlow::Error(Error::reference_error(e))
    }
    fn error_syntax(e: &str) -> Self {
        ControlFlow::Error(Error::syn(e))
    }
    fn error_type(e: String) -> Self {
        ControlFlow::Error(Error::ty(e))
    }

    fn get_error(self) -> std::result::Result<Error, ControlFlow> {
        match self {
            ControlFlow::Error(e) => Ok(e),
            (e) => Err(e),
        }
    }

    fn throw(val: Value) -> Self {
        ControlFlow::Error(Error::throw(val))
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
            _ => Error::new("Incorrect ControlFlow"),
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
        let mut context = &mut Context::new();
        let mut scope = scope::Scope::global(context);

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
    use swc_common::input::StringInput;
    use swc_common::BytePos;
    use swc_ecma_parser::{Parser, Syntax};

    use super::*;

    #[test]
    fn math() {
        let src = r#"

        let x = 1 + 2

        let y = x + true

        let k = x + y
        
        try { 
            console.log(k)
            k = k + 2
            console.log(k)
        } catch (e) {
            console.log("i don't care")
        }

        let z = 69;
        function hello(a, b) {
            return a + b
        }

        if (k > 0) {
            z = 1337
        } else {
            z = 42
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
        
        for (let i = 0; i < 10; i++) {
            console.log(i)
        }

        console.log(this)


        function Hello() {
            this.x = 1
            this.y = 2
        }


        console.log(new Hello())


        try {
            throw 1
        } catch ({message}) {
            console.log("error:", message)
        }

        let a = 1
        while (a < 10) {
            console.log("infinite loop")
            a++;
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
