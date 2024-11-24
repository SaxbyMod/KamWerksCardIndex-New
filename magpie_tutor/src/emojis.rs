//! Emoji constant for the bot.

use magpie_engine::{SpAtk, TraitsFlag};

macro_rules! emoji_table {
    (pub mod $mod:ident {$($name:ident = $value:literal;)*}) => {
        pub mod $mod {
            #![allow(missing_docs)]
            $(pub const $name: &'static str = $value;)*
        }
    };
}

emoji_table! {
    pub mod number {
        NEGATE = "<:negative:1254856239108853810>";
        X = "<:x_:1254844718907654204>";

        // Number icon
        ZERO = "<:0_:1254844707817787415>";
        ONE = "<:1_:1254844708375625739>";
        TWO = "<:2_:1254844709612814578>";
        THREE = "<:3_:1254844710531629107>";
        FOUR = "<:4_:1254844711294992498>";
        FIVE = "<:5_:1254844711806701682>";
        SIX = "<:6_:1254844713043755038>";
        SEVEN = "<:7_:1254844781255725127>";
        EIGHT = "<:8_:1254844715568730224>";
        NINE = "<:9_:1254844782136786988>";
    }
}

emoji_table! {
    pub mod cost {
        // Cost icon
        BLOOD = "<:blood:1254812601452597350>";
        BONE = "<:bones:1254812629181137036>";
        ENERGY = "<:energy:1254812689608343674>";
        MAX = "<:overcharge:1254812739118043198>";
        LINK = "<:link:1292910794064789564>";
        GOLD = "<:gold:1292910650342768640>";

        // Mox color
        ORANGE = "<:ruby:1254812785985196134>";
        GREEN = "<:emerald:1254812654795624531>";
        BLUE = "<:sapphire:1254812816351952956>";
        GRAY = "<:prism:1254812757757268142>";
        BLACK = "<:onyx:1292911543159230746>";
        PLUS1 = "<:1_cost:1274031134442913872>";
    }
}

emoji_table! {
    pub mod icon {
        CONDUCTIVE = "<:conductive:1254849745869078569>";
        RARE = "<:rare:1254852219090767897>";
        BAN = "<:banned:1254841974129692764>";
        HARD = "<:unhammerable:1254848827970555975>";
        TERRAIN = "<:bloodless:1254848805032038591>";

        ANT = "<:ant:1254853395097976953>";
        BELL = "<:bell:1254854216875507722>";
        MOX = "<:mox:1254853396079312906>";
        CARD = "<:card_atk:1274031231255969885>";
        MIRROR = "<:mirror:1254853397908164682>";
    }
}

/// Allow value to turn into emoji(s).
pub trait ToEmoji {
    /// Turn a value to emoji(s).
    fn to_emoji(&self) -> String;
}

impl ToEmoji for SpAtk {
    fn to_emoji(&self) -> String {
        match self {
            SpAtk::MOX | SpAtk::GREEN_MOX => icon::MOX,
            SpAtk::MIRROR => icon::MIRROR,
            SpAtk::ANT => icon::ANT,
            SpAtk::BONE => todo!(),
            SpAtk::BELL => icon::BELL,
            SpAtk::CARD => icon::CARD,
        }
        .to_string()
    }
}

impl ToEmoji for TraitsFlag {
    fn to_emoji(&self) -> String {
        self.iter()
            .map(|v| match v {
                TraitsFlag::CONDUCTIVE => icon::CONDUCTIVE,
                TraitsFlag::BAN => icon::BAN,
                TraitsFlag::TERRAIN => icon::TERRAIN,
                TraitsFlag::HARD => icon::HARD,
                _ => unreachable!(),
            })
            .fold(String::new(), |a, b| a + b + " ") // this could def be faster but whatever
    }
}

macro_rules! impl_emoji {
    ($($type:tt)*) => {
        $(

        impl ToEmoji for $type {
            fn to_emoji(&self) -> String {
                let mut out = String::new();

                for d in self.to_string().chars() {
                    out.push_str(match d {
                        '-' => self::number::NEGATE,

                        '0' => number::ZERO,
                        '1' => number::ONE,
                        '2' => number::TWO,
                        '3' => number::THREE,
                        '4' => number::FOUR,
                        '5' => number::FIVE,
                        '6' => number::SIX,
                        '7' => number::SEVEN,
                        '8' => number::EIGHT,
                        '9' => number::NINE,
                        _ => unreachable!(),
                    });
                }

                out
            }
        }

        )*
    };
}

impl_emoji!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
);
