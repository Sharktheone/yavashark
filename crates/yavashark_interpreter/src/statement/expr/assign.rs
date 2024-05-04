use swc_ecma_ast::{AssignExpr, AssignTarget, Ident, MemberExpr, MemberProp, SimpleAssignTarget};

use crate::Error;

use crate::context::Context;
use crate::scope::Scope;
use crate::Value;
use crate::{Res, RuntimeResult};

impl Context {
    pub fn run_assign(&mut self, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = self.run_expr(&stmt.right, stmt.span, scope)?;

        Ok(self
            .assign_target(&stmt.left, value, scope)
            .map(|_| Value::Undefined)?)
    }
    
    pub fn with_target(&mut self, target: &AssignTarget, f: &impl Fn(&mut Value), scope: &mut Scope) -> Res {
        match target { 
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();
                    scope.with_mut(&name, f)?;
                }
                SimpleAssignTarget::Member(m) => {
                    self.with_member(m, f, scope)?;
                },
                
                _ => todo!("assign targets"),
            }
            
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
        }
        
        
        Ok(())
        
    }

    pub fn assign_target(&mut self, target: &AssignTarget, value: Value, scope: &mut Scope) -> Res {
        match target {
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();
                    scope.update_or_define(name, value)
                }
                SimpleAssignTarget::Member(m) => self.assign_member(m, value, scope),
                _ => todo!("assign targets"),
            },
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
        }
    }

    
    
    
    pub fn assign_member(&mut self, m: &MemberExpr, value: Value, scope: &mut Scope) -> Res {
        let obj = self.run_expr(&m.obj, m.span, scope)?;
        if let Value::Object(obj) = obj {
            let mut obj = obj.get_mut()?;

            let name = match &m.prop {
                MemberProp::Ident(i) => i.sym.to_string(),
                MemberProp::PrivateName(p) => p.id.sym.to_string(),
                MemberProp::Computed(c) => {
                    let name = self.run_expr(&c.expr, c.span, scope)?;
                    value.to_string() //TODO: numbers will have problems
                }
            }
            .into();

            obj.update_or_define_property(name, value);
            Ok(())
        } else {
            Err(Error::ty("Invalid let-hand side in assignment".to_string()))
        }
    }
    
    pub fn with_member(&mut self, m: &MemberExpr, f: &impl Fn(&mut Value), scope: &mut Scope) -> Res {
        let obj = self.run_expr(&m.obj, m.span, scope)?;
        if let Value::Object(obj) = obj {
            let mut obj = obj.get_mut()?;
            
            let name = match &m.prop {
                MemberProp::Ident(i) => i.sym.to_string(),
                MemberProp::PrivateName(p) => p.id.sym.to_string(),
                MemberProp::Computed(c) => {
                    let name = self.run_expr(&c.expr, c.span, scope)?;
                    name.to_string() //TODO: numbers will have problems
                }
            };
            
            scope.with_mut(&name, f)?;
        } else {
            return Err(Error::ty("Invalid let-hand side in assignment".to_string()));
        }
        
        Ok(())
    }
}
