//! Provide function to fetch json and supported sets.

use isahc::ReadResponseExt;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Display;
use reqwest::blocking::Client;

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
/// Error that happen when calling [`fetch_json`].
#[derive(Debug)]
pub enum FetchError {
/// Error variant for handling Isahc errors.
IsahcError(isahc::Error),

/// Error variant for handling Serde JSON errors.
SerdeError(serde_json::Error),

/// Error variant for handling Request errors.
RequestError(reqwest::Error),

/// Error variant for handling errors during deserialization.
DeserializeError(serde_json::Error),

HttpError(reqwest::StatusCode),

}

impl Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::RequestError(e) => write!(f, "Request failed: {e}"),
            FetchError::DeserializeError(e) => write!(f, "JSON deserialization failed: {e}"),
            _ => write!(f, "An unknown error occurred"),
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

/// Fetches data from the Notion API.
///
/// # Arguments
/// * `url` - The URL to fetch data from.
/// * `api_key` - An optional API key for authorization.
///
/// # Returns
/// A `Result` containing the fetched data or an error.
pub fn fetch_from_notion<S>(
    url: &str,
    api_key: Option<&str>,
    payload: Option<serde_json::Value>,
) -> Result<S, FetchError>
where
    S: for<'de> Deserialize<'de>,
{
    let client = Client::new();
    let mut request = client.post(url);

    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
        request = request.header("Notion-Version", "2022-06-28");
    }

    if let Some(body) = payload {
        request = request.json(&body);
    }

    let response = request.send().map_err(FetchError::RequestError)?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            eprintln!(
                "Error: Unauthorized access. Check your API key and database permissions for {}",
                url
            );
        }
        return Err(FetchError::HttpError(response.status()));
    }    

    
    let data = response
        .json::<S>()
        .map_err(|err| FetchError::RequestError(err))?;

    Ok(data)
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
    // Error when Notion API Key is Missing
    MissingApiKey(String),
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
    DeserializeError(String),
}

impl Display for SetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetError::FetchError(e, url) => {
                write!(f, "cannot fetch set's data from `{url}` due to: {e}")
            }
            SetError::MissingApiKey(_) => write!(f, "missing API key"), // Match case for MissingApiKey
            SetError::UnknownTemple(e) => write!(f, "unknown scrybe: {e}"),
            SetError::UnknownRarity(e) => write!(f, "unknown rarity: {e}"),
            SetError::UnknownMoxColor(e) => write!(f, "unknown mox color: {e}"),
            SetError::UnknownCost(e) => write!(f, "unknown cost: {e}"),
            SetError::UnknownSpAtk(e) => write!(f, "unknown special attack: {e}"),
            SetError::InvalidCostFormat(e) => write!(f, "unknown cost format: {e}"),
            SetError::DeserializeError(e) => write!(f, "Missing results field: {e}"),

        }
    }
}

impl Error for SetError {}
