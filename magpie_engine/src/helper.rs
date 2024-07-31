//! Collection of helper for this crate

/// Tiny implmentation of bits flag to be a bit more flexiable and allow for extension.
#[macro_export]
macro_rules! bitsflag {
    (
        $(#[$attr:meta])*
        $v:vis struct $name:ident: $t:ty
        {
            $(
                $(#[$flag_attrs:meta])*
                $flag:ident = $value:expr;
            )*
        }
    ) => {
        #[derive(PartialEq, Eq, Clone, Debug)]
        $(#[$attr])*
        $v struct $name($t);

        impl $name {
            $(
                $(#[$flag_attrs])*
                $v const $flag: $name = $name($value);
            )*
            /// Empty variant
            $v const EMPTY: $name = $name(0);

            /// Set all bit to true.
            #[must_use]
            pub fn all() -> $t {
                $(Self::$flag | )* 0
            }

            /// Check if this bit flag contain a bit.
            #[must_use]
            pub fn contains(&self, other: impl Into<$t>) -> bool {
                let other = other.into();
                self.0 & other == other
            }

            /// Turn a bit on if the `toggle` is true and do nothing otherwise
            ///
            /// This is just sugar for `self | (bit * toggle)`
            #[must_use]
            pub fn set_if(self, bit: $name, toggle: bool) -> $name {
                self | (bit * toggle)
            }

            /// Get the actual flag inside the struct
            pub fn flags(&self) -> impl Iterator<Item = &'static $name> {
                let flag = vec![$(self.contains($name::$flag),)*];
                [
                    $($name::$flag,)*
                ].iter().zip(flag).filter_map(|(v,f)| f.then(|| v))
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output {
                $name(self.0 | rhs.0)
            }
        }

        impl std::ops::BitOr<$t> for $name {
            type Output = $t;
            fn bitor(self, rhs: $t) -> Self::Output {
                self.0 | rhs
            }
        }

        impl std::ops::BitOr<$name> for $t {
            type Output = $t;
            fn bitor(self, rhs: $name) -> Self::Output {
                self | rhs.0
            }
        }

        impl std::ops::BitOrAssign<$name> for $t {
            fn bitor_assign(&mut self, rhs: $name) {
                *self = rhs | *self
            }
        }

        impl std::ops::BitOrAssign<$name> for $name {
            fn bitor_assign(&mut self, rhs: $name) {
                self.0 = self.0 | rhs.0;
            }
        }

        impl std::ops::Mul<bool> for $name {
            type Output = $name;
            fn mul(self, rhs: bool) -> Self::Output {
                match rhs {
                    true => self,
                    false => $name(0)
                }
            }
        }

        impl From<$t> for $name {
            fn from(value: $t) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $t {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::EMPTY
            }
        }
    };
}

/// Generate a [`UpgradeCard`] implementation for upgrading to the same type.
#[macro_export]
macro_rules! self_upgrade {
    ($ty:ty) => {
        impl $crate::UpgradeCard<$ty> for Card<$ty> {
            fn upgrade(self) -> Card<$ty> {
                self
            }
        }
    };
}
