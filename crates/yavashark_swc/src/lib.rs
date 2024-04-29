
#[test]
fn test() {
    use swc_common::BytePos;
    use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
    let src = r#"
    try {}
    catch ({ name, message: m }) {}
    "#;

    let c = TsConfig {
        ..Default::default()
    };

    let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32 - 1));

    let mut p = Parser::new(Syntax::Typescript(c), input, None);
    let prog = p.parse_program().unwrap();

    println!("{:#?}", prog);
}
