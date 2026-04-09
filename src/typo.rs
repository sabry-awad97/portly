/// Calculate Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string into another.
///
/// # Examples
///
/// ```
/// # use portly::typo::levenshtein_distance;
/// assert_eq!(levenshtein_distance("list", "list"), 0);
/// assert_eq!(levenshtein_distance("list", "lst"), 1);
/// assert_eq!(levenshtein_distance("kill", "kil"), 1);
/// ```
#[allow(dead_code)]
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }

    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for (i, a_char) in a.chars().enumerate() {
        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,    // deletion
                    matrix[i + 1][j] + 1,    // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Suggest a command for a typo.
///
/// Returns a "Did you mean 'X'?" suggestion if a close match is found.
/// Uses Levenshtein distance with a threshold of ≤2 to catch common typos.
/// Returns `None` for exact matches or if no close match is found.
///
/// # Examples
///
/// ```
/// # use portly::typo::suggest_command;
/// assert_eq!(suggest_command("lst"), Some("Did you mean 'list'?".to_string()));
/// assert_eq!(suggest_command("kil"), Some("Did you mean 'kill'?".to_string()));
/// assert_eq!(suggest_command("list"), None); // Exact match
/// assert_eq!(suggest_command("xyz"), None); // No close match
/// ```
#[allow(dead_code)]
pub fn suggest_command(typo: &str) -> Option<String> {
    let commands = vec!["list", "details", "kill", "clean", "ps", "watch", "config"];

    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for cmd in commands {
        let distance = levenshtein_distance(typo, cmd);
        if distance < best_distance && distance <= 2 && distance > 0 {
            best_distance = distance;
            best_match = Some(cmd);
        }
    }

    best_match.map(|cmd| format!("Did you mean '{cmd}'?"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance_identical() {
        assert_eq!(levenshtein_distance("list", "list"), 0);
        assert_eq!(levenshtein_distance("kill", "kill"), 0);
        assert_eq!(levenshtein_distance("watch", "watch"), 0);
    }

    #[test]
    fn test_levenshtein_distance_single_edit() {
        // Single deletion
        assert_eq!(levenshtein_distance("list", "lst"), 1);
        assert_eq!(levenshtein_distance("kill", "kil"), 1);

        // Single insertion
        assert_eq!(levenshtein_distance("lst", "list"), 1);
        assert_eq!(levenshtein_distance("kil", "kill"), 1);

        // Single substitution
        assert_eq!(levenshtein_distance("watch", "wach"), 1);
        assert_eq!(levenshtein_distance("list", "lust"), 1);
    }

    #[test]
    fn test_levenshtein_distance_two_edits() {
        assert_eq!(levenshtein_distance("details", "detals"), 1);
        assert_eq!(levenshtein_distance("config", "confg"), 1);
        assert_eq!(levenshtein_distance("clean", "clen"), 1);
    }

    #[test]
    fn test_levenshtein_distance_empty_strings() {
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("test", ""), 4);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_distance_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
        assert_eq!(levenshtein_distance("list", "watch"), 5);
    }

    #[test]
    fn test_suggest_command_close_match() {
        assert_eq!(
            suggest_command("lst"),
            Some("Did you mean 'list'?".to_string())
        );
        assert_eq!(
            suggest_command("kil"),
            Some("Did you mean 'kill'?".to_string())
        );
        assert_eq!(
            suggest_command("wach"),
            Some("Did you mean 'watch'?".to_string())
        );
        assert_eq!(
            suggest_command("detals"),
            Some("Did you mean 'details'?".to_string())
        );
        assert_eq!(
            suggest_command("clen"),
            Some("Did you mean 'clean'?".to_string())
        );
        assert_eq!(
            suggest_command("confg"),
            Some("Did you mean 'config'?".to_string())
        );
    }

    #[test]
    fn test_suggest_command_no_match() {
        // Too different (distance > 2)
        assert_eq!(suggest_command("xyz"), None);
        assert_eq!(suggest_command("abcdef"), None);
        assert_eq!(suggest_command("verylongcommand"), None);
    }

    #[test]
    fn test_suggest_command_exact_match() {
        // Exact matches should not return suggestions
        assert_eq!(suggest_command("list"), None);
        assert_eq!(suggest_command("kill"), None);
        assert_eq!(suggest_command("watch"), None);
        assert_eq!(suggest_command("details"), None);
        assert_eq!(suggest_command("clean"), None);
        assert_eq!(suggest_command("ps"), None);
        assert_eq!(suggest_command("config"), None);
    }

    #[test]
    fn test_suggest_command_distance_threshold() {
        // Distance = 1 (should suggest)
        assert_eq!(
            suggest_command("lis"),
            Some("Did you mean 'list'?".to_string())
        );

        // Distance = 2 (should suggest)
        assert_eq!(
            suggest_command("lst"),
            Some("Did you mean 'list'?".to_string())
        );

        // Distance > 2 (should not suggest)
        assert_eq!(suggest_command("abcd"), None);
    }

    #[test]
    fn test_suggest_command_picks_closest() {
        // "kil" is distance 1 from "kill", distance 3 from "list"
        // Should pick "kill"
        assert_eq!(
            suggest_command("kil"),
            Some("Did you mean 'kill'?".to_string())
        );

        // "wat" is distance 2 from "watch", distance 4 from "list"
        // Should pick "watch"
        assert_eq!(
            suggest_command("wat"),
            Some("Did you mean 'watch'?".to_string())
        );
    }

    #[test]
    fn test_suggest_command_case_sensitive() {
        // Commands are lowercase, so uppercase typos should still work
        // but with higher distance
        assert_eq!(
            suggest_command("List"),
            Some("Did you mean 'list'?".to_string())
        );
    }
}
