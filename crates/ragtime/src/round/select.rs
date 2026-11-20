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
    // `partial_shuffle` returns the sampled elements as its first slice; their
    // position inside `pool` is an implementation detail, so use the return value.
    let (chosen, _rest) = pool.partial_shuffle(rng, take);
    chosen.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn q(id: i64) -> Question {
        let now = chrono::Utc::now();
        Question {
            id,
            question: format!("q{id}"),
            display_answer: "a".into(),
            acceptable_answers: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn single_pick_is_not_biased_to_the_ends() {
        let mut seen = std::collections::HashSet::new();
        for seed in 0..200 {
            let mut rng = StdRng::seed_from_u64(seed);
            let picked = select_questions((0..10).map(q).collect(), 1, &mut rng);
            assert_eq!(picked.len(), 1);
            seen.insert(picked[0].id);
        }
        assert!(seen.len() > 2, "picks must cover the pool, got {seen:?}");
    }
}
