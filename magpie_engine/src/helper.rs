//! Collection of helper for this crate

/// Tiny implmentation of bits flag to be a bit more flexiable and allow for extension.
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
        $v struct $name(pub $t);

        impl $name {
            $(
                $(#[$flag_attrs])*
                $v const $flag: $name = $name($value);
            )*
            /// Empty variant
            $v const EMPTY: $name = $name(0);

            /// Set all bit to true.
            pub fn all() -> $t {
                $(Self::$flag | )* 0
            }

            /// Check if this bit flag contain a bit.
            pub fn contains(&self, that: $t) -> bool {
                self.0 & that == that
            }

            /// Turn a bit on if the `toggle` is true and do nothing otherwise
            ///
            /// This is just sugar for `self | (bit * toggle)`
            pub fn set_if(self, bit: $name, toggle: bool) -> $name {
                self | (bit * toggle)
            }

            /// Get the actual flag inside the struct
            pub fn flags(self) -> $t {
                self.0
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
    };
}
pub(crate) use bitsflag;

/// Todo variant also remove unused variable warning
macro_rules! todo_unused {
    ($($var:ident),*) => {
        $(let _ = $var;)*
        todo!()
    };
}
pub(crate) use todo_unused;
