//! The final leaderboard embed.

use serenity::all::CreateEmbed;

/// One leaderboard row. Mirrors `ragtime::round::tally::Standing` without
/// depending on that module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TallyRow {
    /// 1-based rank; tied players share a rank (1, 2, 2, 4).
    pub rank: usize,
    pub user: u64,
    /// Number of correct answers (1 point each).
    pub score: u32,
}

/// Discord embed description budget. Leave headroom under the hard
/// `EMBED_MAX_LENGTH` (6000, across the whole embed) and `description`'s own
/// 4096 cap for the title/footer and formatting overhead.
const DESCRIPTION_BUDGET: usize = 3800;

/// Build the final tally embed from already-ranked `standings` (see
/// `ragtime::round::tally::rank`, applied by the caller).
///
/// Rows are rendered in the given order. If the full leaderboard would not
/// fit in the embed, it is truncated with a trailing "and N more" note
/// rather than silently dropped or erroring.
pub fn tally_embed(standings: &[TallyRow]) -> CreateEmbed {
    let embed = CreateEmbed::new().title("Final tally");

    if standings.is_empty() {
        return embed.description("Nobody played.");
    }

    let mut lines = Vec::with_capacity(standings.len());
    let mut used = 0usize;
    let mut shown = 0usize;

    for row in standings {
        let line = format!(
            "**{}.** <@{}> — {} point{}",
            row.rank,
            row.user,
            row.score,
            if row.score == 1 { "" } else { "s" }
        );
        // +1 for the newline that will join this line to the next.
        let additional = line.len() + 1;
        if shown > 0 && used + additional > DESCRIPTION_BUDGET {
            break;
        }
        used += additional;
        lines.push(line);
        shown += 1;
    }

    let remaining = standings.len() - shown;
    if remaining > 0 {
        lines.push(format!("...and {remaining} more."));
    }

    embed.description(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embed_json(embed: &CreateEmbed) -> serde_json::Value {
        serde_json::to_value(embed).expect("CreateEmbed must serialize")
    }

    fn description(embed: &CreateEmbed) -> String {
        embed_json(embed)["description"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn title_is_final_tally() {
        let json = embed_json(&tally_embed(&[]));
        assert_eq!(json["title"], "Final tally");
    }

    #[test]
    fn empty_leaderboard_says_nobody_played() {
        let description = description(&tally_embed(&[]));
        assert!(description.to_lowercase().contains("nobody"));
    }

    #[test]
    fn rows_are_listed_in_given_order_with_rank_and_mention() {
        let standings = vec![
            TallyRow {
                rank: 1,
                user: 10,
                score: 5,
            },
            TallyRow {
                rank: 2,
                user: 20,
                score: 3,
            },
        ];
        let description = description(&tally_embed(&standings));
        let pos1 = description.find("<@10>").unwrap();
        let pos2 = description.find("<@20>").unwrap();
        assert!(pos1 < pos2);
        assert!(description.contains('1'));
        assert!(description.contains('2'));
    }

    #[test]
    fn ties_share_a_rank_and_both_rows_are_shown() {
        let standings = vec![
            TallyRow {
                rank: 1,
                user: 10,
                score: 5,
            },
            TallyRow {
                rank: 1,
                user: 20,
                score: 5,
            },
            TallyRow {
                rank: 3,
                user: 30,
                score: 1,
            },
        ];
        let description = description(&tally_embed(&standings));
        assert!(description.contains("**1.** <@10>"));
        assert!(description.contains("**1.** <@20>"));
        assert!(description.contains("**3.** <@30>"));
    }

    #[test]
    fn many_players_stay_within_the_embed_description_limit() {
        let standings: Vec<TallyRow> = (0..2000)
            .map(|i| TallyRow {
                rank: i as usize + 1,
                user: i as u64,
                score: 1,
            })
            .collect();
        let embed = tally_embed(&standings);
        let description = description(&embed);
        // Discord's hard cap on an embed description.
        assert!(description.chars().count() <= 4096);
    }

    #[test]
    fn truncation_adds_a_count_of_the_remaining_players() {
        let standings: Vec<TallyRow> = (0..2000)
            .map(|i| TallyRow {
                rank: i as usize + 1,
                user: i as u64,
                score: 1,
            })
            .collect();
        let description = description(&tally_embed(&standings));
        assert!(description.contains("more"));
    }

    #[test]
    fn a_small_leaderboard_is_not_truncated() {
        let standings = vec![
            TallyRow {
                rank: 1,
                user: 1,
                score: 1,
            },
            TallyRow {
                rank: 2,
                user: 2,
                score: 0,
            },
        ];
        let description = description(&tally_embed(&standings));
        assert!(!description.contains("more"));
    }

    #[test]
    fn zero_score_players_still_appear() {
        let standings = vec![TallyRow {
            rank: 1,
            user: 1,
            score: 0,
        }];
        let description = description(&tally_embed(&standings));
        assert!(description.contains("<@1>"));
        assert!(description.contains("0 point"));
    }
}
