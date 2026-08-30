//! Deterministic, dependency-free random number generation.

use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple, fast xorshift64 random number generator.
///
/// We avoid the `rand` crate to prevent supply-chain bloat and ensure the
/// compiler can be easily cross-compiled or statically linked without issues.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Initializes a new RNG seeded by system time and process ID.
    /// This guarantees unique output across rapid sequential builds.
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let pid = process::id() as u64;

        let mut rng = Rng {
            state: now ^ (pid << 32) | 1,
        };
        rng.next(); // cycle once to mix the initial state
        rng
    }

    /// Generates the next pseudo-random 64-bit integer.
    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Returns `true` with the given percentage probability (0-100).
    pub fn coin(&mut self, chance: u64) -> bool {
        (self.next() % 100) < chance
    }

    /// Generates a random alphanumeric identifier of the specified length.
    /// Used for randomizing variable names and function signatures.
    pub fn fresh_ident(&mut self, len: usize) -> String {
        let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        (0..len)
            .map(|_| chars[(self.next() % chars.len() as u64) as usize] as char)
            .collect()
    }
}
