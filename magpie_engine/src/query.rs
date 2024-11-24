//! Implementation for querying card
//!
//! To query a card you first start with creating a [`QueryBuilder`] then build up your query using
//! [`Filters`] then finally calling [`QueryBuilder::query`] to obtain a [`Query`].
//!
//! # Examples
//!
//! ```
//! use magpie_engine::prelude::*;
//!
//! // Fetch the set to query
//! let imf = fetch_imf_set(
//!     "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/standard.json",
//!     SetCode::new("std").unwrap(),
//! ).unwrap();
//!
//! // Make the query
//! let query: QueryBuilder<(), (), ()> = QueryBuilder::with_filters(
//!     vec![&imf],
//!     vec![
//!         Filters::Attack(QueryOrder::GreaterEqual, 3),
//!         Filters::Health(QueryOrder::Less, 3 ),
//!         Filters::Sigil("Airborne".to_string()),
//!     ]
//! );
//!
//! // Finally compile and get the results
//! let result = query.query();
//! ```

use crate::{Attack, Card, Costs, Rarity, Set, SpAtk, Temple, Traits};
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::vec;

/// The result of a filters obtain by calling [`QueryBuilder::query`].
#[derive(Debug)]
pub struct Query<'a, E, C, F>
where
    E: Clone,
    C: Clone + PartialEq,
    F: ToFilter<E, C>,
{
    /// The results of this query.
    pub cards: Vec<&'a Card<E, C>>,
    /// The filters that produce this query.
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

/// Type alias for a filter function.
pub type FilterFn<E, C> = Box<dyn Fn(&Card<E, C>) -> bool>;

/// Query builder, it contain the set and is the main way to query cards.
///
/// You build up your query using [`add_filter`](QueryBuilder::add_filter) then call [`query`](QueryBuilder::query) to compile all the filters and
/// query for cards. You could also start out with all the filters using
/// [`with_filters`](QueryBuilder::with_filters) and then just call [`query`](QueryBuilder::query)
///
/// The final result will be a [`Query`] that contain what filters was used and the cards that
/// satisfied those filters.
///
/// # Examples
///
/// ```
/// use magpie_engine::prelude::*;
///
/// // Fetch the set to query
/// let imf = fetch_imf_set(
///     "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/standard.json",
///     SetCode::new("std").unwrap(),
/// ).unwrap();
///
/// // Make the query
/// let query: QueryBuilder<(), (), ()> = QueryBuilder::with_filters(
///     vec![&imf],
///     vec![Filters::Name("Squirrel".to_string())]
/// );
///
/// // Finally compile and get the results
/// let result = query.query();
/// ```
pub struct QueryBuilder<'a, E, C, F>
where
    E: Clone,
    C: Clone + PartialEq,
    F: ToFilter<E, C>,
{
    /// All the set that is use for this query.
    sets: Vec<&'a Set<E, C>>,

    filters: Vec<Filters<E, C, F>>,
    funcs: Vec<FilterFn<E, C>>,
}

impl<'a, E, C, F> QueryBuilder<'a, E, C, F>
where
    C: Clone + PartialEq + 'static,
    E: Clone + 'static,
    F: ToFilter<E, C> + 'static,
{
    /// Create a new empty [`QueryBuilder`] from a collection of set.
    ///
    /// This will give a [`QueryBuilder`] with no filters so you can add them using
    /// [`QueryBuilder::add_filter`] or [`QueryBuilder::add_filter_mut`] to do it inplace
    ///
    /// # Examples
    ///
    /// ```
    /// use magpie_engine::prelude::*;
    ///
    /// // Fetch the set to query
    /// let imf = fetch_imf_set(
    ///     "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/standard.json",
    ///     SetCode::new("std").unwrap(),
    /// ).unwrap();
    ///
    /// // Make the query
    /// let mut query: QueryBuilder<(), (), ()> = QueryBuilder::new(vec![&imf]);
    ///
    /// // Add a health filter
    /// query.add_filter_mut(Filters::Health(QueryOrder::Greater, 3));
    ///
    /// // Finally compile and query get the results
    /// let result = query.query();
    ///
    /// // Or alternatively you could use the builder pattern:
    ///
    /// let mut query: QueryBuilder<(), (), ()> =
    ///     QueryBuilder::new(vec![&imf])
    ///         .add_filter(Filters::Health(QueryOrder::Greater, 3));
    ///
    /// let result = query.query();
    /// ```
    #[must_use]
    pub fn new(sets: Vec<&'a Set<E, C>>) -> Self {
        QueryBuilder {
            sets,
            filters: vec![],
            funcs: vec![],
        }
    }

    /// Create a new [`QueryBuilder`] from a collection sets and filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use magpie_engine::prelude::*;
    ///
    /// // Fetch the set to query
    /// let imf = fetch_imf_set(
    ///     "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/standard.json",
    ///     SetCode::new("std").unwrap(),
    /// ).unwrap();
    ///
    /// // Make the query
    /// let query: QueryBuilder<(), (), ()> = QueryBuilder::with_filters(
    ///     vec![&imf],
    ///     vec![Filters::Attack(QueryOrder::Less, 3)]
    /// );
    ///
    /// // Finally compile and get the results
    /// let result = query.query();
    /// ```
    #[must_use]
    pub fn with_filters(sets: Vec<&'a Set<E, C>>, filters: Vec<Filters<E, C, F>>) -> Self {
        QueryBuilder {
            funcs: filters.clone().into_iter().map(|f| f.to_fn()).collect(),
            sets,
            filters,
        }
    }

    /// Add a new filter to this query.
    ///
    /// If you want to in place version use [`add_filter_mut`](QueryBuilder::add_filter_mut)
    /// instead
    #[must_use]
    pub fn add_filter(mut self, filter: Filters<E, C, F>) -> Self {
        self.filters.push(filter.clone());
        self.funcs.push(filter.to_fn());
        self
    }

    /// Add a new filter in place.
    ///
    /// If you want to use the builder pattern use [`add_filter`](QueryBuilder::add_filter) instead
    pub fn add_filter_mut(&mut self, filter: Filters<E, C, F>) {
        self.filters.push(filter.clone());
        self.funcs.push(filter.to_fn());
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

impl Display for QueryOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                QueryOrder::Greater => ">",
                QueryOrder::GreaterEqual => "≥",
                QueryOrder::Equal => "=",
                QueryOrder::LessEqual => "≤",
                QueryOrder::Less => "<",
            }
        )
    }
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
    Temple(Temple),
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

impl<E, C, F> Display for Filters<E, C, F>
where
    E: Clone + 'static,
    C: Clone + PartialEq + Display + 'static,
    F: ToFilter<E, C> + Display + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filters::Name(n) => write!(f, "name includes {n}"),
            Filters::Description(d) => write!(f, "description includes {d}"),
            Filters::Rarity(r) => write!(f, "is {r}"),
            Filters::Temple(t) => write!(f, "from the {t} temple"),
            Filters::Tribe(t) => match t {
                None => write!(f, "is tribeless"),
                Some(t) => write!(f, "is a {t}"),
            },
            Filters::Attack(o, a) => write!(f, "attack {o} {a}"),
            Filters::Health(o, a) => write!(f, "health {o} {a}"),
            Filters::Sigil(s) => write!(f, "have {s}"),
            Filters::SpAtk(a) => write!(f, "attack value is {a}"),
            Filters::StrAtk(s) => write!(f, "attack value is {s}"),
            Filters::Costs(c) => match c {
                None => write!(f, "is free"),
                Some(c) => write!(f, "cost {c}"),
            },
            Filters::Traits(t) => match t {
                None => write!(f, "is traitless"),
                Some(t) => write!(f, "is {t}"),
            },
            Filters::Or(a, b) => write!(f, "{a} or {b}"),
            Filters::Not(a) => write!(f, "not {a}"),
            Filters::Extra(e) => write!(f, "{e}"),
            Filters::McGuffin(..) | Filters::Cake(..) => unreachable!(),
        }
    }
}
