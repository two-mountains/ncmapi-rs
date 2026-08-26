//! Pure protocol transformations used by the client boundary.
//!
//! These functions deliberately do not read the clock, random state, files, or
//! network. Keeping them as values-in/values-out makes the protocol executable
//! in deterministic tests.

use crate::{Error, Result};

/// A zero-based page request shared by collection endpoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Page {
    limit: u16,
    offset: u32,
}

impl Page {
    /// Creates a page with a non-zero item limit.
    pub fn new(limit: u16, offset: u32) -> Result<Self> {
        if limit == 0 {
            return Err(Error::InvalidInput {
                field: "page limit",
                reason: "must be greater than zero",
            });
        }
        Ok(Self { limit, offset })
    }

    /// Creates the first page with a non-zero item limit.
    pub fn first(limit: u16) -> Result<Self> {
        Self::new(limit, 0)
    }

    pub(crate) const fn limit(self) -> u16 {
        self.limit
    }

    pub(crate) const fn offset(self) -> u32 {
        self.offset
    }
}

pub(crate) fn request_id(timestamp_millis: u128, random_suffix: u16) -> String {
    format!("{timestamp_millis}_{random_suffix:04}")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{Page, request_id};

    #[test]
    fn request_id_is_deterministic_at_the_functional_core() {
        assert_eq!(request_id(1_724_000_000_000, 7), "1724000000000_0007");
    }

    #[test]
    fn pages_reject_a_zero_limit() {
        assert!(Page::first(0).is_err());
        assert_eq!(Page::new(20, 40).unwrap().offset(), 40);
    }

    proptest! {
        #[test]
        fn request_id_preserves_its_input_values(timestamp in any::<u64>(), suffix in 0_u16..1000) {
            let id = request_id(u128::from(timestamp), suffix);
            prop_assert_eq!(id, format!("{timestamp}_{suffix:04}"));
        }
    }
}
