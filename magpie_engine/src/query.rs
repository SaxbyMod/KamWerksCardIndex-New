//! Implementation for querying card
//!
//! To query a card you first start with creating a [`QueryBuilder`] then build up your query using
//! [`Filters`] then finally calling [`QueryBuilder::query`] to obtain a [`Query`]
use crate::data::{Card, Costs, Rarity, Set, SpAtk, Traits};
use crate::Ptr;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::vec;

/// The query object containing your results and infomation about the filter that give you
/// the results.
#[derive(Debug)]
pub struct Query {
    /// The result of this query
    pub cards: Vec<Ptr<Card>>,
    /// The filter that produce this query
    pub filters: Vec<Filters>,
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.cards
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<&str>>()
                .join("\n")
        )
    }
}

/// Type shorthand for a filter.
pub type FilterFn = Box<dyn Fn(Ptr<Card>) -> bool>;

/// Query builder, it contain the set and is the main way to query cards
///
/// You must first build up your query then lastly call `.query()` to compile all the condition and
/// start querying for cards
pub struct QueryBuilder<'a> {
    /// All the set that is use for this query
    pub sets: Vec<&'a Set>,

    filters: Vec<Filters>,
    funcs: Vec<FilterFn>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a new [`QueryBuilder`] from a collection of set.
    pub fn new(sets: Vec<&'a Set>) -> Self {
        QueryBuilder {
            sets,
            filters: vec![],
            funcs: vec![],
        }
    }
    /// Add a new filter to this query.
    pub fn add_filter(mut self, filter: Filters) -> Self {
        self.filters.push(filter.clone());
        self.funcs.push(filter.to_fn());
        self
    }

    /// Compile all the query and give you the result.
    pub fn query(self) -> Query {
        let filter = move |c: Ptr<Card>| self.funcs.iter().all(move |f| f(c.clone()));

        Query {
            filters: self.filters,
            cards: self
                .sets
                .iter()
                .map(|s| &s.cards)
                .flatten()
                .into_iter()
                .filter(|&c| filter(c.clone()))
                .map(|c| c.to_owned())
                .collect(),
        }
    }
}

/// Enum for When query stuff
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filters {
    /// Filter for card name.
    ///
    /// The value in this variant is the name to filter for.
    Name(String),
    /// Filter for card description.
    ///
    /// The value in this variant is the description to filter for.
    Description(String),

    /// Filter for card rarity.
    ///
    /// The value in this variant is the rarity to filter for.
    Rarity(Rarity),
    /// Filter for card rarity
    ///
    /// The value in this variant is bit flags to match against.
    Temple(u16),

    /// Filter for the card attack
    ///
    /// The first value is what what qualifier or comparasion to compare the attack against, the
    /// second is for equality (mainly for >=, <=) and lastly is the value to compare against
    Attack(Ordering, bool, isize),
    /// Filter for the card attack
    ///
    /// The first value is what what qualifier or comparasion to compare the health against, the
    /// second is for equality (mainly for >=, <=) and lastly is the value to compare against
    Health(Ordering, bool, isize),

    /// Filter for card sigils
    ///
    /// The value in this variant is the sigil name to filter for in the card sigils.
    Sigils(String),

    /// filter for card special attack.
    ///
    /// The value in this variant is the special attack to filter for.
    SpAtk(Option<SpAtk>),

    /// Filter for card cost
    ///
    /// The value in this variant is cost table to filter for
    Costs(Option<Costs>),
    /// Filter for card trait
    ///
    /// The value in this variant is trait table to filter for
    Traits(Option<Traits>),
}

/// Trait for a Filter.
pub trait Filter: Clone + Eq {
    /// Turn the value into a filter that take a card and return a bool
    fn to_fn(self) -> FilterFn;
}

impl Filter for Filters {
    fn to_fn(self) -> FilterFn {
        match self {
            Filters::Name(name) => Box::new(move |c| c.name.to_lowercase().contains(name.as_str())),
            Filters::Description(desc) => {
                Box::new(move |c| c.description.to_lowercase().contains(desc.as_str()))
            }

            Filters::Rarity(rarity) => Box::new(move |c| c.rarity == rarity),
            Filters::Temple(temple) => Box::new(move |c| c.temple == temple),
            Filters::Attack(ord, eq, attack) => {
                Box::new(move |c| (eq && c.attack == attack) || c.attack.cmp(&attack) == ord)
            }
            Filters::Health(ord, eq, heath) => {
                Box::new(move |c| (eq && c.health == heath) || c.health.cmp(&heath) == ord)
            }
            Filters::Sigils(s) => {
                let lower = s.to_lowercase();
                Box::new(move |c| {
                    c.sigils
                        .iter()
                        .map(|s| s.to_lowercase())
                        .find(|s| s.eq(&lower))
                        .is_some()
                })
            }
            Filters::SpAtk(a) => Box::new(move |c| c.sp_atk == a),
            Filters::Costs(cost) => Box::new(move |c| c.costs == cost),
            Filters::Traits(traits) => Box::new(move |c| c.traits == traits),
        }
    }
}

impl Filter for () {
    fn to_fn(self) -> FilterFn {
        unimplemented!()
    }
}
