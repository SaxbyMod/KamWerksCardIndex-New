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
        NEGATE = "<:negative:1262601007947583529>";
        X = "<:x_:1262600986816544923>";

        // Number icon
        ZERO = "<:num_zero:1262600927001579550>";
        ONE = "<:num_one:1262600952213667840>";
        TWO = "<:num_two:1262600930080067594>";
        THREE = "<:num_three:1262600963164999710>";
        FOUR = "<:num_four:1262600921129554005>";
        FIVE = "<:num_five:1262600922471731342>";
        SIX = "<:num_six:1262600923205603400>";
        SEVEN = "<:num_seven:1262600924287864872>";
        EIGHT = "<:num_eight:1262600925248356394>";
        NINE = "<:num_nine:1262600926066249728>";
    }
}

emoji_table! {
    pub mod cost {
        // Cost icon
        BLOOD = "<:imf_blood:1262601160121122837>";
        BONE = "<:imf_bone:1262601002180284437>";
        ENERGY = "<:imf_energy:1262601154479521835>";
        MAX = "<:imf_energy:1262601154479521835>";
        LINK = "<:imf_energy:1262601154479521835>";
        GOLD = "<:imf_energy:1262601154479521835>";

        // Mox color
        RED = "<:red:1262601156560158770>";
        GREEN = "<:green:1262600994018033786>";
        BLUE = "<:blue:1262600990654468177>";
        GRAY = "<:gray:1264359833587417224>";
    }
}

emoji_table! {
    pub mod icon {
        CONDUCTIVE = "<:conductive:1262600992789237862>";
        RARE = "<:rare:1262601011839635540>";
        BAN = "<:ban:1262600988930478132>";
        HARD = "<:hard:1262601015325098015>";
        TERRAIN = "<:terrain:1262601157382115358>";

        ANT = "<:ant_atk:1262600987752005744>";
        BELL = "<:bell_atk:1262600989685579787>";
        MOX = "<:mox_atk:1262601155473838082>";
        CARD = "<:hand_atk:1262601157868785666>";
        MIRROR = "<:mirror_atk:1262601005082742844>";
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
        self.flags()
            .map(|v| match *v {
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
