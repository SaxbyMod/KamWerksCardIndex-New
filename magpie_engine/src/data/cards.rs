use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;

use bitflags::bitflags;

use crate::SetCode;

macro_rules! card {
    ($($(#[$attr:meta])* $f:ident: $ty:ty,)*) => {
        /// Represent a card containing all the infomation on the cards.
        ///
        /// You can add extra infomation using the [`Card::extra`] field and the generic `E`
        #[derive(Debug, Clone)]
        pub struct Card<E, C>
        where
            E: Clone,
            C: Clone + PartialEq,
        {
            /// The card cost
            ///
            /// Cost contain a few component, one for each of the cost a card may have blood, bone, etc.
            /// The [`mox_count`](Costs::mox_count) component is available if the card can have multiple
            /// mox of each color.
            ///
            /// Free card can have this as [`None`]
            pub costs: Option<Costs<C>>,

            /// Extra
            pub extra: E,

            $(
                $(#[$attr])*
                pub $f: $ty,
            )*
        }

        /// Macro to help with generating [`UpgradeCard`] implementation
        #[macro_export]
        macro_rules! upgrade_card {
            (extra: $extra:expr, costs: $costs:expr, ..$card:expr) => {
                Card {
                    extra: $extra,
                    costs: $card.costs.map(|c| Costs {
                        extra: $costs(c.clone()),

                        blood: c.blood,
                        bone: c.bone,
                        energy: c.energy,
                        mox: c.mox,
                        mox_count: c.mox_count,

                    }),

                    $($f: $card.$f,)*
                }
            };
        }
    };
}

card! {
    /// The set code that the card belong to.
    set: SetCode,

    /// The card name.
    name: String,
    /// The card description, note or favor text.
    description: String,
    /// The url to the card portrait
    portrait: String,

    /// The card rarity.
    rarity: Rarity,
    /// The card temple or archetype.
    ///
    /// Temple are a bit flag to tell which temple the card belong to. You should use the associated
    /// constant of [`Temple`] to set these bit flags. We use a [`u16`] instead of other crate like
    /// [`Bitflags`](https://docs.rs/bitflags/) so we can support more temple and make it easier to
    /// extend, if you need more than 16 temples, may god help you.
    temple: Temple,
    /// The card tribes.
    tribes: Option<String>,

    /// The card attack or power.
    attack: Attack,
    /// The card health.
    health: isize,

    /// The sigils or abilities on the card.
    sigils: Vec<String>,

    /// The card traits
    ///
    /// Traits contain 2 components, the string component which is for uncommon or unique traits and
    /// the flags component for common traits. The flags iare just bit flags that multiple cards have
    /// like terrain, conductive, etc.
    ///
    /// Card with no traits can have this as [`None`]
    traits: Option<Traits>,

    /// Related card or token
    ///
    /// Usuall for tokens, evolution, etc.
    related: Vec<String>,

}

impl<T, U> Hash for Card<T, U>
where
    T: Clone,
    U: Clone + PartialEq,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.set.hash(state);
    }
}

/// Trait for a card to be upgradeable to another card with different generic.
pub trait UpgradeCard<E, U>
where
    E: Clone,
    U: Clone + PartialEq,
{
    /// Convert this card to another version with different generic
    #[must_use]
    fn upgrade(self) -> Card<E, U>;
}

impl<T, U> UpgradeCard<T, U> for Card<(), ()>
where
    T: Default + Clone,
    U: Default + Clone + PartialEq,
{
    fn upgrade(self) -> Card<T, U> {
        upgrade_card! {
            extra: T::default(),
            costs: |_| U::default(),
            ..self
        }
    }
}

/// Rarities or tiers cards belong to
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rarity {
    /// Side deck rarity for card.
    ///
    /// This usually map to card that are restricted to the side deck or card that you can add a
    /// unlimited about of.
    SIDE,
    /// Common rarity for card.
    ///
    /// This usually map to card with the least amount of deck restriction.
    COMMON,
    //// Uncommon rarity for card.
    ///
    /// This usually map to card with a bit more restriction than [`COMMON`](Rarity::COMMON) but
    /// you can still have more than 1 copy.
    UNCOMMON,
    /// Rare rarity for card.
    ///
    /// This usually map to card that you can only have 1 copy per deck.
    RARE,
    /// Unique rarity for card.
    ///
    /// This usually map to card that you can have only have 1 of this rarity per deck.
    UNIQUE,
}

impl Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rarity::SIDE => "side",
                Rarity::COMMON => "common",
                Rarity::UNCOMMON => "uncommon",
                Rarity::RARE => "rare",
                Rarity::UNIQUE => "unique",
            }
        )
    }
}

bitflags! {
    /// Temples, binder or archetypes card belong to.
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    pub struct Temple: u16 {
        /// The Beast or Leshy Temple.
        const BEAST = 1;
        /// The Undead or Grimora Temple.
        const UNDEAD = 1 << 1;
        /// The Tech or PO3 Temple.
        const TECH = 1 << 2;
        /// The Magick or Magnificus Temple.
        const MAGICK = 1 << 3;
        /// The Fool Temple from Augmented.
        const FOOL = 1 << 4;
        /// The Artistry or Galliard Temple from Descryprion.
        const ARTISTRY = 1 << 5;
    }
}

impl Display for Temple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];

        let flags = [
            (Temple::BEAST, "beast"),
            (Temple::UNDEAD, "undead"),
            (Temple::TECH, "tech"),
            (Temple::FOOL, "fool"),
            (Temple::ARTISTRY, "artistry"),
        ];

        for (f, v) in flags {
            if self.contains(f) {
                out.push(v);
            }
        }

        write!(f, "{}", out.join(" or "))
    }
}

/// Enum for the diffrent attack type.
#[derive(Debug, Clone)]
pub enum Attack {
    /// Numeric attack value.
    Num(isize),
    /// Common predefined special attack.
    SpAtk(SpAtk),
    /// String special attack.
    Str(String),
}

/// Special attack for cards.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SpAtk {
    /// Card that gain power from Mox.
    MOX,
    /// Card that gain power from Green Mox.
    GREEN_MOX,
    /// Card that have power of the opposing card.
    MIRROR,
    /// Card that have power of the amount of ant cards.
    ANT,
    /// Card that have power from the amount of bone token you have.
    BONE,
    /// Card that have power from it position to the bell.
    BELL,
    /// Card that have power from the amount of card in your hand.
    CARD,
}

impl Display for SpAtk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SpAtk::MOX => "mox",
                SpAtk::GREEN_MOX => "green mox",
                SpAtk::MIRROR => "mirror",
                SpAtk::ANT => "ant",
                SpAtk::BONE => "bone",
                SpAtk::BELL => "bell",
                SpAtk::CARD => "card",
            }
        )
    }
}

bitflags! {
    /// Bits flag for Moxes.
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    pub struct Mox: u16 {
        /// Orange or Ruby Mox.
        const O = 1;
        /// Green or Emerald Mox.
        const G = 1 << 1;
        /// Blue or Sapphire Mox.
        const B = 1 << 2;
        /// Gray or Prism Mox
        const Y = 1 << 3;

        /// Red or Garnet Mox
        const R = 1<< 4;
        /// Yellow or Topaz Mox
        const E = 1 << 5;
        /// Purple or Amethyst Mox
        const P = 1 << 6;


        /// Black or Onyx Mox.
        const K = 1 << 7;
        /// Plus 1 indicator for Descryption
        const P1 = 1<< 8;
    }
}

/// Component for when card cost multiple of 1 Mox color.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MoxCount {
    /// The Orange component.
    pub o: usize,
    /// The Green component.
    pub g: usize,
    /// The Blue component.
    pub b: usize,
    /// The Gray component.
    pub y: usize,

    /// The Red component
    pub r: usize,
    /// The Yellow component
    pub e: usize,
    /// The Purple component
    pub p: usize,

    /// The Black component.
    pub k: usize,
}

/// Contain all the cost info.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Costs<E> {
    /// Other case where the card are not free.
    /// Blood cost for the card.
    pub blood: isize,
    /// Bone cost for the card.
    pub bone: isize,
    /// Energy cost for the card.
    pub energy: isize,
    /// Mox bit flags for the card.
    pub mox: Mox,
    /// Multiple Mox support for card.
    ///
    /// If the card only cost 1 Mox max you should not add this type.
    pub mox_count: Option<MoxCount>,

    /// Extra Field for cost extension.
    pub extra: E,
}

impl<E> Display for Costs<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];

        if self.blood != 0 {
            out.push(format!("{} blood", self.blood));
        }
        if self.bone != 0 {
            out.push(format!("{} bone", self.bone));
        }
        if self.energy != 0 {
            out.push(format!("{} energy", self.energy));
        }
        if self.blood != 0 {
            out.push(format!("{} blood", self.blood));
        }

        if self.mox.contains(Mox::O) {
            out.push(format!(
                "{}orange",
                if let Some(ref m) = self.mox_count {
                    m.o
                } else {
                    1
                }
            ));
        }
        if self.mox.contains(Mox::G) {
            out.push(format!(
                "{}green",
                if let Some(ref m) = self.mox_count {
                    m.g
                } else {
                    1
                }
            ));
        }
        if self.mox.contains(Mox::B) {
            out.push(format!(
                "{}blue",
                if let Some(ref m) = self.mox_count {
                    m.b
                } else {
                    1
                }
            ));
        }
        if self.mox.contains(Mox::Y) {
            out.push(format!(
                "{}gray",
                if let Some(ref m) = self.mox_count {
                    m.y
                } else {
                    1
                }
            ));
        }
        if self.mox.contains(Mox::K) {
            out.push(format!(
                "{}black",
                if let Some(ref m) = self.mox_count {
                    m.k
                } else {
                    1
                }
            ));
        }

        out.push(format!("{}", self.extra));

        write!(f, "{}", out.join(" and "))
    }
}

bitflags! {
    /// Bit flags for a card trait.
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    pub struct TraitsFlag: u16 {
        /// If this card is conductive.
        const CONDUCTIVE = 1;
        /// If this card is ban.
        const BAN = 1 << 1;
        /// If this card is unsaccable or a terrain.
        const TERRAIN = 1 << 2;
        /// If this card is hard or unhammerable.
        const HARD = 1 << 3;
    }
}

impl Display for TraitsFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];

        let flags = [
            (TraitsFlag::CONDUCTIVE, "conductive"),
            (TraitsFlag::BAN, "banned"),
            (TraitsFlag::TERRAIN, "terrain"),
            (TraitsFlag::HARD, "hard"),
        ];

        for (f, v) in flags {
            if self.contains(f) {
                out.push(v);
            }
        }

        write!(f, "{}", out.join(" and "))
    }
}

/// Store both flag based traits and string based traits.
#[derive(Clone, Debug, PartialEq)]
pub struct Traits {
    /// Traits that are not flags so they are [`String`].
    ///
    /// Uncommon trait are store in [`String`] form to reduce headache.
    pub strings: Option<Vec<String>>,
    /// Trait that are in bit flags form.
    ///
    /// Common traits are store using bit flags to save space.
    pub flags: TraitsFlag,
}

impl Display for Traits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if let Some(ref strs) = self.strings {
                strs.join(" and ") + " and "
            } else {
                String::new()
            },
            self.flags
        )
    }
}

impl Traits {
    /// Create a new Traits with flags component and empty [`Traits::strings`].
    ///
    /// # Examples
    /// ```
    /// use magpie_engine::prelude::*;
    ///
    /// let test = Traits::with_flags(TraitsFlag::BAN | TraitsFlag::HARD);
    ///
    /// assert_eq!(
    ///     test,
    ///     Traits {
    ///         strings: None,
    ///         flags: TraitsFlag::BAN | TraitsFlag::HARD,
    ///     }
    /// );
    /// ```
    #[must_use]
    pub fn with_flags(flags: TraitsFlag) -> Self {
        Traits {
            strings: None,
            flags,
        }
    }

    /// Create a new Traits with string component and empty [`Traits::flags`]
    ///
    /// # Examples
    /// ```
    /// use magpie_engine::prelude::*;
    ///
    /// let test = Traits::with_string(
    ///     vec!["A Good Song Never Die".to_string()]
    /// );
    ///
    /// assert_eq!(
    ///     test,
    ///     Traits {
    ///         strings: Some(
    ///             vec!["A Good Song Never Die".to_string()]
    ///         ),
    ///         flags: TraitsFlag::empty()
    ///     }
    /// );
    /// ```
    #[must_use]
    pub fn with_string(traits: Vec<String>) -> Self {
        Traits {
            strings: Some(traits),
            flags: TraitsFlag::empty(),
        }
    }
}
