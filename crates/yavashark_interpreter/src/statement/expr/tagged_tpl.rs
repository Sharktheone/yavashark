use crate::location::get_location;
use crate::Interpreter;
use swc_common::Spanned;
use swc_ecma_ast::TaggedTpl;
use yavashark_env::array::Array;
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_tagged_tpl(realm: &mut Realm, stmt: &TaggedTpl, scope: &mut Scope) -> RuntimeResult {
        let tag = Self::run_expr(realm, &stmt.tag, stmt.tag.span(), scope)?;
        let tag = tag.as_object()?;

        let mut exprs = stmt
            .tpl
            .exprs
            .iter()
            .map(|e| Self::run_expr(realm, e, e.span(), scope))
            .collect::<Result<Vec<Value>, ControlFlow>>()?;

        let quasis = stmt
            .tpl
            .quasis
            .iter()
            .map(|q| q.raw.to_string().into())
            .collect::<Vec<Value>>();
        let quasis = Array::with_elements(realm, quasis)?.into_value();

        let mut args = Vec::with_capacity(exprs.len() + 1);

        args.push(quasis);
        args.append(&mut exprs);

        Ok(tag
            .call(realm, args, scope.this()?) //In strict mode, this is undefined
            .map_err(|mut e| {
                e.attach_function_stack(tag.name(), get_location(stmt.span, scope));

                e
            })?)
    }
}
