//! Contain the main querying function and implementations.
//!
//! The query take in a input pass it into a simple lexer and then into a simple parser to get a
//! list of keywords. These keywords then get converted into a set of filters to then be use for
//! [`QueryBuilder`]

use std::vec;

use magpie_engine::prelude::*;
use poise::serenity_prelude::{colours::roles, CreateEmbed};

use crate::{Filters, Set};

mod lexer;
mod parser;

use lexer::tokenize_query;

use self::parser::QueryParser;

macro_rules! unwrap {
    ($expr:expr) => {
        match $expr {
            Ok(it) => it,
            Err(err) => {
                return CreateEmbed::new()
                    .color(roles::RED)
                    .title("Query Error")
                    .description(err)
            }
        }
    };
}

/// Query a message
pub fn query_message(sets: Vec<&Set>, query: &str) -> CreateEmbed {
    let tokens = unwrap!(tokenize_query(query));
    let keywords = unwrap!(QueryParser::gen_ast_with(tokens));

    let mut filters: Vec<Filters> = vec![];

    for kw in keywords {
        filters.push(unwrap!(kw.try_into()));
    }

    let query = QueryBuilder::with_filters(sets, filters).query();

    let output = query
        .cards
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    CreateEmbed::new()
        .color(roles::PURPLE)
        .title(format!(
            "Result: {} cards in selected sets",
            query.cards.len()
        ))
        .description(if query.cards.len() >= 200 || output.len() >= 2000 {
            String::from("Too many results...Try narrowing your search")
        } else {
            format!(
                "Cards that {}\n{}",
                query
                    .filters
                    .into_iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(" and "),
                output
            )
        })
}
