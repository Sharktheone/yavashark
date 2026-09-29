use super::Checker;
use crate::{Result, ValidationError};
use swc_ecma_ast::Ident;

impl<'a> Checker<'a, '_> {
    pub(super) fn validate_ident(&self, id: &'a Ident) -> Result<'a> {
        let name = id.sym.as_str();
        Self::identifier_name(name).map_err(|m| ValidationError::new(m, id.span))?;

        if Self::reserved(name)
            || (name == "await"
                && (self.ctx.asynchronous || self.ctx.module || self.ctx.static_block))
            || (name == "yield" && (self.ctx.generator || self.ctx.strict))
            || (self.ctx.strict && Self::strict_reserved(name))
        {
            return Err(Self::binding_error("Reserved identifier", name, id.span));
        }

        if self.ctx.no_arguments && name == "arguments" {
            return Err(Self::binding_error(
                "Arguments is forbidden in class initializers",
                name,
                id.span,
            ));
        }

        Ok(())
    }

    pub(super) fn validate_binding(&self, id: &'a Ident) -> Result<'a> {
        self.validate_ident(id)?;

        if self.ctx.strict && matches!(id.sym.as_str(), "eval" | "arguments") {
            return Err(Self::binding_error(
                "Invalid strict binding or assignment",
                &id.sym,
                id.span,
            ));
        }

        Ok(())
    }
}
