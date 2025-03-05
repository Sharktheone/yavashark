use crate::Interpreter;
use swc_ecma_ast::{MetaPropExpr, MetaPropKind};
use swc_ecma_ast::TsKeywordTypeKind::TsObjectKeyword;
use yavashark_env::scope::Scope;
use yavashark_env::{NativeFunction, Object, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_meta_prop(
        realm: &mut Realm,
        stmt: &MetaPropExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        Ok(match stmt.kind {
            MetaPropKind::NewTarget => scope.get_target()?,
            MetaPropKind::ImportMeta => {
                let obj = Object::with_proto(Value::Null);
                
                obj.define_property(
                    "url".into(),
                    scope.get_current_path()?.to_string_lossy().into_owned().into(),
                    
                )?;
                
                obj.define_property("resolve".into(), Value::Undefined)?; //TODO
                
                
                obj.into()
            }
        })
    }
}
