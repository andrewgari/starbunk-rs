//! Random question selection.

use crate::question::Question;
use rand::seq::SliceRandom;
use rand::Rng;

/// Pick up to `n` distinct questions from `pool` in random order using `rng`.
///
/// If the pool has `n` or fewer questions, all of them are used (shuffled).
/// Pass a seeded RNG (e.g. `StdRng::seed_from_u64`) for deterministic tests.
pub fn select_questions<R: Rng + ?Sized>(
    mut pool: Vec<Question>,
    n: usize,
    rng: &mut R,
) -> Vec<Question> {
    let take = n.min(pool.len());
    // Moves a uniform random sample of `take` elements to the front, in random order.
    pool.partial_shuffle(rng, take);
    pool.truncate(take);
    pool
}
