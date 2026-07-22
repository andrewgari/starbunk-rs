use rand::Rng;
use regex::Regex;

/// Resolves response template placeholders against the triggering message content.
///
/// Supported placeholders:
/// - `{start}` — an excerpt of the triggering message, wrapped in `***...***`. The excerpt is
///   built by taking the **first 3 whitespace-separated words**, joining them with a single space,
///   and then slicing to at most **15 Unicode characters** (char-boundary safe). A `...` suffix is
///   appended only when the message has more than 3 words OR when the joined text was truncated to
///   15 chars. Matches the JS `response-resolver` word-based extraction with conditional ellipsis.
/// - `{random:min-max:chars}` — repeat `chars` a random N times, N ∈ [min, max], capped at 1000
/// - `{swap_message:word1:word2}` — swap occurrences of word1↔word2 (word-boundary, case-preserving)
///
/// If the template contains no placeholders it is returned unchanged.
/// Unknown or malformed placeholders are passed through verbatim.
pub fn resolve_template(template: &str, msg_content: &str) -> String {
    let mut result = template.to_string();
    result = resolve_start(&result, msg_content);
    result = resolve_random(&result);
    result = resolve_swap_message(&result, msg_content);
    result
}

fn resolve_start(template: &str, msg: &str) -> String {
    const MARKER: &str = "{start}";
    let words: Vec<&str> = msg.split_whitespace().collect();
    let word_count = words.len();

    // Take at most the first 3 words and join with spaces
    let joined: String = words.into_iter().take(3).collect::<Vec<_>>().join(" ");

    // Slice to at most 15 Unicode characters (char-boundary safe)
    let char_count = joined.chars().count();
    let (clipped, was_truncated) = if char_count <= 15 {
        (joined, false)
    } else {
        let truncated: String = joined.chars().take(15).collect();
        (truncated, true)
    };

    // Append "..." only when there are more than 3 words OR the text was truncated to 15 chars
    let excerpt = if word_count > 3 || was_truncated {
        format!("{}...", clipped)
    } else {
        clipped
    };

    template.replace(MARKER, &format!("***{}***", excerpt))
}

fn resolve_random(template: &str) -> String {
    let mut result = template.to_string();
    const MARKER: &str = "{random:";
    let mut offset = 0;
    while let Some(rel) = result[offset..].find(MARKER) {
        let start = offset + rel;
        let content_start = start + MARKER.len();

        let Some(close_offset) = result[content_start..].find('}') else {
            // No closing brace — leave as-is, skip past this marker
            offset = content_start;
            continue;
        };
        let placeholder_end = content_start + close_offset + 1;
        let content = result[content_start..content_start + close_offset].to_string();

        let Some(colon) = content.find(':') else {
            offset = placeholder_end;
            continue;
        };
        let range_str = &content[..colon];
        let chars_str = content[colon + 1..].to_string();

        let Some(dash) = range_str.find('-') else {
            offset = placeholder_end;
            continue;
        };
        let min = match range_str[..dash].parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                offset = placeholder_end;
                continue;
            }
        };
        let max = match range_str[dash + 1..].parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                offset = placeholder_end;
                continue;
            }
        };

        let effective_max = max.min(1000);
        let effective_min = min.min(effective_max);
        let count = if effective_min >= effective_max {
            effective_min
        } else {
            rand::thread_rng().gen_range(effective_min..=effective_max)
        };

        let replacement = chars_str.repeat(count);
        result = format!(
            "{}{}{}",
            &result[..start],
            replacement,
            &result[placeholder_end..]
        );
        offset = start + replacement.len();
    }
    result
}

fn resolve_swap_message(template: &str, msg: &str) -> String {
    let mut result = template.to_string();
    const MARKER: &str = "{swap_message:";
    let mut offset = 0;
    while let Some(rel) = result[offset..].find(MARKER) {
        let start = offset + rel;
        let content_start = start + MARKER.len();

        let Some(close_offset) = result[content_start..].find('}') else {
            offset = content_start;
            continue;
        };
        let placeholder_end = content_start + close_offset + 1;
        let content = result[content_start..content_start + close_offset].to_string();

        let Some(colon) = content.find(':') else {
            offset = placeholder_end;
            continue;
        };
        let word1 = content[..colon].to_string();
        let word2 = content[colon + 1..].to_string();

        if word1.is_empty() || word2.is_empty() {
            offset = placeholder_end;
            continue;
        }

        let swapped = swap_words(msg, &word1, &word2);
        result = format!(
            "{}{}{}",
            &result[..start],
            swapped,
            &result[placeholder_end..]
        );
        offset = start + swapped.len();
    }
    result
}

/// Swaps all whole-word occurrences of word1↔word2 in `msg` (case-insensitive detection,
/// case-preserving output). Uses a dynamic regex — static LazyLock is not possible here
/// because the pattern depends on runtime-provided words.
fn swap_words(msg: &str, word1: &str, word2: &str) -> String {
    let pattern = format!(
        r"(?i)\b({}|{})\b",
        regex::escape(word1),
        regex::escape(word2)
    );
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return msg.to_string(),
    };
    re.replace_all(msg, |caps: &regex::Captures| {
        let matched = &caps[0];
        let replacement = if matched.to_lowercase() == word1.to_lowercase() {
            word2
        } else {
            word1
        };
        apply_case(matched, replacement)
    })
    .into_owned()
}

/// Applies the case pattern of `source` to `replacement`.
/// Supports: all-uppercase ("CHECK" → "CZECH"), title-case ("Check" → "Czech"),
/// and lowercase passthrough ("check" → "czech").
fn apply_case(source: &str, replacement: &str) -> String {
    let alphabetic: Vec<char> = source.chars().filter(|c| c.is_alphabetic()).collect();
    let all_upper = !alphabetic.is_empty() && alphabetic.iter().all(|c| c.is_uppercase());
    let first_upper = source
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false);

    if all_upper {
        replacement.to_uppercase()
    } else if first_upper {
        let mut chars = replacement.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                let upper: String = first.to_uppercase().collect();
                upper + &chars.as_str().to_lowercase()
            }
        }
    } else {
        replacement.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_template;

    // -----------------------------------------------------------------------
    // {start} — word-based excerpt of the triggering message (JS-aligned)
    // -----------------------------------------------------------------------

    /// 1 word, ≤3 words, ≤15 chars — no ellipsis
    #[test]
    fn start_short_message_no_truncation() {
        assert_eq!(
            resolve_template("{start} said hi", "Hi"),
            "***Hi*** said hi"
        );
    }

    /// 2 words, ≤3 words, ≤15 chars — no ellipsis
    #[test]
    fn start_two_words_no_ellipsis() {
        assert_eq!(
            resolve_template("{start}-- sorry", "Hey there"),
            "***Hey there***-- sorry"
        );
    }

    /// Exactly 3 words that fit in 15 chars — no ellipsis
    #[test]
    fn start_exactly_3_words_fits_no_ellipsis() {
        // "Hello world goo" is exactly 15 chars, 3 words
        assert_eq!(
            resolve_template("{start}!", "Hello world goo"),
            "***Hello world goo***!"
        );
    }

    /// Exactly 3 words but joined text > 15 chars — ellipsis because truncated
    #[test]
    fn start_3_words_over_15_chars_truncated_with_ellipsis() {
        // "Superlongword1 Superlongword2 X" — first 3 words joined = "Superlongword1 Superlongword2 X"
        // "Superlongword1" is 14 chars, space is char 15 → first 15 chars = "Superlongword1 "
        // truncated, so ellipsis added
        let result = resolve_template("{start}", "Superlongword1 Superlongword2 X");
        assert_eq!(result, "***Superlongword1 ...***");
    }

    /// More than 3 words, first 3 fit in 15 chars — ellipsis because >3 words
    #[test]
    fn start_more_than_3_words_ellipsis_appended() {
        // "What are you doing" — first 3 words = "What are you" (12 chars, fits in 15)
        // but there are >3 words so ellipsis is added
        assert_eq!(
            resolve_template("{start}-- Oh, sorry... go ahead", "What are you doing"),
            "***What are you...***-- Oh, sorry... go ahead"
        );
    }

    /// Classic production case: long message, >3 words, first 3 words truncated at 15 chars
    #[test]
    fn start_message_over_15_chars_truncated_with_ellipsis() {
        // "Hello world this is a test" — first 3 words = "Hello world this" (16 chars)
        // truncated to 15 = "Hello world thi", plus ellipsis (both >3 words AND truncated)
        assert_eq!(
            resolve_template("{start}-- go ahead", "Hello world this is a test"),
            "***Hello world thi...***-- go ahead"
        );
    }

    #[test]
    fn start_empty_message_produces_empty_excerpt() {
        let result = resolve_template("{start} said something", "");
        assert_eq!(result, "****** said something");
    }

    #[test]
    fn start_unicode_message_truncates_at_character_not_byte_boundary() {
        // Each emoji is 4 bytes but 1 char — one very long "word" of 16 emoji.
        // First 3 words = just that one word, but it exceeds 15 chars so truncation fires.
        let msg = "🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉"; // 16 emoji, 1 word
        let result = resolve_template("{start}", msg);
        assert!(result.starts_with("***"));
        assert!(result.contains("...***"));
        // Must not contain the 16th emoji
        let inner = result.trim_start_matches("***").trim_end_matches("***");
        let excerpt = inner.trim_end_matches("...");
        assert_eq!(excerpt.chars().count(), 15);
    }

    #[test]
    fn start_multiple_in_template_each_replaced() {
        // Two {start} in one template — both expand to the same excerpt
        let result = resolve_template("{start} and {start}", "Hi");
        assert_eq!(result, "***Hi*** and ***Hi***");
    }

    // -----------------------------------------------------------------------
    // {random:min-max:char} — repeated character sequence
    // -----------------------------------------------------------------------

    #[test]
    fn random_single_char_length_in_range() {
        for _ in 0..50 {
            let result = resolve_template("sh{random:2-5:e}sh", "");
            // Must be "sh" + N*"e" + "sh" where N in [2,5]
            assert!(result.starts_with("sh"));
            assert!(result.ends_with("sh"));
            let middle = &result[2..result.len() - 2];
            assert!(
                middle.len() >= 2 && middle.len() <= 5,
                "expected 2–5 'e's, got: {middle:?}"
            );
            assert!(middle.chars().all(|c| c == 'e'));
        }
    }

    #[test]
    fn random_min_equals_max_is_deterministic() {
        // {random:3-3:e} must always produce exactly "eee"
        for _ in 0..20 {
            assert_eq!(resolve_template("{random:3-3:e}", ""), "eee");
        }
    }

    #[test]
    fn random_zero_to_zero_produces_empty_string() {
        assert_eq!(resolve_template("{random:0-0:e}", ""), "");
    }

    #[test]
    fn random_one_to_one_produces_single_char() {
        assert_eq!(resolve_template("{random:1-1:x}", ""), "x");
    }

    #[test]
    fn random_multi_char_string_repeated() {
        // Production: "{random:2-20:Mister } Beeeeeeeeeast"
        for _ in 0..20 {
            let result = resolve_template("{random:2-20:Mister } Beeeeeeeeeast", "");
            assert!(result.ends_with(" Beeeeeeeeeast"));
            let prefix = result.trim_end_matches(" Beeeeeeeeeast");
            // Each repetition is "Mister " (7 chars), so total prefix len ∈ [14, 140]
            assert!(prefix.len() >= 14 && prefix.len() <= 140);
        }
    }

    #[test]
    fn random_capped_at_1000_repetitions() {
        // Even if range says 0-9999, output must not exceed 1000 repetitions
        let result = resolve_template("{random:0-9999:e}", "");
        assert!(result.len() <= 1000);
    }

    #[test]
    fn random_with_emoji_in_surrounding_text() {
        // Production: "sh{random:2-20:e}sh 😤"
        for _ in 0..20 {
            let result = resolve_template("sh{random:2-20:e}sh 😤", "");
            assert!(result.starts_with("sh"));
            assert!(result.ends_with("sh 😤"));
        }
    }

    // -----------------------------------------------------------------------
    // {swap_message:word1:word2} — word swap in original message
    // -----------------------------------------------------------------------

    #[test]
    fn swap_message_basic_word_swap() {
        let result = resolve_template("{swap_message:check:czech}", "check the list");
        assert_eq!(result, "czech the list");
    }

    #[test]
    fn swap_message_reverse_direction() {
        let result = resolve_template("{swap_message:check:czech}", "czech republic");
        assert_eq!(result, "check republic");
    }

    #[test]
    fn swap_message_both_words_present_swaps_both() {
        let result = resolve_template("{swap_message:check:czech}", "check czech check");
        assert_eq!(result, "czech check czech");
    }

    #[test]
    fn swap_message_preserves_capitalisation() {
        let result = resolve_template("{swap_message:check:czech}", "Check the list");
        assert_eq!(result, "Czech the list");
    }

    #[test]
    fn swap_message_all_caps_preserved() {
        let result = resolve_template("{swap_message:check:czech}", "CHECK IT OUT");
        assert_eq!(result, "CZECH IT OUT");
    }

    #[test]
    fn swap_message_no_match_returns_original() {
        let result = resolve_template("{swap_message:check:czech}", "nothing to swap here");
        assert_eq!(result, "nothing to swap here");
    }

    #[test]
    fn swap_message_is_case_insensitive_detection() {
        // Should find "Check" even though the pattern is "check"
        let result = resolve_template("{swap_message:check:czech}", "Check this out");
        assert!(result.to_lowercase().contains("czech"));
    }

    // -----------------------------------------------------------------------
    // No placeholder — passthrough
    // -----------------------------------------------------------------------

    #[test]
    fn no_placeholder_returns_template_unchanged() {
        let template = "Always bring a :banana: to a party!";
        assert_eq!(resolve_template(template, "I like banana"), template);
    }

    #[test]
    fn empty_template_returns_empty_string() {
        assert_eq!(resolve_template("", "hello"), "");
    }

    // -----------------------------------------------------------------------
    // Multiple placeholders in one response
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_placeholders_both_resolved() {
        // Hypothetical: "{start} sh{random:2-5:e}sh"
        let result = resolve_template("{start} sh{random:2-5:e}sh", "Hello world this");
        assert!(result.starts_with("***Hello world thi"));
        assert!(result.contains("sh") && result.ends_with("sh"));
    }

    // -----------------------------------------------------------------------
    // Unknown / unrecognised placeholders — passthrough
    // -----------------------------------------------------------------------

    #[test]
    fn unknown_placeholder_returned_verbatim() {
        // A placeholder we don't recognise must not be stripped or expanded
        let template = "Hello {unknown_thing} world";
        assert_eq!(
            resolve_template(template, "anything"),
            "Hello {unknown_thing} world"
        );
    }

    #[test]
    fn partial_placeholder_not_expanded() {
        // A brace that is never closed is not a placeholder
        let template = "Hello {unclosed world";
        assert_eq!(resolve_template(template, "msg"), "Hello {unclosed world");
    }

    // -----------------------------------------------------------------------
    // {swap_message} — word-boundary behaviour
    // -----------------------------------------------------------------------

    #[test]
    fn swap_message_does_not_match_partial_word() {
        // "checkout" contains "check" as a prefix, but a word-boundary swap must
        // not fire because "check" is not a standalone word here.
        let result = resolve_template("{swap_message:check:czech}", "checkout the list");
        assert_eq!(result, "checkout the list");
    }

    #[test]
    fn swap_message_matches_word_followed_by_punctuation() {
        // "check," — the comma follows "check" but word-boundary still holds
        let result = resolve_template("{swap_message:check:czech}", "check, mate");
        assert_eq!(result, "czech, mate");
    }
}
