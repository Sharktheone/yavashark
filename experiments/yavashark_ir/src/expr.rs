use regress::Regex;
use crate::atom::Atom;

#[derive(Debug, Clone, Copy)]
pub enum Expr<'a> {
    Bool(bool),
    Null,
    Undefined,
    Float(f64),
    Int32(i32),
    String(Atom),
    BigInt(Atom),
    RegExp(&'a Regex),
    Template(&'a Template<'a>),
    Ident(Atom),
    MetaProp(&'a MetaProp),
    Super(&'a SuperProp),
    Array(&'a Array<'a>),
    ArrowFunction(&'a Arrow<'a>),
    Assignment(&'a Assignment<'a>),
    Await(&'a Await<'a>),
    Binary(&'a Binary<'a>),
    Call(&'a Call<'a>),
    Chain(&'a Chain<'a>),
    Class(&'a Class<'a>),
    Conditional(&'a Conditional<'a>),
    Function(&'a Function<'a>),
    Import(&'a Import<'a>),
    Logical(&'a Logical<'a>),
    New(&'a New<'a>),
    Object(&'a Object<'a>),
    Sequence(&'a Sequence<'a>),
    TaggedTemplate(&'a TaggedTemplate<'a>),
    This,
    Unary(&'a Unary<'a>),
    Update(&'a Update<'a>),
    Yield(&'a Yield<'a>),
    PrivateIn(&'a PrivateIn<'a>),
    Member(&'a Member<'a>),
    ComputedMember(&'a ComputedMember<'a>),
    PrivateField(&'a PrivateField<'a>),
}

const _: () = {
    assert!(size_of::<Expr<'static>>() == 16, "Expr must be 16 bytes");
    assert!(size_of::<Expr<'static>>() == size_of::<Option<Expr<'static>>>());
};