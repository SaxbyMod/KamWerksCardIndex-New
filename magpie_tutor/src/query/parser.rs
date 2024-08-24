//! Implementation of the Query Syntax Parser.
//!
//! Contain both the parser and the conversion between [`Keyword`] and [`Filters`]
//!
//! The parser is a simple Recursive descent parser. It will emit out a set of keywords then later
//! then main query function will convert those keyword into filter to put into [`QueryBuilder`]
//!
//! Here a simple top down view of the parser in
//! pesudo EBFN
//!
//! ```ebnf
//! (*
//!     Uppercase mean they are emit by the tokenizer, they are usually too long and you can infer
//!     what they mean.
//! *)
//!
//! program = { expr }
//!
//! expr = not { "or" not }
//! not = [ "!" ] keyword
//! keyword = str_keyword | cmp_keyword
//!
//! str_keyword = STR_KEYWORD ":" ( NUM | STR )
//! cmp_keyword = CMP_KEYWORD ( ":" | "=" | ">" | "<" | ">=" | "<=" ) NUM
//! ```

use std::{fmt::Display, vec};

use magpie_engine::prelude::*;

use crate::{
    engine::{CostType, FilterExt},
    Filters, COST_REGEX,
};

use super::lexer::Token;

#[derive(Debug)]
pub enum Keyword {
    Name(String),
    Desc(String),

    Rarity(String),
    Temple(String),
    Tribe(String),

    Attack(QueryOrder, isize),
    Health(QueryOrder, isize),

    Sigil(String),
    SpAtk(String),

    Costs(String),
    CostType(String),

    Trait(String),

    Or(Box<Keyword>, Box<Keyword>),
    Not(Box<Keyword>),
}

/// helper to generate match tree to map token to keyword
macro_rules! tk_to_kw {
    (match $tk:ident($value:ident) {$($name:ident),*}) => {
        match $tk {
            $(Token::$name => Keyword::$name($value),)*
            _ => unreachable!(),
        }
    };
}

#[derive(Debug)]
pub enum ParseErr {
    InvalidKeyword(Token),
    ExpectToken(Token, Token),
    ExpectTokens(Vec<Token>, Token),
}

impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErr::InvalidKeyword(tk) => write!(f, "Invalid keyword {tk:?}"),
            ParseErr::ExpectToken(expect, tk) => {
                write!(f, "Expected {expect:?} but found {tk:?}")
            }
            ParseErr::ExpectTokens(expects, tk) => {
                write!(f, "Expected {expects:?} by found {tk:?}")
            }
        }
    }
}

impl From<ParseErr> for String {
    fn from(val: ParseErr) -> Self {
        val.to_string()
    }
}

pub struct QueryParser {
    tokens: Vec<Token>,
}

type ParseRes = Result<Keyword, ParseErr>;

impl QueryParser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();
        QueryParser { tokens }
    }

    pub fn gen_ast_with(tokens: Vec<Token>) -> Result<Vec<Keyword>, ParseErr> {
        Self::new(tokens).gen_ast()
    }

    pub fn gen_ast(mut self) -> Result<Vec<Keyword>, ParseErr> {
        let mut ast = Vec::new();

        while !self.tokens.is_empty() && self.not_eof() {
            ast.push(self.parse()?);
        }

        Ok(ast)
    }

    fn parse(&mut self) -> ParseRes {
        self.parse_or()
    }

    fn parse_or(&mut self) -> ParseRes {
        let mut left = self.parse_not()?;

        while self.curr_is(&Token::Or) {
            self.next();
            let right = self.parse_not()?;
            left = Keyword::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> ParseRes {
        if !self.curr_is(&Token::Not) {
            return self.parse_keyword();
        }
        self.next();
        Ok(Keyword::Not(Box::new(self.parse_keyword()?)))
    }

    fn parse_keyword(&mut self) -> ParseRes {
        match self.curr() {
            Token::Name
            | Token::Desc
            | Token::Rarity
            | Token::Temple
            | Token::Tribe
            | Token::Sigil
            | Token::SpAtk
            | Token::Costs
            | Token::CostType
            | Token::Trait => self.parse_str_keyword(),

            Token::Attack | Token::Health => self.parse_cmp_keyword(),

            Token::OpenParen => {
                self.next();
                let t = self.parse();
                self.expect_token(Token::CloseParen)?;
                t
            }

            _ => Err(ParseErr::InvalidKeyword(self.next())),
        }
    }

    fn parse_str_keyword(&mut self) -> ParseRes {
        let keyword = self.next();

        self.expect_token(Token::Colon)?;

        let val = match self.next() {
            Token::Num(num) => num.to_string(),
            Token::Str(str) => str,
            tk => {
                return Err(ParseErr::ExpectTokens(
                    vec![Token::Num(0), Token::Str(String::new())],
                    tk,
                ))
            }
        };

        Ok(
            tk_to_kw!(match keyword(val) { Name, Desc, Rarity, Temple, Tribe, Sigil, SpAtk, Costs, CostType, Trait }),
        )
    }

    fn parse_cmp_keyword(&mut self) -> ParseRes {
        let keyword = self.next();

        let cmp = match self.next() {
            Token::Colon | Token::Equal => QueryOrder::Equal,
            Token::Greater => QueryOrder::Greater,
            Token::GreaterEq => QueryOrder::GreaterEqual,
            Token::Less => QueryOrder::Less,
            Token::LessEq => QueryOrder::LessEqual,

            tk => {
                return Err(ParseErr::ExpectTokens(
                    vec![
                        Token::Colon,
                        Token::Equal,
                        Token::Greater,
                        Token::GreaterEq,
                        Token::Less,
                        Token::LessEq,
                    ],
                    tk,
                ))
            }
        };

        let num = match self.next() {
            Token::Num(num) => num,
            tk => return Err(ParseErr::ExpectToken(Token::Num(0), tk)),
        };

        Ok(match keyword {
            Token::Attack => Keyword::Attack(cmp, num),
            Token::Health => Keyword::Health(cmp, num),
            _ => unreachable!(),
        })
    }

    fn not_eof(&self) -> bool {
        !matches!(self.curr(), Token::Eof)
    }

    fn curr(&self) -> &Token {
        self.tokens.last().unwrap()
    }

    fn curr_is(&self, what: &Token) -> bool {
        self.curr() == what
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap()
    }

    fn expect_token(&mut self, what: Token) -> Result<Token, ParseErr> {
        let next = self.next();
        if next == what {
            Ok(next)
        } else {
            Err(ParseErr::ExpectToken(what, next))
        }
    }
}

// Helper to convert keyword to filter
macro_rules! map_kw_ft {
    ($value:ident => $type:ident, $($pat:pat => $variant:ident),*) => {
        match $value.as_str() {
            $($pat => ft!($type($type::$variant.into())),)*
            _ => Err(concat!("Invalid", stringify!($type)))
        }
    };
}
macro_rules! ft { ($type:ident ($($value:expr),*)) => {Ok(Filters::$type($($value,)*)) }; }
macro_rules! ft_some { ($type:ident ($($value:expr),*)) => {ft!($type(Some($($value,)*))) }; }

impl TryFrom<Keyword> for Filters {
    type Error = &'static str;
    fn try_from(value: Keyword) -> Result<Filters, Self::Error> {
        match value {
            Keyword::Name(name) => ft!(Name(name)),
            Keyword::Desc(desc) => ft!(Description(desc)),
            Keyword::Rarity(rarity) => map_kw_ft! {
                rarity => Rarity,
                "side" | "s" => SIDE,
                "common" | "c" => COMMON,
                "uncommon" | "u" => UNCOMMON,
                "rare" | "r" => RARE,
                "unique" | "n" => UNIQUE
            },
            Keyword::Temple(temple) => map_kw_ft! {
                temple => Temple,
                "beast" | "b" => BEAST,
                "undead" | "u" => UNDEAD,
                "technology" | "tech" | "t" => TECH,
                "magick" | "m" => MAGICK,
                "fool" | "f" => FOOL,
                "artistry" | "a" => ARTISTRY
            },
            Keyword::Tribe(tribe) => ft!(Tribe(Some(tribe))),
            Keyword::Attack(cmp, attack) => ft!(Attack(cmp, attack)),
            Keyword::Health(cmp, health) => ft!(Health(cmp, health)),
            Keyword::Sigil(sigil) => ft!(Sigil(sigil)),
            Keyword::SpAtk(spatk) => map_kw_ft! {
                spatk => SpAtk,
                "mox" => MOX,
                "green" => GREEN_MOX,
                "mirror" => MIRROR,
                "ant" => ANT,
                "bone" => BONE,
                "bell" => BELL,
                "card" => CARD
            },
            Keyword::Costs(str) => {
                let mut costs = Costs::default();
                for (count, cost_type) in COST_REGEX.captures_iter(&str).map(|c| {
                    (
                        c.get(1)
                            .and_then(|m| m.as_str().parse::<isize>().ok())
                            .unwrap_or(1),
                        c.get(2).and_then(|m| m.as_str().chars().next()).unwrap(),
                    )
                }) {
                    match cost_type {
                        'b' => costs.blood = count,
                        'o' => costs.bone = count,
                        'e' => costs.energy = count,
                        'r' => {
                            costs.mox |= Mox::O;
                            if let Some(ref mut c) = costs.mox_count {
                                c.r = count as usize;
                            }
                        }
                        'g' => {
                            costs.mox |= Mox::G;
                            if let Some(ref mut c) = costs.mox_count {
                                c.g = count as usize;
                            }
                        }
                        'u' => {
                            costs.mox |= Mox::B;
                            if let Some(ref mut c) = costs.mox_count {
                                c.b = count as usize;
                            }
                        }
                        'y' => {
                            costs.mox |= Mox::Y;
                            if let Some(ref mut c) = costs.mox_count {
                                c.y = count as usize;
                            }
                        }
                        _ => return Err("Invalid Cost"),
                    }
                }

                ft_some!(Costs(costs))
            }
            Keyword::CostType(c) => {
                let mut t = CostType::empty();
                for c in c.chars() {
                    t |= match c {
                        'b' => CostType::BLOOD,
                        'o' => CostType::BONE,
                        'e' => CostType::ENERGY,
                        'm' => CostType::MOX,
                        _ => return Err("Invalid Cost Type"),
                    }
                }

                ft!(Extra(FilterExt::CostType(t)))
            }
            Keyword::Trait(t) => match t.as_str() {
                "conductive" => {
                    ft_some!(Traits(Traits::with_flags(TraitsFlag::CONDUCTIVE)))
                }
                "ban" => {
                    ft_some!(Traits(Traits::with_flags(TraitsFlag::BAN)))
                }
                "terrain" => {
                    ft_some!(Traits(Traits::with_flags(TraitsFlag::TERRAIN)))
                }
                "hard" => {
                    ft_some!(Traits(Traits::with_flags(TraitsFlag::HARD)))
                }
                _ => {
                    ft_some!(Traits(Traits::with_string(
                        t.split(',').map(ToOwned::to_owned).collect()
                    )))
                }
            },
            Keyword::Or(a, b) => ft!(Or(Box::new((*a).try_into()?), Box::new((*b).try_into()?))),
            Keyword::Not(a) => ft!(Not(Box::new((*a).try_into()?))),
        }
    }
}
