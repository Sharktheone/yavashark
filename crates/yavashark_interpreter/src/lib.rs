#![allow(unused, clippy::needless_pass_by_ref_mut)] //pass by ref mut is just temporary until all functions are implemented

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use swc_ecma_ast::Stmt;
use yavashark_env::{scope, Context, ControlFlow, Value, ValueResult};
mod class;
mod function;
mod pat;
pub mod statement;

pub struct Interpreter;

impl Interpreter {
    pub fn run(script: &Vec<Stmt>) -> anyhow::Result<Value> {
        let mut context = &mut Context::new()?;
        let mut scope = scope::Scope::global(context);

        Self::run_statements(context, script, &mut scope)
            .or_else(|e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            })
            .map_err(|e| anyhow!("{e:?}"))
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    pub fn run_test(script: &Vec<Stmt>) -> (ValueResult, Rc<RefCell<yavashark_env::tests::State>>) {
        let mut context = &mut Context::new().unwrap();
        let mut scope = scope::Scope::global(context);

        let (mock, state) = yavashark_env::tests::mock_object(context);

        scope.declare_global_var("mock".into(), mock);

        (
            Self::run_statements(context, script, &mut scope).or_else(|e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            }),
            state,
        )
    }
}

#[cfg(test)]
mod temp_test {
    use env_logger::Logger;
    use std::thread;
    use swc_common::input::StringInput;
    use swc_common::BytePos;
    use swc_ecma_parser::{EsConfig, Parser, Syntax};

    use super::*;

    #[test]
    fn math() {
        {
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
        // 
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
        
        
        let array = new Array(4, 5, 6, 7)
        
        
        console.log(array[0], array[1], array[2], array[3])
        
        for (let i in array) {
            console.log("in", i)
        }
        
        for (let i of array) {
            console.log("of", i)
        }
        
        
        
        let arrow = (a, b) => {
            return a + b
        }
        
        console.log("arrow", arrow(1, 2))
        
        function Arrows() {
            this.x = "hello"
            this.y = "world"
            
            this.arrow = (a, b) => {
                console.log("from_arrows", this.x, this.y)
                return a + b
            }
        }
        
        let arr = new Arrows()
        
        console.log("arrow", arr.arrow(1, 2))
        
        
        // console.log(Array) //TODO: this causes an stack overflow (probably because of the prototype)
        
        
        let lit_array = [1,2,3,,4,5,6,7,,,8,9,10]
        
        console.log(lit_array)
        
        let obj = {
            x: 11,
            y: 22,
            z: 33
        }
        
        console.log(obj)
        console.log(obj.x, obj.y, obj.z)
        
        
        let x = function() {
        console.log("hello")
        }
        
        x()
        
        
        
        class Hello {
            constructor() {
                this.x = 1
                this.y = 2
            }
            
            hello() {
                console.log("hello")
            }
            
            
            static staticHello() {
                console.log("static hello")
            }
        }
        
        Hello.staticHello()
        
        let h = new Hello() //constructor is wrong somehow
        
        // h.hello()

        z
        "#;

            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Warn)
                .init();

            let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

            let c = EsConfig::default();

            let mut p = Parser::new(Syntax::Es(c), input, None);
            let script = p.parse_script().unwrap();

            let result = Interpreter::run(&script.body).unwrap();

            println!("{result:?}");
        }

        // #[cfg(not(miri))]
        // std::thread::sleep(std::time::Duration::from_secs(20));

        println!(
            "LEAKED OBJECTS: {}/{}",
            yavashark_value::OBJECT_COUNT.get(),
            yavashark_value::OBJECT_ALLOC.get()
        );
    }
}
