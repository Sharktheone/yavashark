use crate::tokens::group::Group;
use crate::tokens::ident::Ident;
use crate::tokens::keyword::Keyword;
use crate::tokens::lit::Lit;
use crate::tokens::punct::Punct;

mod group;
mod ident;
mod keyword;
mod lit;
mod punct;

enum Token {
    Ident(Ident),
    Lit(Lit),
    Punct(Punct),
    Keyword(Keyword),
    Group(Group),
}
