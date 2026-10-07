mod class;
mod decl;
mod expr;
mod function;
mod ident;
mod module;
mod pattern;
mod scope;
mod statement;
mod utils;

use crate::ValidationError;
use bumpalo::Bump;
use hashbrown::HashMap;
use smallvec::SmallVec;
use swc_common::Span;

type Map<'a, 'b, V> = HashMap<&'a str, V, hashbrown::DefaultHashBuilder, &'b Bump>;
const LEX: u8 = 1;
const VAR: u8 = 2;
const PARAM: u8 = 4;
const FUNCTION: u8 = 8;
const CATCH: u8 = 16;

#[derive(Clone, Copy, PartialEq)]
enum ScopeKind {
    Script,
    Module,
    Function,
    Block,
    Catch,
    Switch,
}

struct Scope<'a, 'b> {
    kind: ScopeKind,
    names: Bindings<'a, 'b>,
}
// Most lexical scopes contain at most a handful of names. Keep those inline;
// grow into the arena only when hashing is worth its setup and storage cost.

struct Bindings<'a, 'b> {
    inline: SmallVec<[(&'a str, u8); 4]>,
    table: Map<'a, 'b, u8>,
    capacity_hint: usize,
}

#[derive(Clone, Copy, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Independent ECMAScript grammar parameters, copied at function boundaries"
)]
struct Context {
    strict: bool,
    module: bool,
    function: bool,
    asynchronous: bool,
    generator: bool,
    parameters: bool,
    unique_parameters: bool,
    super_prop: bool,
    super_call: bool,
    new_target: bool,
    no_arguments: bool,
    static_block: bool,
    loops: u32,
    switches: u32,
    label_base: usize,
}

pub struct Checker<'a, 'b> {
    arena: &'b Bump,
    scopes: SmallVec<[Scope<'a, 'b>; 8]>,
    labels: SmallVec<[(&'a str, bool); 8]>,
    private: SmallVec<[Map<'a, 'b, u8>; 4]>,
    ctx: Context,
    depth: usize,
}

impl<'a, 'b> Checker<'a, 'b> {
    pub fn new(arena: &'b Bump, strict: bool, module: bool) -> Self {
        Self {
            arena,
            scopes: SmallVec::new(),
            labels: SmallVec::new(),
            private: SmallVec::new(),
            depth: 0,
            ctx: Context {
                strict,
                module,
                ..Context::default()
            },
        }
    }

    pub(crate) const fn allow_new_target(&mut self) {
        self.ctx.new_target = true;
    }

    fn push_scope(&mut self, kind: ScopeKind) {
        self.scopes.push(Scope {
            kind,
            names: Bindings::new(self.arena, 8),
        });
    }

    fn push_scope_with_capacity(&mut self, kind: ScopeKind, capacity_hint: usize) {
        self.scopes.push(Scope {
            kind,
            names: Bindings::new(self.arena, capacity_hint),
        });
    }

    const fn binding_error(
        message: &'static str,
        name: &'a str,
        span: Span,
    ) -> ValidationError<'a> {
        ValidationError {
            message,
            name: Some(name),
            span,
        }
    }
}
