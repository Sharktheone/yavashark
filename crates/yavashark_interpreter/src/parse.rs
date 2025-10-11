use swc_common::input::StringInput;
use swc_common::util::take::Take;
use swc_common::BytePos;
use swc_ecma_ast::{Module, Program, Script};
use swc_ecma_parser::{EsSyntax, PResult, Parser, Syntax};

pub fn parse_module(input: &str) -> PResult<Module> {
    if input.is_empty() {
        return Ok(Module::dummy());
    }

    let end = BytePos(input.len() as u32);

    let input = StringInput::new(input, BytePos(0), end);

    let mut p = Parser::new(Syntax::Es( EsSyntax {
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
        }), input, None);

    p.parse_module()
}
