//! Final leaderboard ranking.

use super::UserId;

/// One leaderboard row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    /// 1-based rank. Tied players share a rank and the next rank is skipped
    /// ("standard competition" ranking: 1, 2, 2, 4).
    pub rank: usize,
    pub user: UserId,
    /// Number of correct answers (1 point each).
    pub score: u32,
}

/// Rank `scores` (given in join order) by score, descending.
///
/// Ties share a rank and keep their join order, so whoever joined first is
/// listed first within a tie. Every player appears, including those with 0 points.
pub fn rank(scores: &[(UserId, u32)]) -> Vec<Standing> {
    let mut sorted = scores.to_vec();
    // Stable sort keeps join order within a tie.
    sorted.sort_by_key(|&(_, score)| std::cmp::Reverse(score));

    let mut out: Vec<Standing> = Vec::with_capacity(sorted.len());
    for (position, (user, score)) in sorted.into_iter().enumerate() {
        let rank = match out.last() {
            Some(prev) if prev.score == score => prev.rank,
            _ => position + 1,
        };
        out.push(Standing { rank, user, score });
    }
    out
}
