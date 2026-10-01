use crate::Interpreter;
use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::realm::Eval;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Value, ValueResult};
use yavashark_swc_validator::Validator;

pub struct InterpreterEval;

impl Eval for InterpreterEval {
    fn eval(&self, code: &str, realm: &mut Realm, scope: &mut Scope) -> ValueResult {
        if code.is_empty() {
            return Ok(Value::Undefined);
        }

        let input = StringInput::new(code, BytePos(0), BytePos(code.len() as u32));
        let syn = Syntax::Es(EsSyntax {
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

        let mut p = Parser::new(syn, input, None);

        let script = match p.parse_script() {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::syn_error(format!("{e:?}")));
            }
        };

        let mut validator = Validator::new();

        if scope.is_strict_mode()? {
            validator.enable_script_strict_mode();
        }

        let allow_new_target = scope.allows_new_target()?;

        if allow_new_target {
            validator.enable_new_target();
        }

        if let Err(e) = validator.validate_statements(&script.body) {
            return Err(Error::syn_error(e.to_string()));
        }

        let mut eval_scope = scope.child()?;

        if scope.is_strict_mode()? || Interpreter::is_strict(&script.body) {
            eval_scope.set_strict_mode()?;
            eval_scope.state_set_function()?;
            eval_scope.set_new_target_allowed(allow_new_target)?;
        }

        Interpreter::run_in(&script.body, realm, &mut eval_scope)
    }
}
