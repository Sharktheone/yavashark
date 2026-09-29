use super::{Bindings, CATCH, Checker, FUNCTION, LEX, Map, PARAM, ScopeKind, VAR};
use crate::Result;
use crate::ValidationError;
use bumpalo::Bump;
use smallvec::SmallVec;
use swc_ecma_ast::Ident;

impl<'a, 'b> Bindings<'a, 'b> {
    pub(super) fn new(arena: &'b Bump, capacity_hint: usize) -> Self {
        Self {
            inline: SmallVec::new(),
            table: Map::new_in(arena),
            capacity_hint,
        }
    }

    fn get(&self, key: &str) -> Option<&u8> {
        if self.table.is_empty() {
            self.inline.iter().find(|(n, _)| *n == key).map(|(_, v)| v)
        } else {
            self.table.get(key)
        }
    }

    pub(super) fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    fn insert(&mut self, key: &'a str, value: u8) {
        if !self.table.is_empty() {
            self.table.insert(key, value);
            return;
        }

        if let Some((_, v)) = self.inline.iter_mut().find(|(n, _)| *n == key) {
            *v = value;
            return;
        }

        if self.inline.len() < 4 {
            self.inline.push((key, value));
            return;
        }

        self.table.reserve(self.capacity_hint.max(8));

        for (k, v) in self.inline.drain(..) {
            self.table.insert(k, v);
        }

        self.table.insert(key, value);
    }
}

impl<'a> Checker<'a, '_> {
    pub(super) fn declare_binding(&mut self, id: &'a Ident, kind: u8) -> Result<'a> {
        self.validate_binding(id)?;
        let name = id.sym.as_str();

        if kind == VAR {
            for scope in self.scopes.iter_mut().rev() {
                let old = scope.names.get(name).copied().unwrap_or(0);

                if old & LEX != 0 && old & CATCH == 0 {
                    return Err(ValidationError {
                        message: "Variable conflicts with lexical binding",
                        name: Some(name),
                        span: id.span,
                    });
                }
                scope.names.insert(name, old | VAR);

                if matches!(
                    scope.kind,
                    ScopeKind::Function | ScopeKind::Script | ScopeKind::Module
                ) {
                    break;
                }
            }
        } else {
            let current_scope = self.scopes.len() - 1;
            let scope = &mut self.scopes[current_scope];
            let old = scope.names.get(name).copied().unwrap_or(0);
            let sloppy_functions =
                !self.ctx.strict && kind == (LEX | FUNCTION) && old == (LEX | FUNCTION);

            if old != 0
                && !sloppy_functions
                && !(kind == PARAM && old == PARAM && !self.ctx.unique_parameters)
            {
                return Err(ValidationError {
                    message: "Duplicate binding",
                    name: Some(name),
                    span: id.span,
                });
            }
            scope.names.insert(name, old | kind);
        }

        Ok(())
    }
}
