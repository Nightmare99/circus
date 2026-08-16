//! Task-board vocabulary shared between `circus` (the full multi-tenant
//! service) and `mini-circus` (the single-file CLI board). Only the concepts
//! that are genuinely identical between the two live here — status and
//! priority. Everything else (org/project structure, RBAC, task shape) is
//! deliberately not shared, since the two tools disagree on purpose.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A task's error on failing to parse from a string (CLI arg, JSON, etc).
#[derive(Debug)]
pub struct ParseEnumError {
    value: String,
    valid: &'static [&'static str],
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid value {:?}, expected one of: {}",
            self.value,
            self.valid.join(", ")
        )
    }
}

impl std::error::Error for ParseEnumError {}

macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $str:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type)]
        #[sqlx(type_name = "text", rename_all = "snake_case")]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $str),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = ParseEnumError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($str => Ok($name::$variant),)+
                    other => Err(ParseEnumError {
                        value: other.to_string(),
                        valid: &[$($str),+],
                    }),
                }
            }
        }
    };
}

string_enum! {
    /// A task's place in its board's workflow. Fixed on purpose — no custom
    /// columns — to keep both `circus` and `mini-circus` boards predictable.
    TaskStatus {
        Pending => "pending",
        InProgress => "in_progress",
        Blocked => "blocked",
        Completed => "completed",
    }
}

// Not derived: the shared `string_enum!` macro has no per-invocation way to
// mark a `#[default]` variant, and each enum's default differs.
#[allow(clippy::derivable_impls)]
impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

string_enum! {
    Priority {
        Low => "low",
        Medium => "medium",
        High => "high",
        Urgent => "urgent",
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_str() {
        for status in TaskStatus::ALL {
            assert_eq!(status.as_str().parse::<TaskStatus>().unwrap(), *status);
        }
        for priority in Priority::ALL {
            assert_eq!(priority.as_str().parse::<Priority>().unwrap(), *priority);
        }
    }

    #[test]
    fn rejects_unknown_values() {
        assert!("nonsense".parse::<TaskStatus>().is_err());
    }
}
