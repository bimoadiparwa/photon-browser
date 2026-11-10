use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            pub fn next() -> Self {
                static COUNTER: AtomicU64 = AtomicU64::new(1);
                Self(COUNTER.fetch_add(1, Ordering::Relaxed))
            }
            pub fn new(v: u64) -> Self {
                Self(v)
            }
            pub fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

id_type!(TabId);
id_type!(ProfileId);
id_type!(SpaceId);
id_type!(PageId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_monotonic() {
        let a = TabId::next();
        let b = TabId::next();
        assert!(b.get() > a.get());
    }
}
