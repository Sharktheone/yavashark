use std::mem;
use swc_ecma_ast::Pass;
use std::rc::Rc;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::sync::Lrc;
use swc_common::{FileName, Globals, Mark, SourceFile, SourceMap, GLOBALS};
use swc_common::comments::NoopComments;
use swc_ecma_ast::{Program, Script};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{EsSyntax, PResult, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_transforms_typescript::{Config, TsxConfig};

#[allow(unused)]
const SYNTAX_ES: Syntax = Syntax::Es(EsSyntax {
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

const SYNTAX_TS: Syntax = Syntax::Typescript(TsSyntax {
    tsx: true,
    decorators: true,
    dts: false,
    no_early_errors: false,
    disallow_ambiguous_jsx_like: false,
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
    let mut p = Parser::new(SYNTAX_TS, StringInput::from(&*fm), None);
    let res = p.parse_script();

    let mut res = emit(Rc::clone(&cm), &mut p, res);


    if let Some(script) = res.as_mut() {
        let s = mem::take(script);

        let mut p = Program::Script(s);

        strip_ts(&mut p, cm);

        if let Program::Script(s) = p {
            *script = s;
        } else {
            unreachable!("Failed to convert Program::Script back to Script");
        }
    }

    res
}

pub fn parse_program(code: &str, name: &str) -> Option<Program> {
    let (cm, fm) = source(code, name);
    let mut p = Parser::new(SYNTAX_TS, StringInput::from(&*fm), None);
    let res = p.parse_program();

    let mut res = emit(Rc::clone(&cm), &mut p, res);

    if let Some(p) = res.as_mut() {
        strip_ts(p, cm);
    }


    res
}


fn strip_ts(prog: &mut Program, cm: Rc<SourceMap>) {
    GLOBALS.set(&Globals::default(), || {
        swc_ecma_transforms_typescript::tsx(
            Rc::clone(&cm),
            Config::default(),
            TsxConfig::default(),
            NoopComments,
            Mark::new(),
            Mark::new()
        ).process(prog);

        swc_ecma_transforms_typescript::strip(Mark::new(), Mark::new())
            .process(prog);

        swc_ecma_transforms_react::jsx(
            cm,
            None::<NoopComments>,
            swc_ecma_transforms_react::Options {
                runtime: Some(swc_ecma_transforms_react::Runtime::Classic),
                ..Default::default()
            },
            Mark::new(),
            Mark::new(),
        )
        .process(prog);
    });
}
