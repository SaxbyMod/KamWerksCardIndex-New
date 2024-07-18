use crate::helper::bitsflag;
use crate::Ptr;
use crate::SetCode;
use std::fmt::Debug;

/// Represent a card containing all the infomation on the cards.
///
/// You can add extra infomation using the [`Card::extra`] field and the generic `E`
#[derive(Debug, Clone)]
pub struct Card<E> {
    /// The set code that the card belong to.
    pub set: SetCode,

    /// The card name.
    pub name: String,
    /// The card description, note or favor text.
    pub description: String,
    /// Return the url to the card portrait
    pub portrait: String,

    /// The card rarity.
    pub rarity: Rarity,
    /// The card temple or archetype.
    ///
    /// Temple are a bit flag to tell which temple the card belong to. You should use the associated
    /// constant of [`Temple`] to set these bit flags. We use a [`u16`] instead of other crate like
    /// [`Bitflags`](https://docs.rs/bitflags/) so we can support more temple and make it easier to
    /// extend, if you need more than 16 temples, may god help you.
    pub temple: u16,

    /// The card attack or power.
    pub attack: isize,
    /// The card health.
    pub health: isize,

    /// The sigils or abilities on the card.
    pub sigils: Vec<Ptr<String>>,

    /// The card special attack, [`None`] if the card have no special attack
    ///
    /// Usually for card with variable attack or attack that are affected by traits. You would
    /// usually want [`Card::attack`] to return `0` if the card have a special attack.
    pub sp_atk: Option<SpAtk>,

    /// The card cost
    ///
    /// Cost contain a few component, one for each of the cost a card may have blood, bone, etc.
    /// The [`mox_count`](Costs::mox_count) component is available if the card can have multiple
    /// mox of each color.
    pub costs: Option<Costs>,
    /// The card traits
    ///
    /// Traits contain 2 components, the string component which is for uncommon or unique traits and
    /// the flags component for common traits. The flags iare just bit flags that multiple cards have
    /// like terrain, conductive, etc.
    pub traits: Option<Traits>,

    /// Related card or token
    ///
    /// Usuall for tokens, evolution, etc.
    pub related: Option<Vec<String>>,

    /// Extra
    pub extra: E,
}

impl Card<()> {
    /// Convert a empty, no extension card into something with a extension
    #[must_use]
    pub fn upgrade<T>(self) -> Card<T>
    where
        T: Default,
    {
        Card {
            extra: T::default(),

            set: self.set,

            name: self.name,
            description: self.description,

            portrait: self.portrait,

            rarity: self.rarity,
            temple: self.temple,

            attack: self.attack,
            health: self.health,

            sigils: self.sigils,

            sp_atk: self.sp_atk,

            costs: self.costs,

            traits: self.traits,
            related: self.related,
        }
    }
}

/// Rarities or tiers cards belong to
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rarity {
    /// Side deck rarity for card.
    ///
    /// This usually map to card that are restricted to the side deck or card that you can add a
    /// unlimited about of
    SIDE,
    /// Common rarity for card
    ///
    /// This usually map to card with the least amount of deck restriction
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
    /// This usually map to card that you can have only have 1 of this rarity per deck
    UNIQUE,
}

bitsflag! {
    /// Temples, binder or archetypes card belong to.
    pub struct Temple: u16 {
        /// The Beast or Leshy Temple.
        BEAST = 1;
        /// The Undead or Grimora Temple.
        UNDEAD = 1 << 1;
        /// The Tech or PO3 Temple.
        TECH = 1 << 2;
        /// The Magick or Magnificus Temple.
        MAGICK = 1 << 3;
        /// The Fool Temple from Augmented.
        FOOL = 1 << 4;
        /// The Artistry or Galliard Temple from Descryprion.
        ARTISTRY = 1 << 5;
    }
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
    /// Card that have power from the amount of card in your hand
    CARD,
}

bitsflag! {
    /// Bits flag for Mox, If you need more than these 4 colors you need to make you own mox type and
    /// extend it
    pub struct Mox: u16 {
        /// Red, Orange or Ruby Mox
        R = 1;
        /// Blue or Sapphire Mox
        G = 1 << 1;
        /// Green or Emerald Mox
        B = 1 << 2;
        /// Gray or Prism Mox
        Y = 1 << 3;
    }
}

/// Component for when card cost multiple of 1 Mox color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoxCount {
    /// The Red, Orange or Ruby component
    pub r: usize,
    /// The Green or Emerald component
    pub g: usize,
    /// The Blue or Sapphire component
    pub b: usize,
    /// The Gray, Prism component
    pub y: usize,
}

impl Default for MoxCount {
    fn default() -> Self {
        MoxCount {
            r: 1,
            g: 1,
            b: 1,
            y: 1,
        }
    }
}

/// Contain all the cost info.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Costs {
    /// Other case where the card are not free.
    /// Blood cost for the card.
    pub blood: isize,
    /// Bone cost for the card.
    pub bone: isize,
    /// Energy cost for the card.
    pub energy: isize,
    /// Mox bit flags for the card.
    pub mox: u16,
    /// Multiple Mox support for card.
    ///
    /// If the card only cost 1 Mox max you should not add this type.
    pub mox_count: Option<MoxCount>,
}

bitsflag! {
    /// Bit flags for a card trait
    pub struct TraitsFlag: u16 {
        /// If this card is conductive.
        CONDUCTIVE = 1;
        /// If this card is ban.
        BAN = 1 << 1;
        /// If this card is unsaccable or a terrain.
        TERRAIN = 1 << 2;
        /// If this card is hard or unhammerable
        HARD = 1 << 3;
    }
}

/// Store both flag based traits and string based traits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Traits {
    /// Traits that are not flags so they are [`String`].
    ///
    /// Uncommon trait are store in [`String`] form to reduce headache.
    pub traits: Option<Vec<String>>,
    /// Trait that are in bit flags form.
    ///
    /// Common traits are store using bit flags to save space.
    pub flags: u16,
}
