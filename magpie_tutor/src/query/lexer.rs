//! Implementation of the Query Lexer
//!
//! The lexer is a simple lexer using regex to separate into large chunk to then be use to split
//! off into smaller token that the parser can use later.
//!
//! You can check the the regex is [`QUERY_REGEX`]

use crate::QUERY_REGEX;

#[derive(Debug, PartialEq)]
pub enum Token {
    Eof,

    OpenParen,
    CloseParen,

    Str(String),
    Num(isize),

    Name,
    Desc,

    Rarity,
    Temple,
    Tribe,

    Attack,
    Health,

    Sigil,
    SpAtk,

    Costs,
    CostType,

    Trait,

    Or,
    Not,

    Colon,

    Equal,
    Greater,
    GreaterEq,
    Less,
    LessEq,
}

/// Tokenize a given query. Fail on unrecognized token.
pub fn tokenize_query(query: &str) -> Result<Vec<Token>, String> {
    let mut tokens = vec![];
    for tk in QUERY_REGEX.captures_iter(query).map(|c| {
        (
            c.get(1).map(|m| m.as_str()), // string: ".+"
            c.get(2).map(|m| m.as_str()), // singular word: [-\w]+
            c.get(3).map(|m| m.as_str()), // symbol matches: [^\s\w"-]*
        )
    }) {
        tokens.push(match tk {
            // Simple string macthes
            (Some(str), ..) => Token::Str(str.to_owned()),
            // Single word matches. To reduce complexicity these are also responsible for number
            // matching so we try to convert to number first before sending out a string token
            (_, Some(sing), ..) => match sing {
                "name" | "n" => Token::Name,
                "description" | "d" => Token::Desc,
                "rarity" | "r" => Token::Rarity,
                "temple" | "tp" => Token::Temple,
                "tribe" | "tb" => Token::Tribe,
                "attack" | "a" => Token::Attack,
                "health" | "h" => Token::Health,
                "sigil" | "s" => Token::Sigil,
                "spatk" | "sp" => Token::SpAtk,
                "cost" | "c" => Token::Costs,
                "costtype" | "ct" => Token::CostType,
                "trait" | "tr" => Token::Trait,

                "or" => Token::Or,

                str => str
                    .parse()
                    .map(Token::Num)
                    .unwrap_or(Token::Str(str.to_owned())),
            },
            // Other symbol token, if they are not multi simple we try to separate them into simple
            // token and parse them.
            //
            // TODO: FIX THIS, BECAUSE IT GET CAUGHT ON "(<=" AND PRODUCE 3 TOKENS INSTEAD OF 2.
            (.., Some(sym)) => {
                tokens.extend(match_sym(sym)?);
                continue;
            }

            _ => unreachable!(),
        });
    }

    tokens.push(Token::Eof);

    Ok(tokens)
}

fn match_sym(sym: &str) -> Result<Vec<Token>, String> {
    Ok(vec![match sym {
        "(" => Token::OpenParen,
        ")" => Token::CloseParen,

        "!" => Token::Not,

        ":" => Token::Colon,
        "=" => Token::Equal,
        ">" => Token::Greater,
        "<" => Token::Less,

        ">=" => Token::GreaterEq,
        "<=" => Token::LessEq,

        sym if sym.len() > 1 => {
            let mut vec = vec![];
            for s in sym.chars() {
                vec.push(match_sym(&s.to_string())?.into_iter().next().unwrap());
            }
            return Ok(vec);
        }

        tk => return Err(format!("Unrecognized token: {tk}")),
    }])
}
