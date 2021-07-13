//! Handling Parsing of ASN.1 Types

use crate::error::Error;
use crate::structs::types::{Asn1ConstructedType, Asn1Type, Asn1TypeKind, ASN_BUILTIN_TYPE_KINDS};
use crate::tokenizer::Token;

use super::constraints::parse_constraints;
use super::utils::{
    expect_keyword, expect_one_of_keywords, expect_one_of_tokens, parse_set_ish_value,
};

// Parses the `Type` Expansion in the ASN.1 Grammar.
pub(super) fn parse_type<'parser>(tokens: &'parser [Token]) -> Result<(Asn1Type, usize), Error> {
    let mut consumed = 0;

    if !expect_one_of_tokens(
        tokens,
        &[Token::is_type_reference, Token::is_asn_builtin_type],
    )? {
        return Err(unexpected_token!(
            "'Type Reference' or 'Builtin Type'",
            tokens[0]
        ));
    }

    // Now: Parse The Type definition.
    let token = &tokens[0];
    let typestr = token.text.as_str();
    let (kind, id, id_consumed) = match typestr {
        "BIT-STRING" => {
            let (id, id_consumed) = parse_bit_string_type(tokens)?;
            (
                ASN_BUILTIN_TYPE_KINDS.get(typestr).unwrap().clone(),
                id,
                id_consumed,
            )
        }

        "ENUMERATED" => {
            let (id, id_consumed) = parse_enumerated_type(tokens)?;
            (
                ASN_BUILTIN_TYPE_KINDS.get(typestr).unwrap().clone(),
                id,
                id_consumed,
            )
        }

        "INTEGER" | "BOOLEAN" | "NULL" | "OBJECT-IDENTIFIER" | "UTF8String" | "IA5String"
        | "PrintableString" | "CHARACTER-STRING" => (
            ASN_BUILTIN_TYPE_KINDS.get(typestr).unwrap().clone(),
            token.text.clone(),
            1,
        ),

        "SET" | "SEQUENCE" | "CHOICE" => parse_constructed_type(tokens)?,

        _ => (Asn1TypeKind::default(), token.text.clone(), 1),
    };
    consumed += id_consumed;

    let (constraints, constraints_str_consumed) = match parse_constraints(&tokens[consumed..]) {
        Ok((s, c)) => (Some(s), c),
        Err(_) => (None, 0),
    };
    consumed += constraints_str_consumed;

    Ok((
        Asn1Type {
            id,
            kind,
            constraints,
        },
        consumed,
    ))
}

fn parse_bit_string_type<'parser>(_tokens: &'parser [Token]) -> Result<(String, usize), Error> {
    Err(parse_error!("Not Implemented yet!"))
}

fn parse_enumerated_type<'parser>(tokens: &'parser [Token]) -> Result<(String, usize), Error> {
    let mut consumed = 0;

    if !expect_keyword(tokens, "ENUMERATED")? {
        return Err(unexpected_token!("ENUMERATED", tokens[0]));
    }
    consumed += 1;

    let (def, def_consumed) = parse_set_ish_value(&tokens[consumed..])?;
    consumed += def_consumed;

    Ok((["ENUMERATED".to_string(), def].to_vec().join(" "), consumed))
}

fn parse_constructed_type<'parser>(
    tokens: &'parser [Token],
) -> Result<(Asn1TypeKind, String, usize), Error> {
    let mut consumed = 0;

    if !expect_one_of_keywords(tokens, &["SEQUENCE", "SET", "CHOICE"])? {
        return Err(unexpected_token!("'SEQUENCE', 'SET', 'CHOICE'", tokens[0]));
    }
    let id = tokens[0].text.clone();
    consumed += 1;

    let (def, def_consumed) = parse_set_ish_value(&tokens[consumed..])?;
    consumed += def_consumed;

    Ok((
        Asn1TypeKind::Constructed(Asn1ConstructedType::Sequence),
        [id, def].to_vec().join(" "),
        consumed,
    ))
}

// TODO: Add test cases
