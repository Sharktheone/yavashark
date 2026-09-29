//! Read-only ECMAScript early-error validation of SWC syntax trees.
//!
//! Names and diagnostics borrow the tree. A reusable bump arena owns only scratch
//! tables; no node, atom, or source string is cloned. SWC parse errors (including
//! recovered errors) must be handled by the caller before invoking this crate.
mod check;
mod regexp;

use bumpalo::Bump;
use std::{fmt, marker::PhantomData};
use swc_common::Span;
use swc_ecma_ast::{ModuleItem, Program, Stmt};

/// A diagnostic with no heap allocation, even on the rejection path.
#[derive(Clone, Copy, Debug)]
pub struct ValidationError<'a> {
    pub message: &'static str,
    pub name: Option<&'a str>,
    pub span: Span,
}

impl fmt::Display for ValidationError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)?;

        if let Some(name) = self.name {
            write!(f, ": {name}")?;
        }

        Ok(())
    }
}

impl std::error::Error for ValidationError<'_> {}
pub(crate) type Result<'a> = std::result::Result<(), ValidationError<'a>>;
impl ValidationError<'_> {
    pub(crate) const fn new(message: &'static str, span: Span) -> Self {
        Self {
            message,
            name: None,
            span,
        }
    }
}

/// Reuse an instance to amortize scratch allocation across independent trees.
/// An error never borrows the scratch arena, so later calls cannot invalidate it.
#[derive(Default)]
pub struct Validator<'a> {
    scratch: Bump,
    strict: bool,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> Validator<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn enable_script_strict_mode(&mut self) {
        self.strict = true;
    }

    // Avoid retaining an unusually large program's scratch high-water mark.
    fn reset_scratch(&mut self) {
        if self.scratch.allocated_bytes() > 4 * 1024 * 1024 {
            self.scratch = Bump::new();
        } else {
            self.scratch.reset();
        }
    }

    pub fn validate_statements(&mut self, ast: &'a [Stmt]) -> Result<'a> {
        self.reset_scratch();

        check::Checker::new(&self.scratch, self.strict, false).validate_script(ast)
    }

    pub fn validate_module_items(&mut self, ast: &'a [ModuleItem]) -> Result<'a> {
        self.reset_scratch();

        check::Checker::new(&self.scratch, true, true).validate_module(ast)
    }

    pub fn validate(&mut self, ast: &'a Program) -> Result<'a> {
        match ast {
            Program::Script(s) => self.validate_statements(&s.body),
            Program::Module(m) => self.validate_module_items(&m.body),
        }
    }

    #[must_use]
    pub fn is_valid(&mut self, ast: &'a Program) -> bool {
        self.validate(ast).is_ok()
    }
}
