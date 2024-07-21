//! Implementation for querying card
//!
//! To query a card you first start with creating a [`QueryBuilder`] then build up your query using
//! [`Filters`] then finally calling [`QueryBuilder::query`] to obtain a [`Query`]
use crate::{Card, Costs, Rarity, Set, SpAtk, Traits};
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::vec;

/// The query object containing your results and infomation about the filter that give you
/// the results.
#[derive(Debug)]
pub struct Query<'a, C, F>
where
    C: Clone,
    F: Filter<C>,
{
    /// The result of this query
    pub cards: Vec<&'a Card<C>>,
    /// The filter that produce this query
    pub filters: Vec<Filters<C, F>>,
}

impl<C, F> Display for Query<'_, C, F>
where
    C: Clone,
    F: Filter<C>,
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
pub type FilterFn<C> = Box<dyn Fn(&Card<C>) -> bool>;

/// Query builder, it contain the set and is the main way to query cards
///
/// You must first build up your query then lastly call `.query()` to compile all the condition and
/// start querying for cards
pub struct QueryBuilder<'a, C, F>
where
    C: Clone,
    F: Filter<C>,
{
    /// All the set that is use for this query
    pub sets: Vec<&'a Set<C>>,

    filters: Vec<Filters<C, F>>,
    funcs: Vec<FilterFn<C>>,
}

impl<'a, C, F> QueryBuilder<'a, C, F>
where
    C: Clone,
    F: Filter<C>,
{
    /// Create a new [`QueryBuilder`] from a collection of set.
    #[must_use]
    pub fn new(sets: Vec<&'a Set<C>>) -> Self {
        QueryBuilder {
            sets,
            filters: vec![],
            funcs: vec![],
        }
    }
    /// Add a new filter to this query.
    #[must_use]
    pub fn add_filter(mut self, filter: Filters<C, F>) -> Self {
        self.filters.push(filter.clone());
        self.funcs.push(filter.to_fn());
        self
    }

    /// Compile all the query and give you the result.
    #[must_use]
    pub fn query(self) -> Query<'a, C, F> {
        let filter = move |c: &Card<C>| self.funcs.iter().all(move |f| f(c));

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

/// [`Ordering`] extension for more ordering
#[derive(Debug, Clone)]
pub enum QueryOrder {
    /// Greater than another
    Greater,
    /// Greater than or equal to another
    GreaterEqual,
    /// Equal to another
    Equal,
    /// Less than or equal to another
    LessEqual,
    /// Less than another
    Less,
}

/// Enum for When query stuff
#[derive(Debug, Clone)]
pub enum Filters<C, F>
where
    F: Filter<C>,
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
    /// Filter for card tribes
    ///
    /// The value is the tribe or tribes to match against.
    Tribes(Option<String>),

    /// Filter for the card attack
    ///
    /// The first value is what what qualifier or comparasion to compare the attack against, the
    /// second is for equality (mainly for >=, <=) and lastly is the value to compare against
    Attack(QueryOrder, isize),
    /// Filter for the card attack
    ///
    /// The first value is what what qualifier or comparasion to compare the health against, the
    /// second is for equality (mainly for >=, <=) and lastly is the value to compare against
    Health(QueryOrder, isize),

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

    /// Extra filter you can add.
    Extra(F),

    #[doc(hidden)]
    McGuffin(Infallible, PhantomData<C>),
}

/// Trait for a Filter.
///
/// The generic is for the cards extension
pub trait Filter<C>: Clone {
    /// Turn the value into a filter that take a card and return a bool
    fn to_fn(self) -> FilterFn<C>;
}

/// Generate code to help with matching [`QueryOrder`]
#[macro_export]
macro_rules! query_order {
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

impl<C, F> Filter<C> for Filters<C, F>
where
    C: Clone,
    F: Filter<C>,
{
    fn to_fn(self) -> FilterFn<C> {
        match self {
            Filters::Name(name) => {
                Box::new(move |c| c.name.to_lowercase().contains(&name.to_lowercase()))
            }
            Filters::Description(desc) => {
                Box::new(move |c| c.description.to_lowercase().contains(&desc.to_lowercase()))
            }

            Filters::Rarity(rarity) => Box::new(move |c| c.rarity == rarity),
            Filters::Temple(temple) => Box::new(move |c| c.temple == temple),
            Filters::Tribes(tribes) => Box::new(move |c| match &c.tribes {
                Some(tr) if tribes.is_some() => tr
                    .to_lowercase()
                    .contains(&tribes.as_ref().unwrap().to_lowercase()),
                _ => c.tribes == tribes,
            }),
            Filters::Attack(ord, attack) => Box::new(move |c| query_order!(ord, c.attack, attack)),
            Filters::Health(ord, health) => Box::new(move |c| query_order!(ord, c.health, health)),
            Filters::Sigils(s) => {
                let lower = s.to_lowercase();
                Box::new(move |c| {
                    c.sigils
                        .iter()
                        .map(|s| s.to_lowercase())
                        .any(|s| s.eq(&lower))
                })
            }
            Filters::SpAtk(a) => Box::new(move |c| c.sp_atk == a),
            Filters::Costs(cost) => Box::new(move |c| c.costs == cost),
            Filters::Traits(traits) => Box::new(move |c| c.traits == traits),

            Filters::Extra(filter) => filter.to_fn(),

            Filters::McGuffin(..) => unreachable!(),
        }
    }
}

impl<C> Filter<C> for () {
    fn to_fn(self) -> FilterFn<C> {
        unimplemented!()
    }
}
