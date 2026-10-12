//! Random question selection.

use crate::question::Question;
use rand::Rng;

/// Pick up to `n` distinct questions from `pool` in random order using `rng`.
///
/// If the pool has `n` or fewer questions, all of them are used (shuffled).
/// Pass a seeded RNG (e.g. `StdRng::seed_from_u64`) for deterministic tests.
pub fn select_questions<R: Rng + ?Sized>(
    pool: Vec<Question>,
    n: usize,
    rng: &mut R,
) -> Vec<Question> {
    let _ = (pool, n, rng);
    todo!("ragtime: select_questions (PR 2)")
}
