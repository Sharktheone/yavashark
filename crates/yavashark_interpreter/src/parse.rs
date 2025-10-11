use swc_common::input::StringInput;
use swc_common::util::take::Take;
use swc_common::BytePos;
use swc_ecma_ast::{Module, Program, Script};
use swc_ecma_parser::{EsSyntax, PResult, Parser, Syntax};
use yavashark_env::{Error, Res};
use yavashark_swc_validator::Validator;

pub fn parse_module(input: &str) -> Res<Module> {
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

    let m =  p.parse_module()
        .map_err(|e| Error::syn_error(format!("{e:?}")))?;


    let errors = p.take_errors();

    if !errors.is_empty() {
        return Err(Error::syn_error(format!("Parse errors: {errors:?}")));
    }

    if let Err(e) = Validator::new().validate_module_items(&m.body) {
        return Err(Error::syn_error(e));

    }



    Ok(m)
}
