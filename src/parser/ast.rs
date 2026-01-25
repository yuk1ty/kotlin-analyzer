use chumsky::span::SimpleSpan;

#[derive(Debug, Clone)]
pub struct File {
    pub declarations: Vec<Decl>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Class(ClassDecl),
    Fun(FunDecl),
    Property(PropertyDecl),
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub span: SimpleSpan<usize>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: Ident,
}

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub name: Ident,
}

#[derive(Debug, Clone)]
pub struct PropertyDecl {
    pub name: Ident,
    pub mutability: Mutability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Val,
    Var,
}
