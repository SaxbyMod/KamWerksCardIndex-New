//! Provide function to fetch json and supported sets.

use isahc::ReadResponseExt;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Display;

mod aug;
mod cti;
mod desc;
mod imf;

pub use aug::*;
pub use cti::*;
pub use desc::*;
pub use imf::*;

use crate::Set;

/// Type alias for set fetch output.
pub type SetResult<E, C> = Result<Set<E, C>, SetError>;

/// Error that happen when calling [`fetch_json`].
#[derive(Debug)]
pub enum FetchError {
    /// [`isahc`](https://docs.rs/isahc) error or error that happen when trying to fetch the json data.
    IsahcError(isahc::Error),
    /// [`serde`] error or error that happen when parsing the json data to the target type..
    SerdeError(serde_json::Error),
}

impl Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::IsahcError(e) => write!(f, "unable to fetch json due to: {e}"),
            FetchError::SerdeError(e) => write!(f, "unable to parse json due to: {e}"),
        }
    }
}

impl Error for FetchError {}

/// Just a wrapper around [`isahc`](https://docs.rs/isahc) to fetch and parse json.
/// # Example
/// ```rust
/// use magpie_engine::fetch::fetch_json;
/// use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Res {
///     url: String
/// }
///
/// let res: Res = fetch_json("https://httpbin.org/get").unwrap();
///
/// assert_eq!(res.url, "https://httpbin.org/get");
/// ```
pub fn fetch_json<S>(url: &str) -> Result<S, FetchError>
where
    S: for<'de> Deserialize<'de>,
{
    isahc::get(url)
        .map_err(FetchError::IsahcError)?
        .json()
        .map_err(FetchError::SerdeError)
}

/// Fetch google sheet json using [`opensheet`](https://github.com/benborgers/opensheet).
pub fn fetch_google_sheet<S>(id: &str, tab_name: &str) -> Result<S, FetchError>
where
    S: for<'de> Deserialize<'de>,
{
    fetch_json(format!("https://opensheet.elk.sh/{id}/{tab_name}").as_str())
}

/// Error when fetching any set.
#[derive(Debug)]
pub enum SetError {
    /// Error when trying to [`fetch_json`] cards.
    FetchError(FetchError, String),
    /// Unknown Temple or Scrybe.
    UnknownTemple(String),
    /// Unknown rarity.
    UnknownRarity(String),
    /// Unknown Mox color.
    UnknownMoxColor(String),
    /// Unknown cost type.
    UnknownCost(String),
    /// Unknown special attack type
    UnknownSpAtk(String),
    /// Invalid cost format
    InvalidCostFormat(String),
}

impl Display for SetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetError::FetchError(e, url) => {
                write!(f, "cannot fetch set's data from `{url}` due to: {e}")
            }
            SetError::UnknownTemple(e) => write!(f, "unknown scrybe: {e}"),
            SetError::UnknownRarity(e) => write!(f, "unknown rarity: {e}"),
            SetError::UnknownMoxColor(e) => write!(f, "unknown mox color: {e}"),
            SetError::UnknownCost(e) => write!(f, "unknown cost: {e}"),
            SetError::UnknownSpAtk(e) => write!(f, "unknown special attack: {e}"),
            SetError::InvalidCostFormat(e) => write!(f, "unknown cost format: {e}"),
        }
    }
}

impl Error for SetError {}
