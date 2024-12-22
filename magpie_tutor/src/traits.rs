use std::fmt::{Debug, Display};

use poise::serenity_prelude::{CreateAllowedMentions, CreateMessage, MessageReference};

use crate::error;

/// Custom message extension
pub trait MessageCreateExt
where
    Self: Sized,
{
    /// Set this message to reply and not ping the author
    fn reply(self, reference: impl Into<MessageReference>) -> Self;
}

impl MessageCreateExt for CreateMessage {
    fn reply(self, reference: impl Into<MessageReference>) -> Self {
        self.reference_message(reference)
            .allowed_mentions(CreateAllowedMentions::new())
    }
}

/// Trait for converting value to it debug string representation
pub trait ToDebugString {
    /// Convert a value to debug string representation
    fn to_debug_string(&self) -> String
    where
        Self: Debug,
    {
        format!("{self:?}")
    }
}

/// Exrension for Option and Result where it is critical that they don't fails and if they do
/// immediately stop terminate.
pub trait Death<T> {
    /// Unwrap the data inside or terminate the program.
    fn unwrap_or_die(self, message: &str) -> T;
}

impl<T> Death<T> for Option<T> {
    fn unwrap_or_die(self, message: &str) -> T {
        if let Some(it) = self {
            return it;
        }
        error!("{}", message.red());
        error!("Critical error awaiting death...");
        std::process::exit(1)
    }
}

impl<T, E> Death<T> for Result<T, E>
where
    E: Debug,
{
    fn unwrap_or_die(self, message: &str) -> T {
        match self {
            Ok(it) => it,
            Err(err) => {
                error!("{}", message.red());
                error!("{}", format!("{err:?}").red());
                error!("{}", "Critical error awaiting death...".red());
                std::process::exit(1)
            }
        }
    }
}

macro_rules! color_fn {
    (
        $(
            $(#[$attr:meta])*
            fn $color:ident -> $ansi:literal;
        )*
    ) => {$(
        $(#[$attr])*
        fn $color(&self) -> String
        where
            Self: Display,
        {
            format!(concat!("\x1b[0;", stringify!($ansi), "m{}\x1b[0m"), self)
        }
    )*};
}

/// Allow value to be convert to a string with ansi color code.
pub trait Color {
    #[doc = r" Convert value to black text."]
    fn black(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(30), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to red text."]
    fn red(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(31), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to green text."]
    fn green(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(32), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to yellow text."]
    fn yellow(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(33), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to blue text."]
    fn blue(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(34), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to magenta text."]
    fn magenta(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(35), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to cyan text."]
    fn cyan(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(36), "m{}\x1b[0m"), self)
    }
    #[doc = r" Convert value to white text."]
    fn white(&self) -> String
    where
        Self: Display,
    {
        format!(concat!("\x1b[0;", stringify!(37), "m{}\x1b[0m"), self)
    }
}

impl<T: Display> Color for T {}
impl Color for str {}
