//! Implementation for querying card
//!
//! To query a card you first start with creating a [`QueryBuilder`] then build up your query using
//! [`Filters`] then finally calling [`QueryBuilder::query`] to obtain a [`Query`].
use crate::{Attack, Card, Costs, Rarity, Set, SpAtk, Traits};
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::vec;

/// The query object containing your results and infomation about the filter that give you
/// the results.
#[derive(Debug)]
pub struct Query<'a, E, C, F>
where
    E: Clone,
    C: Clone + PartialEq,
    F: ToFilter<E, C>,
{
    /// The result of this query.
    pub cards: Vec<&'a Card<E, C>>,
    /// The filter that produce this query.
    pub filters: Vec<Filters<E, C, F>>,
}

impl<E, C, F> Display for Query<'_, E, C, F>
where
    E: Clone,
    C: Clone + PartialEq,
    F: ToFilter<E, C>,
{
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
pub type FilterFn<E, C> = Box<dyn Fn(&Card<E, C>) -> bool>;

/// Query builder, it contain the set and is the main way to query cards.
///
/// You must first build up your query then lastly call `.query()` to compile all the condition and
/// start querying for cards.
pub struct QueryBuilder<'a, E, C, F>
where
    E: Clone,
    C: Clone + PartialEq,
    F: ToFilter<E, C>,
{
    /// All the set that is use for this query.
    pub sets: Vec<&'a Set<E, C>>,

    filters: Vec<Filters<E, C, F>>,
    funcs: Vec<FilterFn<E, C>>,
}

impl<'a, E, C, F> QueryBuilder<'a, E, C, F>
where
    C: Clone + PartialEq + 'static,
    E: Clone + 'static,
    F: ToFilter<E, C> + 'static,
{
    /// Create a new [`QueryBuilder`] from a collection of set.
    #[must_use]
    pub fn new(sets: Vec<&'a Set<E, C>>) -> Self {
        QueryBuilder {
            sets,
            filters: vec![],
            funcs: vec![],
        }
    }

    /// Create a new [`QueryBuilder`] from some sets and filters.
    #[must_use]
    pub fn with_filters(sets: Vec<&'a Set<E, C>>, filters: Vec<Filters<E, C, F>>) -> Self {
        QueryBuilder {
            funcs: filters.clone().into_iter().map(|f| f.to_fn()).collect(),
            sets,
            filters,
        }
    }

    /// Add a new filter to this query.
    #[must_use]
    pub fn add_filter(mut self, filter: Filters<E, C, F>) -> Self {
        self.filters.push(filter.clone());
        self.funcs.push(filter.to_fn());
        self
    }

    /// Compile all the query and give you the result.
    #[must_use]
    pub fn query(self) -> Query<'a, E, C, F> {
        let filter = move |c: &Card<E, C>| self.funcs.iter().all(move |f| f(c));

        Query {
            filters: self.filters,
            cards: self
                .sets
                .iter()
                .flat_map(|s| &s.cards)
                .filter(|&c| filter(c))
                .collect(),
        }
    }
}

/// [`Ordering`](std::cmp::Ordering) extension for more ordering.
#[derive(Debug, Clone)]
pub enum QueryOrder {
    /// Greater than another.
    Greater,
    /// Greater than or equal to another.
    GreaterEqual,
    /// Equal to another.
    Equal,
    /// Less than or equal to another.
    LessEqual,
    /// Less than another.
    Less,
}

/// Filters to be apply to when querying card.
///
/// You can add custom filter by providing the `F` generic and implementing [`ToFilter`] trait for
/// it.
#[derive(Debug, Clone)]
pub enum Filters<E, C, F>
where
    E: Clone,
    C: Clone + PartialEq,
    F: ToFilter<E, C>,
{
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
    /// Filter for card tribe
    ///
    /// The value is the tribe or tribes to match against.
    Tribe(Option<String>),

    /// Filter for the card attack.
    ///
    /// The first value is what what qualifier or comparasion to compare the attack against, the
    /// second is the value to compare against.
    Attack(QueryOrder, isize),
    /// Filter for the card attack.
    ///
    /// The first value is what what qualifier or comparasion to compare the health against, the
    /// second is the value to compare against.
    Health(QueryOrder, isize),

    /// Filter for card sigil
    ///
    /// The value in this variant is the sigil name to filter for in the card sigils.
    Sigil(String),

    /// filter for card special attack.
    ///
    /// The value in this variant is the special attack to filter for.
    SpAtk(SpAtk),

    /// filter for card special attack saved as [`String`].
    ///
    /// The value in this variant is the special attack to filter for.
    StrAtk(String),

    /// Filter for card cost.
    ///
    /// The value in this variant is cost table to filter for.
    Costs(Option<Costs<C>>),
    /// Filter for card trait.
    ///
    /// The value in this variant is trait table to filter for.
    Traits(Option<Traits>),

    /// Logical `or` between 2 filters instead of the default and.
    Or(Box<Filters<E, C, F>>, Box<Filters<E, C, F>>),
    /// Logical `not` for a filter.
    Not(Box<Filters<E, C, F>>),

    /// Extra filter you can add.
    Extra(F),

    #[doc(hidden)]
    McGuffin(Infallible, PhantomData<C>),
    #[doc(hidden)]
    Cake(Infallible, PhantomData<E>),
}

/// Traits for converting a type to a [`FilterFn`].
///
/// The generic is for the cards extension.
pub trait ToFilter<E, C>: Clone
where
    E: Clone,
    C: Clone + PartialEq,
{
    /// Convert the value into a [`FilterFn`].
    fn to_fn(self) -> FilterFn<E, C>;
}

/// Generate code to help with matching [`QueryOrder`].
#[macro_export]
macro_rules! match_query_order {
    ($ord:expr, $a:expr, $b:expr) => {
        match $ord {
            QueryOrder::Greater => $a > $b,
            QueryOrder::GreaterEqual => $a >= $b,
            QueryOrder::Equal => $a == $b,
            QueryOrder::LessEqual => $a <= $b,
            QueryOrder::Less => $a < $b,
        }
    };
}

impl<E, C, F> ToFilter<E, C> for Filters<E, C, F>
where
    E: Clone + 'static,
    C: Clone + PartialEq + 'static,
    F: ToFilter<E, C> + 'static,
{
    fn to_fn(self) -> FilterFn<E, C> {
        match self {
            Filters::Name(name) => {
                Box::new(move |c| c.name.to_lowercase().contains(&name.to_lowercase()))
            }
            Filters::Description(desc) => {
                Box::new(move |c| c.description.to_lowercase().contains(&desc.to_lowercase()))
            }

            Filters::Rarity(rarity) => Box::new(move |c| c.rarity == rarity),
            Filters::Temple(temple) => Box::new(move |c| c.temple == temple),
            Filters::Tribe(tribes) => Box::new(move |c| match &c.tribes {
                Some(tr) if tribes.is_some() => tr
                    .to_lowercase()
                    .contains(&tribes.as_ref().unwrap().to_lowercase()),
                _ => c.tribes == tribes,
            }),
            Filters::Attack(ord, attack) => Box::new(move |c| {
                if let Attack::Num(a) = c.attack {
                    match_query_order!(ord, a, attack)
                } else {
                    false
                }
            }),
            Filters::Health(ord, health) => {
                Box::new(move |c| match_query_order!(ord, c.health, health))
            }
            Filters::Sigil(s) => {
                let lower = s.to_lowercase();
                Box::new(move |c| {
                    c.sigils
                        .iter()
                        .map(|s| s.to_lowercase())
                        .any(|s| s.eq(&lower))
                })
            }
            Filters::SpAtk(a) => Box::new(move |c| {
                if let Attack::SpAtk(sp) = &c.attack {
                    *sp == a
                } else {
                    false
                }
            }),
            Filters::StrAtk(s) => Box::new(move |c| {
                if let Attack::Str(str) = &c.attack {
                    *str == s
                } else {
                    false
                }
            }),
            Filters::Costs(cost) => Box::new(move |c| c.costs == cost),
            Filters::Traits(traits) => Box::new(move |c| c.traits == traits),

            Filters::Or(a, b) => {
                let a = a.to_fn();
                let b = b.to_fn();
                Box::new(move |c| a(c) || b(c))
            }

            Filters::Not(f) => {
                let f = f.to_fn();
                Box::new(move |c| !f(c))
            }

            Filters::Extra(filter) => filter.to_fn(),

            Filters::McGuffin(..) | Filters::Cake(..) => unreachable!(),
        }
    }
}

impl<E, C> ToFilter<E, C> for ()
where
    E: Clone,
    C: Clone + PartialEq,
{
    fn to_fn(self) -> FilterFn<E, C> {
        unimplemented!()
    }
}
