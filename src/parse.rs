use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceFile, SourceMap};
use swc_ecma_ast::{Program, Script};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{EsSyntax, PResult, Parser, StringInput, Syntax};

const SYNTAX: Syntax = Syntax::Es(EsSyntax {
    jsx: false,
    fn_bind: false,
    decorators: true,
    decorators_before_export: true,
    export_default_from: true,
    import_attributes: true,
    allow_super_outside_method: false,
    allow_return_outside_function: false,
    auto_accessors: true,
    explicit_resource_management: true,
});

fn source(code: &str, name: &str) -> (Lrc<SourceMap>, Lrc<SourceFile>) {
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(Lrc::new(FileName::Custom(name.to_owned())), code.to_owned());

    (cm, fm)
}

fn emit<T>(cm: Lrc<SourceMap>, p: &mut Parser<Lexer<'_>>, res: PResult<T>) -> Option<T> {
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm));

    for e in p.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    res.map_err(|e| e.into_diagnostic(&handler).emit()).ok()
}

pub fn parse_script(code: &str, name: &str) -> Option<Script> {
    let (cm, fm) = source(code, name);
    let mut p = Parser::new(SYNTAX, StringInput::from(&*fm), None);
    let res = p.parse_script();

    emit(cm, &mut p, res)
}

pub fn parse_program(code: &str, name: &str) -> Option<Program> {
    let (cm, fm) = source(code, name);
    let mut p = Parser::new(SYNTAX, StringInput::from(&*fm), None);
    let res = p.parse_program();

    emit(cm, &mut p, res)
}
