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

pub struct Block<'a> {
    statements: &'a [Stmt<'a>],
}


pub struct Try<'a> {
    block: Block<'a>,
    catch: Option<Catch<'a>>,
    finally: Option<Block<'a>>,
}

pub struct Catch<'a> {
    param: Option<Pattern<'a>>,
    block: Block<'a>,
}

pub struct If<'a> {
    test: Expr<'a>,
    consequent: Stmt<'a>,
    alternate: Option<Stmt<'a>>,
}

pub struct Switch<'a> {
    discriminant: Expr<'a>,
    cases: &'a [SwitchCase<'a>],
}

pub struct SwitchCase<'a> {
    test: Option<Expr<'a>>,
    consequent: &'a [Stmt<'a>],
}

pub struct Labelled<'a> {
    label: YSString,
    body: Stmt<'a>,
}

pub struct While<'a> {
    test: Expr<'a>,
    body: Stmt<'a>,
}

pub struct DoWhile<'a> {
    body: Stmt<'a>,
    test: Expr<'a>,
}

pub struct For<'a> {
    init: Option<ForInit<'a>>,
    test: Option<Expr<'a>>,
    update: Option<Expr<'a>>,
    body: Stmt<'a>,
}

pub enum ForInit<'a> {
    VarDecl(VarDecl<'a>),
    Expression(Expr<'a>),
}

pub struct ForIn<'a> {
    left: ForInLeft<'a>,
    right: Expr<'a>,
    body: Stmt<'a>,
}

pub enum ForInLeft<'a> {
    VarDecl(VarDecl<'a>),
    Pattern(Pattern<'a>),
}

pub struct ForOf<'a> {
    left: ForOfLeft<'a>,
    right: Expr<'a>,
    body: Stmt<'a>,
}

pub enum ForOfLeft<'a> {
    VarDecl(VarDecl<'a>),
    Pattern(Pattern<'a>),
}

pub struct With<'a> {
    object: Expr<'a>,
    body: Stmt<'a>,
}

pub struct Debugger;
pub enum Pattern<'a> {
    A(&'a ()),
}

struct VarDecl<'a> {
    a: &'a (),
}


struct FuncId(u32);


struct Class<'a> {
    a: &'a (),
}

struct Import<'a> {
    a: &'a (),
}

struct Export<'a> {
    a: &'a (),
}