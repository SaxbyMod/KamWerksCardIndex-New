use crate::Card;
use crate::UpgradeCard;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;

/// A 3 ascii characters set code for card and set
#[derive(Clone, Copy, Hash)]
pub struct SetCode([u8; 3]);

impl SetCode {
    /// Create a new [`SetCode`] using a 3 ascii characters.
    ///
    /// Character are ascii so it is guaranteed that every character is a single byte, because of
    /// this fact [`SetCode`] are just 3 bytes internally (`[u8; 3]`)
    ///
    /// # Example
    /// ```
    /// use magpie_engine::cards::SetCode;
    ///
    /// assert!(SetCode::new("ABC").is_some());
    /// assert!(SetCode::new("ABCD").is_none());
    /// ```
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // should never panic because we already check if the bytes are ascii
    pub fn new(code: &str) -> Option<Self> {
        let bytes = code.as_bytes();
        (bytes.len() == 3 && bytes.is_ascii()).then(|| SetCode(bytes.try_into().unwrap()))
    }

    /// Return the code as str.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn code(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }

    /// Return the bytes of the set code
    #[must_use]
    pub fn bytes(&self) -> [u8; 3] {
        self.0
    }
}

impl From<SetCode> for String {
    fn from(val: SetCode) -> Self {
        val.code().to_owned()
    }
}

impl Display for SetCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl Debug for SetCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.bytes())
    }
}

/// Representation of a set containing info on the set and cards.
///
/// Sets are container for cards, they also carry a few other infomation like the sigils look up
/// table and pools. Pools are pre-sorted cards into categories.
#[derive(Clone, Debug)]
pub struct Set<C> {
    /// The set code for the deck.
    pub code: SetCode,
    /// The name of the set.
    pub name: String,
    /// The cards store in the set.
    ///
    /// These cards should be shared along with the card in the pools to save space on larger set.
    pub cards: Vec<Card<C>>,
    /// The sigils description look up table for the set.
    ///
    /// Set are require to include **every** sigil in this look up table. So you can safely get
    /// value from this table without worrying about [`None`].
    pub sigils_description: HashMap<String, String>,
}

impl<T> Set<T> {
    /// Upgrade a set to another with different genric.
    pub fn upgrade<U>(self) -> Set<U>
    where
        Card<T>: UpgradeCard<U>,
    {
        Set {
            code: self.code,
            name: self.name,
            cards: self.cards.into_iter().map(UpgradeCard::upgrade).collect(),
            sigils_description: self.sigils_description,
        }
    }
}
