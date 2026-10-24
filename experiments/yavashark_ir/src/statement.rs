use crate::expr::Expr;
use crate::atom::Atom;

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
    DoWhile(&'a DoWhile<'a>),
    For(&'a For<'a>),
    ForIn(&'a ForIn<'a>),
    ForOf(&'a ForOf<'a>),

    With(&'a With<'a>),

    /// Modules

    Import(&'a Import<'a>),
    Export(&'a Export<'a>),

    /// Debugger
    Debugger(Debugger),
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub struct Block<'a> {
    pub stmts: &'a [Stmt<'a>],
}


#[derive(Debug, Clone, Copy)]
pub struct Try<'a> {
    block: Block<'a>,
    catch: Option<Catch<'a>>,
    finally: Option<Block<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Catch<'a> {
    param: Option<Pat<'a>>,
    block: Block<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct If<'a> {
    test: Expr<'a>,
    consequent: Stmt<'a>,
    alternate: Option<Stmt<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Switch<'a> {
    discriminant: Expr<'a>,
    cases: &'a [SwitchCase<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct SwitchCase<'a> {
    test: Option<Expr<'a>>,
    consequent: &'a [Stmt<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct Labelled<'a> {
    label: Atom,
    body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct While<'a> {
    test: Expr<'a>,
    body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct DoWhile<'a> {
    body: Stmt<'a>,
    test: Expr<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct For<'a> {
    init: Option<ForInit<'a>>,
    test: Option<Expr<'a>>,
    update: Option<Expr<'a>>,
    body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForInit<'a> {
    VarDecl(VarDecl<'a>),
    Expression(Expr<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ForIn<'a> {
    left: ForInLeft<'a>,
    right: Expr<'a>,
    body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForInLeft<'a> {
    VarDecl(VarDecl<'a>),
    Pattern(Pat<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ForOf<'a> {
    left: ForOfLeft<'a>,
    right: Expr<'a>,
    body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForOfLeft<'a> {
    VarDecl(VarDecl<'a>),
    Pattern(Pat<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct With<'a> {
    object: Expr<'a>,
    body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct Debugger;


#[derive(Debug, Clone, Copy)]
pub enum Pat<'a> {
    A(&'a ()),
}

struct VarDecl<'a> {
    a: &'a (),
}


#[derive(Debug, Clone, Copy)]
struct FuncId(u32);


struct Class<'a> {
    a: &'a (),
}

#[derive(Debug, Clone, Copy)]
struct Import<'a> {
    a: &'a (),
}

#[derive(Debug, Clone, Copy)]
struct Export<'a> {
    a: &'a (),
}