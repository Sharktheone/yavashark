pub struct SWCTranslator;

impl SWCTranslator {
    pub fn translate(&self, input: &str) {
        let mut parser = swc_ecma_parser::Parser::new(
            swc_ecma_parser::Syntax::Es(swc_ecma_parser::EsSyntax {
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
            }),
            swc_common::input::StringInput::new(
                input,
                swc_common::BytePos(0),
                swc_common::BytePos(input.len() as u32),
            ),
            None,
        );

        let out = parser.parse_program().expect("Failed to parse SWC code");

    }
}
