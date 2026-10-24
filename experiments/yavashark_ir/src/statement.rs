use crate::expr::Expression;
use yavashark_string::YSString;
use crate::atom::Atom;

pub enum Statement<'a> {
    /// Expressions
    Expression(Expression<'a>),

    /// Block
    Block(&'a Block<'a>),

    Var(&'a VarDecl<'a>),
    Function(FuncId),
    Class(&'a Class<'a>),

    /// Control Flow

    Break(Atom),
    Continue(Atom),
    Return(&'a Expression<'a>),
    Throw(&'a Expression<'a>),

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
    dbg!(size_of::<Statement>());
}

pub struct Block<'alloc> {
    statements: &'alloc [Statement<'alloc>],
}

pub struct Return<'alloc> {
    value: Option<Expression<'alloc>>,
}

pub struct Throw<'alloc> {
    value: Expression<'alloc>,
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
    test: Expression<'alloc>,
    consequent: Statement<'alloc>,
    alternate: Option<Statement<'alloc>>,
}

pub struct Switch<'alloc> {
    discriminant: Expression<'alloc>,
    cases: &'alloc [SwitchCase<'alloc>],
}

pub struct SwitchCase<'alloc> {
    test: Option<Expression<'alloc>>,
    consequent: &'alloc [Statement<'alloc>],
}

pub struct Labelled<'alloc> {
    label: YSString,
    body: Statement<'alloc>,
}

pub struct While<'alloc> {
    test: Expression<'alloc>,
    body: Statement<'alloc>,
}

pub struct DoWhile<'alloc> {
    body: Statement<'alloc>,
    test: Expression<'alloc>,
}

pub struct For<'alloc> {
    init: Option<ForInit<'alloc>>,
    test: Option<Expression<'alloc>>,
    update: Option<Expression<'alloc>>,
    body: Statement<'alloc>,
}

pub enum ForInit<'alloc> {
    VarDecl(VarDecl<'alloc>),
    Expression(Expression<'alloc>),
}

pub struct ForIn<'alloc> {
    left: ForInLeft<'alloc>,
    right: Expression<'alloc>,
    body: Statement<'alloc>,
}

pub enum ForInLeft<'alloc> {
    VarDecl(VarDecl<'alloc>),
    Pattern(Pattern<'alloc>),
}

pub struct ForOf<'alloc> {
    left: ForOfLeft<'alloc>,
    right: Expression<'alloc>,
    body: Statement<'alloc>,
}

pub enum ForOfLeft<'alloc> {
    VarDecl(VarDecl<'alloc>),
    Pattern(Pattern<'alloc>),
}

pub struct With<'alloc> {
    object: Expression<'alloc>,
    body: Statement<'alloc>,
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