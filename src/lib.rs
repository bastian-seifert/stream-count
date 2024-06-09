use std::collections::HashSet;

pub mod distinct;
pub mod error;

// TODO: Replace with something more efficient?
pub type ElementSet<T> = HashSet<T>;
