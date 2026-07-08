use crate::Interpreter;
use log::info;
use std::any::Any;
use std::cell::RefCell;
use std::iter;
use swc_ecma_ast::{BlockStmt, Callee, Expr, MemberProp, Param, Pat, Stmt};
use yavashark_env::array::Array;
use yavashark_env::builtins::Arguments;
use yavashark_env::optimizer::FunctionCode;
use yavashark_env::realm::Realm;
use yavashark_env::scope::Scope;
use yavashark_env::value::{
    BoxedObj, Constructor, ConstructorFn, CustomGcRefUntyped, CustomName, Func, Obj, ObjectProperty,
};
use yavashark_env::{
    ControlFlow, Error, MutObject, Object, ObjectHandle, Res, RuntimeResult, Value, ValueResult,
    Variable,
};
use yavashark_garbage::{Collectable, GcRef};
use yavashark_macro::object;
use yavashark_string::YSString;

#[allow(clippy::module_name_repetitions)]
#[object(function, constructor, direct(prototype), name)]
#[derive(Debug)]
pub struct JSFunction {
    // #[gc(untyped)] //TODO: this is a memleak!
    pub raw: RawJSFunction,
}

#[derive(Debug)]
pub struct RawJSFunction {
    pub name: RefCell<String>,
    pub params: Vec<Param>,
    pub block: Option<BlockStmt>,
    pub scope: Scope,
    pub is_strict: bool,
    pub needs_arguments: bool,
}

#[derive(Debug)]
pub struct OptimizedJSFunction {
    pub block: BlockStmt,
}

impl FunctionCode for OptimizedJSFunction {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, this: Value) -> RuntimeResult {
        Interpreter::run_block_this(realm, &self.block, scope, this)
    }

    fn function_any(&self) -> &dyn Any {
        self
    }
}

impl CustomName for JSFunction {
    fn custom_name(&self) -> String {
        self.raw.name.borrow().clone()
    }
}

impl JSFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        params: Vec<Param>,
        block: Option<BlockStmt>,
        scope: Scope,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let prototype = Object::new(realm);

        scope.copy_path();

        let len = params.last().map_or(0, |last| {
            if last.pat.is_rest() {
                params.len() - 1
            } else {
                params.len()
            }
        });

        let is_strict = scope.is_strict_mode()?
            || block
                .as_ref()
                .is_some_and(|b| Interpreter::is_strict(&b.stmts));

        let this = Self {
            inner: RefCell::new(MutableJSFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
                prototype: prototype.clone().into(),
            }),
            raw: RawJSFunction {
                name: RefCell::new(name.clone()),
                params,
                needs_arguments: block.as_ref().is_some_and(block_needs_arguments),
                block,
                scope,
                is_strict,
            },
        };

        if !is_strict {
            this.define_property_attributes(
                "caller".into(),
                Variable::new_read_only(Value::Undefined),
                realm,
            )?;
        }

        let handle = ObjectHandle::new(this);

        handle.define_property_attributes("name".into(), Variable::config(name.into()), realm)?;
        handle.define_property_attributes("length".into(), Variable::config(len.into()), realm)?;
        prototype.define_property_attributes(
            "constructor".into(),
            Variable::write_config(handle.clone().into()),
            realm,
        );

        Ok(handle)
    }

    pub fn update_name(&self, n: &str) -> Res {
        let mut name = self.raw.name.try_borrow_mut()?;

        if name.is_empty() {
            n.clone_into(&mut name);

            self.inner
                .try_borrow_mut()?
                .object
                .force_update_property_cb("name".into(), |v| {
                    if let Some(v) = v
                        && !v.value.is_string()
                    {
                        return None;
                    }

                    Some(YSString::from_ref(n).into())
                })?;
        }

        Ok(())
    }

    pub fn new_instance(&self, realm: &mut Realm) -> ValueResult {
        let inner = self.inner.try_borrow()?;

        let proto = inner.prototype.value.clone().to_object()?;

        let obj = Object::with_proto(proto);

        obj.set(
            "name",
            Value::String(self.raw.name.borrow().clone().into()),
            realm,
        )?;

        Ok(obj.into())
    }
}

impl Func for JSFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        yavashark_env::profiler::profile_call(
            realm,
            || self.name(),
            |realm| self.raw.call(realm, args, this),
        )
    }
}

impl RawJSFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        let arguments_args = self.needs_arguments.then(|| args.clone());

        let scope = &mut Scope::with_parent_this(&self.scope, this.copy())?;
        if self.is_strict {
            scope.set_strict_mode();
        }

        scope.state_set_function();
        scope.state_set_returnable();

        let mut iter = args.into_iter();

        for p in &self.params {
            Interpreter::run_pat(
                realm,
                &p.pat,
                scope,
                &mut iter,
                &mut |scope, name, value, realm| {
                    scope.declare_var(name, value, realm);
                    Ok(())
                },
            )?;
        }

        if let Some(arguments_args) = arguments_args {
            let caller = if scope.is_strict_mode()? {
                None
            } else {
                Some(this.copy())
            };

            let args = Arguments::new(arguments_args, caller, realm)?;
            let args = ObjectHandle::new(args);

            scope.declare_var("arguments".to_string(), args.into(), realm);
        }

        if let Some(block) = &self.block
            && let Err(e) = Interpreter::run_block(realm, block, scope)
        {
            return match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                ControlFlow::Break(_) => Err(Error::syn("Illegal break statement")),
                ControlFlow::Continue(_) => Err(Error::syn("Illegal continue statement")),
                ControlFlow::Yield(_) | ControlFlow::YieldStar(_) => {
                    Err(Error::syn("Illegal yield statement"))
                }
                ControlFlow::Await(_) => Err(Error::syn("Illegal await statement")),
                ControlFlow::OptChainShortCircuit => Ok(Value::Undefined),
            };
        }

        Ok(Value::Undefined)
    }
}

pub(crate) fn block_needs_arguments(block: &BlockStmt) -> bool {
    block.stmts.iter().any(stmt_needs_arguments)
}

fn stmt_needs_arguments(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Block(block) => block_needs_arguments(block),
        Stmt::Expr(stmt) => expr_needs_arguments(&stmt.expr),
        Stmt::If(stmt) => {
            expr_needs_arguments(&stmt.test)
                || stmt_needs_arguments(&stmt.cons)
                || stmt.alt.as_deref().is_some_and(stmt_needs_arguments)
        }
        Stmt::While(stmt) => expr_needs_arguments(&stmt.test) || stmt_needs_arguments(&stmt.body),
        Stmt::DoWhile(stmt) => stmt_needs_arguments(&stmt.body) || expr_needs_arguments(&stmt.test),
        Stmt::For(stmt) => {
            stmt.init.as_ref().is_some_and(|init| match init {
                swc_ecma_ast::VarDeclOrExpr::VarDecl(var) => var.decls.iter().any(|decl| {
                    decl.init
                        .as_ref()
                        .is_some_and(|expr| expr_needs_arguments(expr))
                }),
                swc_ecma_ast::VarDeclOrExpr::Expr(expr) => expr_needs_arguments(expr),
            }) || stmt
                .test
                .as_ref()
                .is_some_and(|expr| expr_needs_arguments(expr))
                || stmt
                    .update
                    .as_ref()
                    .is_some_and(|expr| expr_needs_arguments(expr))
                || stmt_needs_arguments(&stmt.body)
        }
        Stmt::ForIn(stmt) => expr_needs_arguments(&stmt.right) || stmt_needs_arguments(&stmt.body),
        Stmt::ForOf(stmt) => expr_needs_arguments(&stmt.right) || stmt_needs_arguments(&stmt.body),
        Stmt::Return(stmt) => stmt
            .arg
            .as_ref()
            .is_some_and(|expr| expr_needs_arguments(expr)),
        Stmt::Throw(stmt) => expr_needs_arguments(&stmt.arg),
        Stmt::Switch(stmt) => {
            expr_needs_arguments(&stmt.discriminant)
                || stmt.cases.iter().any(|case| {
                    case.test
                        .as_ref()
                        .is_some_and(|expr| expr_needs_arguments(expr))
                        || case.cons.iter().any(stmt_needs_arguments)
                })
        }
        Stmt::Try(stmt) => {
            block_needs_arguments(&stmt.block)
                || stmt
                    .handler
                    .as_ref()
                    .is_some_and(|handler| block_needs_arguments(&handler.body))
                || stmt.finalizer.as_ref().is_some_and(block_needs_arguments)
        }
        Stmt::Decl(decl) => match decl {
            swc_ecma_ast::Decl::Var(var) => var.decls.iter().any(|decl| {
                decl.init
                    .as_ref()
                    .is_some_and(|expr| expr_needs_arguments(expr))
            }),
            swc_ecma_ast::Decl::Fn(_) | swc_ecma_ast::Decl::Class(_) => false,
            _ => true,
        },
        Stmt::Labeled(stmt) => stmt_needs_arguments(&stmt.body),
        Stmt::With(_) => true,
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Debugger(_) | Stmt::Empty(_) => false,
    }
}

fn expr_needs_arguments(expr: &Expr) -> bool {
    match expr {
        Expr::Ident(ident) => ident.sym == *"arguments" || ident.sym == *"eval",
        Expr::This(_) | Expr::Lit(_) | Expr::PrivateName(_) | Expr::SuperProp(_) => false,
        Expr::Array(expr) => expr
            .elems
            .iter()
            .flatten()
            .any(|elem| expr_needs_arguments(&elem.expr)),
        Expr::Object(expr) => expr.props.iter().any(|prop| match prop {
            swc_ecma_ast::PropOrSpread::Spread(spread) => expr_needs_arguments(&spread.expr),
            swc_ecma_ast::PropOrSpread::Prop(prop) => match &**prop {
                swc_ecma_ast::Prop::Shorthand(ident) => {
                    ident.sym == *"arguments" || ident.sym == *"eval"
                }
                swc_ecma_ast::Prop::KeyValue(kv) => {
                    prop_name_needs_arguments(&kv.key) || expr_needs_arguments(&kv.value)
                }
                swc_ecma_ast::Prop::Assign(assign) => expr_needs_arguments(&assign.value),
                swc_ecma_ast::Prop::Getter(getter) => prop_name_needs_arguments(&getter.key),
                swc_ecma_ast::Prop::Setter(setter) => prop_name_needs_arguments(&setter.key),
                swc_ecma_ast::Prop::Method(method) => prop_name_needs_arguments(&method.key),
            },
        }),
        Expr::Unary(expr) => expr_needs_arguments(&expr.arg),
        Expr::Update(expr) => expr_needs_arguments(&expr.arg),
        Expr::Bin(expr) => expr_needs_arguments(&expr.left) || expr_needs_arguments(&expr.right),
        Expr::Assign(expr) => {
            assign_target_needs_arguments(&expr.left) || expr_needs_arguments(&expr.right)
        }
        Expr::Member(expr) => {
            expr_needs_arguments(&expr.obj)
                || matches!(&expr.prop, MemberProp::Computed(prop) if expr_needs_arguments(&prop.expr))
        }
        Expr::Cond(expr) => {
            expr_needs_arguments(&expr.test)
                || expr_needs_arguments(&expr.cons)
                || expr_needs_arguments(&expr.alt)
        }
        Expr::Call(expr) => {
            matches!(&expr.callee, Callee::Expr(callee) if expr_needs_arguments(callee))
                || expr.args.iter().any(|arg| expr_needs_arguments(&arg.expr))
        }
        Expr::New(expr) => {
            expr_needs_arguments(&expr.callee)
                || expr
                    .args
                    .as_ref()
                    .is_some_and(|args| args.iter().any(|arg| expr_needs_arguments(&arg.expr)))
        }
        Expr::Seq(expr) => expr.exprs.iter().any(|expr| expr_needs_arguments(expr)),
        Expr::Paren(expr) => expr_needs_arguments(&expr.expr),
        Expr::Tpl(expr) => expr.exprs.iter().any(|expr| expr_needs_arguments(expr)),
        Expr::TaggedTpl(expr) => expr_needs_arguments(&expr.tag),
        Expr::Arrow(_) | Expr::Fn(_) | Expr::Class(_) => false,
        Expr::Yield(expr) => expr
            .arg
            .as_ref()
            .is_some_and(|expr| expr_needs_arguments(expr)),
        Expr::Await(expr) => expr_needs_arguments(&expr.arg),
        Expr::OptChain(_) => true,
        Expr::Invalid(_) => false,
        _ => true,
    }
}

fn prop_name_needs_arguments(name: &swc_ecma_ast::PropName) -> bool {
    matches!(name, swc_ecma_ast::PropName::Computed(computed) if expr_needs_arguments(&computed.expr))
}

fn assign_target_needs_arguments(target: &swc_ecma_ast::AssignTarget) -> bool {
    match target {
        swc_ecma_ast::AssignTarget::Simple(target) => match target {
            swc_ecma_ast::SimpleAssignTarget::Ident(ident) => {
                ident.sym == *"arguments" || ident.sym == *"eval"
            }
            swc_ecma_ast::SimpleAssignTarget::Member(member) => {
                expr_needs_arguments(&member.obj)
                    || matches!(&member.prop, MemberProp::Computed(prop) if expr_needs_arguments(&prop.expr))
            }
            swc_ecma_ast::SimpleAssignTarget::Paren(paren) => expr_needs_arguments(&paren.expr),
            swc_ecma_ast::SimpleAssignTarget::OptChain(_) => true,
            _ => false,
        },
        swc_ecma_ast::AssignTarget::Pat(_) => true,
    }
}

impl CustomGcRefUntyped for RawJSFunction {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        self.scope.gc_untyped_ref()
    }
}

impl Constructor for JSFunction {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        yavashark_env::profiler::profile_call(
            realm,
            || self.name(),
            |realm| {
                let this = self.new_instance(realm)?;

                if let Value::Object(obj) = self.raw.call(realm, args, this.copy())? {
                    return Ok(obj);
                }

                this.to_object()
            },
        )
    }

    // fn construct_proto(&self) -> Res<ObjectProperty> {
    //     let inner = self.inner.try_borrow()?;
    //
    //     Ok(inner.prototype.clone())
    // }
}

impl ConstructorFn for RawJSFunction {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>> {
        self.scope.gc_untyped_ref()
    }

    fn construct(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res {
        self.call(realm, args, this)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Interpreter;
    use swc_common::DUMMY_SP;
    use swc_ecma_ast::{BlockStmt, Param, Pat};
    use swc_ecma_parser::EsSyntax;
    use yavashark_env::scope::Scope;
    use yavashark_env::test_eval;

    #[test]
    fn test_function() {
        test_eval!(
            r"
            function add(a, b){
                return a + b;
            }
            add(1, 2)
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn test_function_with_scope() {
        test_eval!(
            r"
            let a = 1;
            function add(b){
                return a + b;
            }
            add(2)
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn test_function_with_scope_and_block() {
        test_eval!(
            r"
            let a = 1;
            function add(b){
                {
                    let a = 2;
                }
                return a + b;
            }
            add(2)
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn attach_arbitrary() {
        test_eval!(
            "
                function foo() {}

                console.log(foo)

                console.log(foo.prototype)

                foo.prototype.a = 1

                console.log(foo.prototype.a)


                foo.prototype.a
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn arguments() {
        test_eval!(
            r"
                function foo() {
                    console.log(arguments)
                    for (let arg of arguments) {
                        mock.values(arg)
                    }
                } 
                
                
                foo(1,2,3,4,5)
            ",
            0,
            vec![
                vec![Value::Number(1.0)],
                vec![Value::Number(2.0)],
                vec![Value::Number(3.0)],
                vec![Value::Number(4.0)],
                vec![Value::Number(5.0)]
            ],
            Value::Undefined
        );
    }
}
