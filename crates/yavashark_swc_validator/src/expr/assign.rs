use crate::Validator;
use swc_ecma_ast::{AssignExpr, AssignTarget, AssignTargetPat, Expr, SimpleAssignTarget};

impl<'a> Validator<'a> {
    pub fn validate_assign_expr(&mut self, assign: &'a AssignExpr) -> Result<(), String> {
        match &assign.left {
            AssignTarget::Simple(simple) => match simple {
                SimpleAssignTarget::Ident(ident) => {
                    self.validate_ident(&ident.id)?;

                    if self.in_strict_mode() {
                        let name = ident.id.sym.as_ref();
                        if matches!(name, "eval" | "arguments") {
                            return Err(format!(
                                "Cannot assign to '{name}' in strict mode"
                            ));
                        }
                    }
                }
                SimpleAssignTarget::Member(member) => self.validate_member_expr(member)?,
                SimpleAssignTarget::SuperProp(prop) => {
                    self.validate_super_prop_expr(prop)?;
                }
                SimpleAssignTarget::Paren(paren) => {
                    self.ensure_valid_assignment_target_expr(&paren.expr)?;
                }
                SimpleAssignTarget::OptChain(optchain) => {
                    self.validate_opt_chain_expr(optchain)?;
                    return Err("Invalid assignment target: optional chaining".to_string());
                }
                _ => return Err("Unsupported simple assignment target".to_string()),
            },
            AssignTarget::Pat(pat) => match pat {
                AssignTargetPat::Object(obj) => self.validate_object_pat(obj, &mut None)?,
                AssignTargetPat::Array(arr) => self.validate_array_pat(arr, &mut None)?,
                AssignTargetPat::Invalid(_) => {
                    return Err("Invalid assignment target pattern".to_string())?;
                }
            },
        }

        self.validate_expr(&assign.right)
    }

    pub(crate) fn ensure_valid_assignment_target_expr(
        &mut self,
        expr: &'a Expr,
    ) -> Result<(), String> {
        match expr {
            Expr::Ident(ident) => self.validate_ident(ident),
            Expr::Member(member) => self.validate_member_expr(member),
            Expr::SuperProp(super_prop) => self.validate_super_prop_expr(super_prop),
            Expr::Paren(paren) => self.ensure_valid_assignment_target_expr(&paren.expr),
            Expr::Seq(seq) => {
                let Some((last, rest)) = seq.exprs.split_last() else {
                    return Err("Invalid assignment target: empty sequence".to_string());
                };

                for expr in rest {
                    self.validate_expr(expr)?;
                }

                self.ensure_valid_assignment_target_expr(last)
            }
            Expr::OptChain(_) => Err("Invalid assignment target: optional chaining".to_string()),
            Expr::Await(_) => Err("Invalid assignment target: await expression".to_string()),
            Expr::Assign(_) => Err("Invalid assignment target: assignment expression".to_string()),
            Expr::Update(_) => Err("Invalid assignment target: update expression".to_string()),
            _ => Err("Invalid assignment target".to_string()),
        }
    }
}
