use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_parser::{EsConfig, EsSyntax, Parser, Syntax};
use yavashark_bytecode::Instruction;

mod statement;

struct ByteCodegen {
    instructions: Vec<Instruction>
}




#[test]
fn test_compile() {
    let src = r#"
    console.log("Hello, World!");
 "#;
    
    let input = StringInput::new(src.into(), BytePos(0), BytePos(src.len() as u32));
    
    let c = EsSyntax::default();
    
    let mut p = Parser::new(Syntax::Es(c), input, None);
    let script = p.parse_script().unwrap();
    
    let mut bc = ByteCodegen {
        instructions: vec![]
    };
    
    bc.compile_statements(&script.body);
}