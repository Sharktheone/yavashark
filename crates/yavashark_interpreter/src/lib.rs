#![allow(unused)]


mod context;
mod scope;
mod statement;

use swc_ecma_ast::{Script, Stmt};
use yavashark_value::error::Error;
use yavashark_value::Value;

pub struct Interpreter {
    script: Vec<Stmt>,
}


impl Interpreter {
    pub fn new(script: Vec<Stmt>) -> Self {
        Self {
            script,
        }
    }

    pub fn run(&self) -> Result<Value, Error> {
        let mut context = context::Context::new();
        let mut scope = scope::Scope::new();
        context.run_statements(&self.script, &mut scope)
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
        
        if (false) {
            1 + 2;
        } else {
        2-3;
        }
        
        "#;
        
        let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

        let c = Default::default();
        
        let mut p = Parser::new(Syntax::Es(c),input, None);
        let script = p.parse_script().unwrap();
        
        let interpreter = Interpreter::new(script.body);
        let result = interpreter.run().unwrap();
        println!("{:?}", result);
        assert_eq!(result, Value::Number(3.0));
    }
    
}


