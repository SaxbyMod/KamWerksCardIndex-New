//! Collection of helper for this crate.

use bitflags::Flags;

/// Generate a [`UpgradeCard`](crate::UpgradeCard) implementation for upgrading to the same type.
#[macro_export]
macro_rules! self_upgrade {
    ($ty1:ty, $ty2:ty) => {
        impl $crate::UpgradeCard<$ty1, $ty2> for Card<$ty1, $ty2> {
            fn upgrade(self) -> Card<$ty1, $ty2> {
                self
            }
        }
    };
}

pub trait FlagsExt: Flags {
    /// Just like `set` except it also return the bitflags
    fn set_if(mut self, what: Self, value: bool) -> Self {
        self.set(what, value);
        self
    }
}

impl<T> FlagsExt for T where T: Flags {}
