use bumpalo::collections::Vec;
use bumpalo::boxed::Box;
use yavashark_string::YSString;

pub enum Statement<'alloc> {
    Block(Box<'alloc, Block<'alloc>>),
    Expression(Box<'alloc, Expression<'alloc>>),

    Break(Box<'alloc, Break>),
    Continue(Box<'alloc, Continue>),
    Return(Box<'alloc, Return<'alloc>>),
    Throw(Box<'alloc, Throw<'alloc>>),

    Try(Box<'alloc, Try<'alloc>>),

    If(Box<'alloc, If<'alloc>>),
    Switch(Box<'alloc, Switch<'alloc>>),


    Labelled(Box<'alloc, Labelled<'alloc>>),
    While(Box<'alloc, While<'alloc>>),
    DoWhile(Box<'alloc, DoWhile<'alloc>>),
    For(Box<'alloc, For<'alloc>>),
    ForIn(Box<'alloc, ForIn<'alloc>>),
    ForOf(Box<'alloc, ForOf<'alloc>>),

    With(Box<'alloc, With<'alloc>>),

    Debugger(Debugger),
}


pub enum Expression<'alloc> {
    A(&'alloc ())

}



pub struct Block<'alloc> {
    statements: Vec<'alloc, Statement<'alloc>>,
}

pub struct Break {
    label: Option<YSString>,
}

pub struct Continue {
    label: Option<YSString>,
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
    cases: Vec<'alloc, SwitchCase<'alloc>>,
}

pub struct SwitchCase<'alloc> {
    test: Option<Expression<'alloc>>,
    consequent: Vec<'alloc, Statement<'alloc>>,
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
    A(&'alloc ())
}