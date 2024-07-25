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

/// Query a message
pub fn query_message(sets: Vec<&Set>, query: &str) -> Result<CreateEmbed, CreateEmbed> {
    let tokens = tokenize_query(query).map_err(error_embed)?;
    let keywords = QueryParser::gen_ast_with(tokens).map_err(error_embed)?;

    let mut filters: Vec<Filters> = vec![];

    for kw in keywords {
        filters.push(kw.try_into().map_err(|e| error_embed(e))?);
    }

    let query = QueryBuilder::with_filters(sets, filters).query();

    Ok(CreateEmbed::new()
        .color(roles::PURPLE)
        .title(format!(
            "Result: {} cards in selected sets",
            query.cards.len()
        ))
        .description(if query.cards.len() >= 200 {
            String::from("Too many results...Try narrowing your search")
        } else {
            query
                .cards
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        }))
}

fn error_embed(desc: impl ToString) -> CreateEmbed {
    CreateEmbed::new()
        .color(roles::RED)
        .title("Query Error")
        .description(desc.to_string())
}
