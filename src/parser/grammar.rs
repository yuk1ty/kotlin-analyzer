use chumsky::error::Simple;
use chumsky::input::{IterInput, ValueInput};
use chumsky::prelude::*;
use chumsky::recovery::{nested_delimiters, via_parser};
use chumsky::span::SimpleSpan;

use crate::diagnostics::{Diagnostic, Severity};
use crate::parser::ast::{ClassDecl, Decl, File, FunDecl, Ident, Mutability, PropertyDecl};
use crate::parser::lexer::{SpannedToken, Token};

pub fn parse(tokens: &[SpannedToken], text: &str) -> (Option<File>, Vec<Diagnostic>) {
    let eoi = SimpleSpan::new((), text.len()..text.len());
    let input = IterInput::new(tokens.iter().cloned(), eoi);
    let result = parser().parse(input);
    let (output, errors) = result.into_output_errors();
    let diagnostics = errors
        .into_iter()
        .map(|err| diag_from_parse_error(text, err))
        .collect();
    (output, diagnostics)
}

fn diag_from_parse_error(text: &str, err: Simple<'_, Token>) -> Diagnostic {
    let span = err.span().clone();
    let range = super::span_to_range(text, span);
    let message = match err.found() {
        Some(found) => format!("Unexpected token: {found:?}"),
        None => "Unexpected end of input".to_string(),
    };

    Diagnostic::new(range, Severity::Error, message)
}

fn parser<'src, I>() -> impl Parser<'src, I, File, extra::Err<Simple<'src, Token>>>
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<usize>>,
{
    let ident = select! { Token::Ident(name) => name }
        .map_with(|name, e| Ident {
            name,
            span: e.span(),
        });
    let semi = just(Token::Semi).or_not();

    let qualified = ident
        .clone()
        .then(just(Token::Dot).ignore_then(ident.clone()).repeated())
        .ignored();

    let package_decl = just(Token::Package)
        .ignore_then(qualified.clone())
        .then_ignore(semi.clone())
        .ignored();

    let import_decl = just(Token::Import)
        .ignore_then(qualified.clone())
        .then_ignore(semi.clone())
        .ignored();

    let balanced = recursive(|balanced| {
        let group = choice((
            balanced
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .ignored(),
            balanced
                .clone()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
                .ignored(),
            balanced
                .clone()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .ignored(),
            any()
                .filter(|token: &Token| {
                    !matches!(
                        token,
                        Token::LParen
                            | Token::RParen
                            | Token::LBrace
                            | Token::RBrace
                            | Token::LBracket
                            | Token::RBracket
                    )
                })
                .ignored(),
        ));

        group.repeated().ignored()
    });

    let type_ref = recursive(|type_ref| {
        let type_args = type_ref
            .clone()
            .map(|_| ())
            .or(select! { Token::Op(op) if op == "*" => () })
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LAngle), just(Token::RAngle))
            .ignored();

        let simple_type = ident
            .clone()
            .then(type_args.or_not())
            .ignored();

        let func_type = type_ref
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .or_not()
            .map(|items| items.unwrap_or_default())
            .delimited_by(just(Token::LParen), just(Token::RParen))
        .then_ignore(just(Token::Arrow))
        .then(type_ref.clone())
        .ignored();

        let parenthesized = type_ref
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .ignored();

        choice((func_type, parenthesized, simple_type))
            .then(just(Token::Question).or_not())
            .ignored()
    });

    let type_params = ident
        .clone()
        .then(just(Token::Colon).ignore_then(type_ref.clone()).or_not())
        .ignored()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LAngle), just(Token::RAngle))
        .ignored();

    let expr = recursive(|expr| {
        let literal = select! {
            Token::Int(_) => (),
            Token::String(_) => (),
            Token::Char(_) => (),
        };

        let atom = choice((
            literal.ignored(),
            ident.clone().ignored(),
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .ignored(),
            balanced.clone().delimited_by(just(Token::LBrace), just(Token::RBrace)).ignored(),
        ));

        let call_args = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .or_not()
            .map(|items| items.unwrap_or_default())
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .ignored();

        let postfix = atom
            .clone()
            .then(
                choice((
                    call_args.clone(),
                    just(Token::Dot).ignore_then(ident.clone()).ignored(),
                ))
                .repeated(),
            )
            .ignored();

        let operator = choice((
            select! { Token::Op(_) => () }.ignored(),
            just(Token::LAngle).ignored(),
            just(Token::RAngle).ignored(),
            just(Token::Eq).ignored(),
        ));

        postfix
            .clone()
            .then(operator.then(postfix).repeated())
            .ignored()
    });

    let params = just(Token::Val)
        .or(just(Token::Var))
        .or_not()
        .ignore_then(ident.clone())
        .then(just(Token::Colon).ignore_then(type_ref.clone()).or_not())
        .then(just(Token::Eq).ignore_then(expr.clone()).or_not())
        .ignored()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .or_not()
        .map(|items| items.unwrap_or_default())
        .delimited_by(just(Token::LParen), just(Token::RParen))
    .recover_with(via_parser(nested_delimiters(
        Token::LParen,
        Token::RParen,
        [
            (Token::LBrace, Token::RBrace),
            (Token::LBracket, Token::RBracket),
        ],
        |_| Vec::new(),
    )))
    .ignored();

    let type_annotation = just(Token::Colon)
        .ignore_then(type_ref.clone())
        .ignored()
        .or_not();

    let block = balanced
        .clone()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .recover_with(via_parser(nested_delimiters(
            Token::LBrace,
            Token::RBrace,
            [
                (Token::LParen, Token::RParen),
                (Token::LBracket, Token::RBracket),
            ],
            |_| (),
        )))
        .ignored();

    let params_clone = params.clone();
    let fun_decl = just(Token::Fun)
        .ignore_then(type_params.clone().or_not())
        .ignore_then(ident.clone())
        .then_ignore(type_params.clone().or_not())
        .then_ignore(params)
        .then_ignore(type_annotation.clone())
        .then_ignore(choice((
            block.clone(),
            just(Token::Eq).ignore_then(expr.clone()).ignored(),
            empty().ignored(),
        )))
        .map(|name| Decl::Fun(FunDecl { name }));

    let class_decl = just(Token::Class)
        .ignore_then(ident.clone())
        .then_ignore(type_params.clone().or_not())
        .then_ignore(params_clone.or_not())
        .then_ignore(
            just(Token::Colon)
                .ignore_then(type_ref.clone().separated_by(just(Token::Comma)).allow_trailing())
                .or_not()
                .ignored(),
        )
        .then_ignore(block.clone().or_not())
        .map(|name| Decl::Class(ClassDecl { name }));

    let property_decl = choice((
        just(Token::Val).to(Mutability::Val),
        just(Token::Var).to(Mutability::Var),
    ))
    .then(ident)
    .then_ignore(type_annotation)
    .then_ignore(choice((
        just(Token::Eq).ignore_then(expr.clone()).ignored(),
        empty().ignored(),
    )))
    .then_ignore(semi)
    .map(|(mutability, name)| Decl::Property(PropertyDecl { name, mutability }));

    let sync = choice((
        just(Token::Class).ignored(),
        just(Token::Fun).ignored(),
        just(Token::Val).ignored(),
        just(Token::Var).ignored(),
        end(),
    ));

    let recovery = any()
        .and_is(sync.clone().not())
        .ignored()
        .repeated()
        .at_least(1)
        .ignored()
        .to(None);

    let item = choice((
        package_decl.to(None),
        import_decl.to(None),
        class_decl.map(Some),
        fun_decl.map(Some),
        property_decl.map(Some),
    ))
    .recover_with(via_parser(recovery));

    item.repeated()
        .collect::<Vec<_>>()
        .map(|items| File {
            declarations: items.into_iter().flatten().collect(),
        })
        .then_ignore(end())
}
