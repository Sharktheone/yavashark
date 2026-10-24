use crate::expr::Expression;
use yavashark_string::YSString;
use crate::atom::Atom;

pub enum Statement<'alloc> {
    /// Expressions
    Expression(&'alloc Expression<'alloc>),

    /// Block
    Block(&'alloc Block<'alloc>),

    Var(&'alloc VarDecl<'alloc>),
    Function(FuncId),
    Class(&'alloc Class<'alloc>),

    /// Control Flow

    Break(Atom),
    Continue(Atom),
    Return(&'alloc Expression<'alloc>),
    Throw(&'alloc Expression<'alloc>),

    /// Try

    Try(&'alloc Try<'alloc>),

    /// Branching

    If(&'alloc If<'alloc>),
    Switch(&'alloc Switch<'alloc>),

    /// Loops

    Labelled(&'alloc Labelled<'alloc>),
    While(&'alloc While<'alloc>),
    DoWhile(&'alloc DoWhile<'alloc>),
    For(&'alloc For<'alloc>),
    ForIn(&'alloc ForIn<'alloc>),
    ForOf(&'alloc ForOf<'alloc>),

    With(&'alloc With<'alloc>),

    /// Modules

    Import(&'alloc Import<'alloc>),
    Export(&'alloc Export<'alloc>),

    /// Debugger
    Debugger(Debugger),
    Empty,
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