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

const _: () = {
    assert!(size_of::<Stmt<'static>>() == 16, "Stmt must be 16 bytes");
    assert!(size_of::<Stmt<'static>>() == size_of::<Option<Stmt<'static>>>());

};

#[derive(Debug, Clone, Copy)]
pub struct Block<'a> {
    pub stmts: &'a [Stmt<'a>],
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
pub struct Try<'a> {
    pub block: Block<'a>,
    pub catch: Option<Catch<'a>>,
    pub finally: Option<Block<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Catch<'a> {
    pub param: Option<Pat<'a>>,
    pub block: Block<'a>,
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
    pub cases: &'a [SwitchCase<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct SwitchCase<'a> {
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
pub struct DoWhile<'a> {
    pub body: Stmt<'a>,
    pub test: Expr<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct For<'a> {
    pub init: Option<ForInit<'a>>,
    pub test: Option<Expr<'a>>,
    pub update: Option<Expr<'a>>,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForInit<'a> {
    VarDecl(VarDecl<'a>),
    Expression(Expr<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ForIn<'a> {
    pub left: ForInLeft<'a>,
    pub right: Expr<'a>,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForInLeft<'a> {
    VarDecl(VarDecl<'a>),
    Pattern(Pat<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct ForOf<'a> {
    pub left: ForOfLeft<'a>,
    pub right: Expr<'a>,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForOfLeft<'a> {
    VarDecl(VarDecl<'a>),
    Pattern(Pat<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct With<'a> {
    pub object: Expr<'a>,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct Debugger;



#[derive(Debug, Clone, Copy)]
pub struct FuncId(u32);


struct Class<'a> {
    a: &'a (),
}

#[derive(Debug, Clone, Copy)]
struct Import<'a> {
    pub a: &'a (),
}

#[derive(Debug, Clone, Copy)]
struct Export<'a> {
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
