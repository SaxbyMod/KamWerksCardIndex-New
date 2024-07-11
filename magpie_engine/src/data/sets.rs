use crate::data::Card;
use crate::Ptr;
use std::collections::HashMap;
use std::fmt::Debug;

/// A 3 ascii characters set code for card and set
#[allow(dead_code)] // idk why it yelling the thing is use in the new
#[derive(Clone, Copy)]
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
    pub fn new(code: &str) -> Option<Self> {
        let bytes = code.as_bytes();
        (bytes.len() == 3 && bytes.is_ascii()).then(|| SetCode(bytes.try_into().unwrap()))
    }
}

impl Debug for SetCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
    }
}

/// Representation of a set containing info on the set and cards.
///
/// Sets are container for cards, they also carry a few other infomation like the sigils look up
/// table and pools. Pools are pre-sorted cards into categories.
pub struct Set {
    /// The set code for the deck.
    pub code: SetCode,
    /// The name of the set.
    pub name: String,
    /// The cards store in the set.
    ///
    /// These cards should be shared along with the card in the pools to save space on larger set.
    pub cards: Vec<Ptr<Card>>,
    /// The sigils description look up table for the set.
    pub sigils_description: HashMap<Ptr<String>, String>,
    /// The card pools for the set.
    ///
    /// These cards should be shared along with the card in the card list to save space on larger
    /// set.
    pub pools: HashMap<String, Vec<Ptr<Card>>>,
}
