use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::recovery::skip_then_retry_until;
use chumsky::span::SimpleSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Package,
    Import,
    Class,
    Fun,
    Val,
    Var,
    Ident(String),
    Int(String),
    String(String),
    Char(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    ColonColon,
    Semi,
    Eq,
    Arrow,
    Question,
    At,
    LAngle,
    RAngle,
    Op(String),
}

pub type Span = SimpleSpan<usize>;
pub type SpannedToken = (Token, Span);

pub fn lex<'src>(source: &'src str) -> (Vec<SpannedToken>, Vec<Simple<'src, char>>) {
    let result = lexer().parse(source);
    let (tokens, errors) = result.into_output_errors();
    (tokens.unwrap_or_default(), errors)
}

fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<SpannedToken>, extra::Err<Simple<'src, char>>> {
    let keyword = choice((
        text::ascii::keyword("package").to(Token::Package),
        text::ascii::keyword("import").to(Token::Import),
        text::ascii::keyword("class").to(Token::Class),
        text::ascii::keyword("fun").to(Token::Fun),
        text::ascii::keyword("val").to(Token::Val),
        text::ascii::keyword("var").to(Token::Var),
    ));

    let ident = text::unicode::ident().map(|ident: &str| Token::Ident(ident.to_string()));
    let int = text::int(10).map(|digits: &str| Token::Int(digits.to_string()));

    let string = just('"')
        .ignore_then(
            any()
                .filter(|c: &char| *c != '"' && *c != '\n' && *c != '\r')
                .repeated()
                .collect::<String>(),
        )
        .then_ignore(just('"'))
        .map(Token::String);

    let char_lit = just('\'')
        .ignore_then(any().filter(|c: &char| *c != '\'' && *c != '\n' && *c != '\r'))
        .then_ignore(just('\''))
        .map(|ch| Token::Char(ch.to_string()));

    let punct = choice((
        just("::").to(Token::ColonColon),
        just("->").to(Token::Arrow),
        just('(').to(Token::LParen),
        just(')').to(Token::RParen),
        just('{').to(Token::LBrace),
        just('}').to(Token::RBrace),
        just('[').to(Token::LBracket),
        just(']').to(Token::RBracket),
        just('<').to(Token::LAngle),
        just('>').to(Token::RAngle),
        just(',').to(Token::Comma),
        just('.').to(Token::Dot),
        just(':').to(Token::Colon),
        just(';').to(Token::Semi),
        just('=').to(Token::Eq),
        just('?').to(Token::Question),
        just('@').to(Token::At),
    ));

    let op = one_of("+-*/%!=|&^")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Token::Op);

    let line_comment = just("//")
        .then(any().filter(|c: &char| *c != '\n').repeated())
        .ignored();

    let block_comment = just("/*")
        .then(any().repeated())
        .then_ignore(just("*/"))
        .ignored();

    let whitespace = one_of(" \t\r\n")
        .repeated()
        .at_least(1)
        .ignored();

    let ignored = choice((whitespace, line_comment, block_comment)).repeated();

    let token = choice((keyword, int, string, char_lit, punct, op, ident))
        .map_with(|token, e| (token, e.span()));

    token
        .padded_by(ignored)
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
        .then_ignore(end())
}
