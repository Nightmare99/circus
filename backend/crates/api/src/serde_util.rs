use serde::{Deserialize, Deserializer};

/// Distinguishes "field omitted" (`None`) from "field explicitly set to
/// null" (`Some(None)`) for PATCH-style partial updates, e.g. clearing an
/// assignee vs. leaving it untouched. Pair with `#[serde(default,
/// deserialize_with = "double_option")]`.
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
