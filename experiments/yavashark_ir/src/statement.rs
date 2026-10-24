use crate::expr::Expr;
use yavashark_string::YSString;
use crate::atom::Atom;

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

#[test]
fn foo() {
    dbg!(size_of::<Stmt>());
}

pub struct Block<'alloc> {
    statements: &'alloc [Stmt<'alloc>],
}

pub struct Return<'alloc> {
    value: Option<Expr<'alloc>>,
}

pub struct Throw<'alloc> {
    value: Expr<'alloc>,
}

pub struct Try<'alloc> {
    block: Block<'alloc>,
    catch: Option<Catch<'alloc>>,
    finally: Option<Block<'alloc>>,
}

pub struct Catch<'alloc> {
    param: Option<Pattern<'alloc>>,
    block: Block<'alloc>,
}

pub struct If<'alloc> {
    test: Expr<'alloc>,
    consequent: Stmt<'alloc>,
    alternate: Option<Stmt<'alloc>>,
}

pub struct Switch<'alloc> {
    discriminant: Expr<'alloc>,
    cases: &'alloc [SwitchCase<'alloc>],
}

pub struct SwitchCase<'alloc> {
    test: Option<Expr<'alloc>>,
    consequent: &'alloc [Stmt<'alloc>],
}

pub struct Labelled<'alloc> {
    label: YSString,
    body: Stmt<'alloc>,
}

pub struct While<'alloc> {
    test: Expr<'alloc>,
    body: Stmt<'alloc>,
}

pub struct DoWhile<'alloc> {
    body: Stmt<'alloc>,
    test: Expr<'alloc>,
}

pub struct For<'alloc> {
    init: Option<ForInit<'alloc>>,
    test: Option<Expr<'alloc>>,
    update: Option<Expr<'alloc>>,
    body: Stmt<'alloc>,
}

pub enum ForInit<'alloc> {
    VarDecl(VarDecl<'alloc>),
    Expression(Expr<'alloc>),
}

pub struct ForIn<'alloc> {
    left: ForInLeft<'alloc>,
    right: Expr<'alloc>,
    body: Stmt<'alloc>,
}

pub enum ForInLeft<'alloc> {
    VarDecl(VarDecl<'alloc>),
    Pattern(Pattern<'alloc>),
}

pub struct ForOf<'alloc> {
    left: ForOfLeft<'alloc>,
    right: Expr<'alloc>,
    body: Stmt<'alloc>,
}

pub enum ForOfLeft<'alloc> {
    VarDecl(VarDecl<'alloc>),
    Pattern(Pattern<'alloc>),
}

pub struct With<'alloc> {
    object: Expr<'alloc>,
    body: Stmt<'alloc>,
}

pub struct Debugger;
pub enum Pattern<'alloc> {
    A(&'alloc ()),
}

struct VarDecl<'alloc> {
    a: &'alloc (),
}


struct FuncId(u32);


struct Class<'alloc> {
    a: &'alloc (),
}

struct Import<'alloc> {
    a: &'alloc (),
}

struct Export<'alloc> {
    a: &'alloc (),
}