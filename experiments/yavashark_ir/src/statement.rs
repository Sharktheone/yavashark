use crate::expr::Expr;
use crate::atom::Atom;
use crate::{Ident, MethodKind, ScopeId, Span};

#[derive(Debug, Clone, Copy)]
pub enum Stmt<'a> {
    /// Expressions
    Expr(Expr<'a>),

    /// Block
    Block(&'a Block<'a>),

    Var(&'a VarDecl<'a>),
    Function(FuncId),
    Class(&'a Class<'a>),

    /// Control Flow

    Break(Atom),
    Continue(Atom),
    Return(Option<&'a Expr<'a>>),
    Throw(&'a Expr<'a>),

    /// Try

    Try(&'a Try<'a>),

    /// Branching

    If(&'a If<'a>),
    Switch(&'a Switch<'a>),

    /// Loops

    Labelled(&'a Labelled<'a>),
    While(&'a While<'a>),
    DoWhile(&'a While<'a>),
    For(&'a For<'a>),
    ForIn(&'a ForEach<'a>),
    ForOf(&'a ForEach<'a>),

    With(&'a With<'a>),

    /// Modules

    Import(&'a ImportDecl<'a>),
    Export(&'a Export<'a>),

    /// Debugger
    Debugger(Debugger),
    Empty,
}

const _: () = {
    assert!(size_of::<Stmt<'static>>() == 16, "Stmt must be 16 bytes");
    assert!(size_of::<Stmt<'static>>() == size_of::<Option<Stmt<'static>>>());

};

#[derive(Debug, Clone, Copy)]
pub struct Block<'a> {
    pub stmts: &'a [Stmt<'a>],
    pub scope: ScopeId,
}


#[derive(Debug, Clone, Copy)]
pub struct VarDecl<'a> {
    pub kind: VarKind,
    pub decls: &'a [Declarator<'a>],
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VarKind {
    Var,
    Let,
    Const,
    Using,
    AwaitUsing,
}


#[derive(Debug, Clone, Copy)]
pub struct Declarator<'a> {
    pub target: Pat<'a>,
    pub init: Option<Expr<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Class<'a> {
    pub name: Option<Ident>,
    pub super_class: Option<Expr<'a>>,
    pub constructor: Option<FuncId>,
    pub members: &'a [ClassMember<'a>],
    /// Holds the inner class name binding.
    pub scope: ScopeId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum ClassMember<'a> {
    Method {
        key: PropKey<'a>,
        func: FuncId,
        kind: MethodKind,
        is_static: bool,
    },
    Field {
        key: PropKey<'a>,
        value: Option<FuncId>,
        is_static: bool,
    },
    StaticBlock(FuncId),
}

#[derive(Debug, Clone, Copy)]
pub struct Try<'a> {
    pub block: Block<'a>,
    pub catch: Option<Catch<'a>>,
    pub finally: Option<Block<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Catch<'a> {
    pub param: Option<Pat<'a>>,
    pub block: Block<'a>,
    pub scope: ScopeId,
}

#[derive(Debug, Clone, Copy)]
pub struct If<'a> {
    pub test: Expr<'a>,
    pub consequent: Stmt<'a>,
    pub alternate: Option<Stmt<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Switch<'a> {
    pub discriminant: Expr<'a>,
    pub cases: &'a [Case<'a>],
    pub scope: ScopeId,
}

#[derive(Debug, Clone, Copy)]
pub struct Case<'a> {
    pub test: Option<Expr<'a>>,
    pub consequent: &'a [Stmt<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct Labelled<'a> {
    pub label: Atom,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct While<'a> {
    pub test: Expr<'a>,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct For<'a> {
    pub init: Option<ForInit<'a>>,
    pub test: Option<Expr<'a>>,
    pub update: Option<Expr<'a>>,
    pub body: Stmt<'a>,
    pub scope: ScopeId,
}

#[derive(Debug, Clone, Copy)]
pub enum ForInit<'a> {
    VarDecl(&'a VarDecl<'a>),
    Expression(Expr<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ForEach<'a> {
    pub left: ForHead<'a>,
    pub right: Expr<'a>,
    pub body: Stmt<'a>,
    pub is_await: bool,
    pub scope: ScopeId,
}

#[derive(Debug, Clone, Copy)]
pub enum ForHead<'a> {
    Var(&'a VarDecl<'a>),
    Pat(Pat<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct With<'a> {
    pub object: Expr<'a>,
    pub body: Stmt<'a>,
    pub scope: ScopeId,
}

#[derive(Debug, Clone, Copy)]
pub struct Debugger;



#[derive(Debug, Clone, Copy)]
pub struct FuncId(u32);



#[derive(Debug, Clone, Copy)]
pub struct ImportDecl<'a> {
    pub specifiers: &'a [ImportSpecifier],
    pub source: Atom,
    pub attributes: &'a [(Atom, Atom)],
}

#[derive(Debug, Clone, Copy)]
pub struct ImportSpecifier {
    /// `None` for `* as local`, default imports are `Some("default")`.
    pub imported: Option<Atom>,
    pub local: Ident,
}

#[derive(Debug, Clone, Copy)]
pub struct Export<'a> {
    pub a: &'a (),
}

#[derive(Debug, Clone, Copy)]
pub enum PropKey<'a> {
    /// Identifier and string keys, bigint keys are converted to their decimal string.
    Name(Atom),
    Number(f64), //TODO: we might be able to also put this into `Name`
    Computed(Expr<'a>),
    Private(Ident),
}


#[derive(Debug, Clone, Copy)]
pub enum Pat<'a> {
    Ident(Ident),
    /// Member expressions and other targets only valid in assignments.
    Expr(Expr<'a>),
    Array(&'a ArrayPat<'a>),
    Object(&'a ObjectPat<'a>),
    Default(&'a DefaultPat<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ArrayPat<'a> {
    pub elems: &'a [Option<Pat<'a>>],
    pub rest: Option<Pat<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectPat<'a> {
    pub props: &'a [PatProp<'a>],
    pub rest: Option<Pat<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct PatProp<'a> {
    pub key: PropKey<'a>,
    pub value: Pat<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultPat<'a> {
    pub target: Pat<'a>,
    pub value: Expr<'a>,
}
