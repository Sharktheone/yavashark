use bumpalo::collections::Vec;
use yavashark_string::YSString;

pub enum Statement<'alloc> {
    Block(Block<'alloc>),
    Expression(Expression<'alloc>),

    Break(Break),
    Continue(Continue),
    Return(Return<'alloc>),
    Throw(Throw<'alloc>),

    Try(Try<'alloc>),

    If(If<'alloc>),
    Switch(Switch<'alloc>),


    Labelled(Labelled<'alloc>),
    While(While<'alloc>),
    DoWhile(DoWhile<'alloc>),
    For(For<'alloc>),
    ForIn(ForIn<'alloc>),
    ForOf(ForOf<'alloc>),

    With(With<'alloc>),

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