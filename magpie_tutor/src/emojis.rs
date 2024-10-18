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
        NEGATE = "<:negative:1274030833413656618>";
        X = "<:x_:1274030809388417114>";

        // Number icon
        ZERO = "<:zero:1274030617230835722>";
        ONE = "<:one:1274030650445533184>";
        TWO = "<:two:1274030667751100516>";
        THREE = "<:three:1274030693063725076>";
        FOUR = "<:four:1274030709149012079>";
        FIVE = "<:four:1274030709149012079>";
        SIX = "<:six:1274030738219466835>";
        SEVEN = "<:seven:1274030748705357897>";
        EIGHT = "<:eight:1274030770326868029>";
        NINE = "<:nine:1274030795761385573>";
    }
}

emoji_table! {
    pub mod cost {
        // Cost icon
        BLOOD = "<:blood_cost:1274030851574726777>";
        BONE = "<:bone_cost:1274030863868231792>";
        ENERGY = "<:energy_cost:1274030884508667904>";
        MAX = "<:max_cost:1274030993518624843>";
        LINK = "<:link_cost:1274031021104431175>";
        GOLD = "<:gold_cost:1274031034979061762>";

        // Mox color
        ORANGE = "<:orange_cost:1274031063194407024>";
        GREEN = "<:green_cost:1274031076293214230>";
        BLUE = "<:blue_cost:1274031088594849824>";

        RED = "RED,";
        YELLOW = "YELLOW,";
        PURPLE = "PURPLE,";

        GRAY = "<:gray_cost:1274031125483884617>";
        BLACK = "<:black_cost:1274031104059248751>";

        // Shattered Mox color
        SHATTERED_ORANGE = "SHATTER ORANGE,";
        SHATTERED_GREEN = "SHATTER GREEN,";
        SHATTERED_BLUE = "SHATTER BLUE,";
        SHATTERED_GRAY = "SHATTER GRAY,";

        SHATTERED_RED = "SHATTER RED,";
        SHATTERED_YELLOW = "SHATTER YELLOW,";
        SHATTERED_PURPLE = "SHATTER PURPLE,";

        PLUS1 = "<:1_cost:1274031134442913872>";
    }
}

emoji_table! {
    pub mod icon {
        CONDUCTIVE = "<:conductive:1274031391788761212>";
        RARE = "<:rare:1274031362147352646>";
        BAN = "<:banned:1274031342350237879>";
        HARD = "<:hard:1274031567160741908>";
        TERRAIN = "<:terrain:1274031378543149208>";

        ANT = "<:ant_atk:1274031195142881330>";
        BELL = "<:bell_atk:1274031207583318139>";
        MOX = "<:green_atk:1274031281306468462>";
        CARD = "<:card_atk:1274031231255969885>";
        MIRROR = "<:mirror_atk:1274031328811290664>";
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
