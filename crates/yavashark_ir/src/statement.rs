pub enum Statement<'alloc> {
    Block(Block<'alloc>),
    Expression(Expression<'alloc>),

    Break(Break<'alloc>),
    Continue(Continue<'alloc>),
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

    Debugger(Debugger<'alloc>),
}
