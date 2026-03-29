use strsim::normalized_levenshtein;

pub fn is_duplicate_lesson(existing: &str, new: &str, threshold: f64) -> bool {
    if existing.is_empty() && new.is_empty() {
        return true;
    }
    normalized_levenshtein(existing, new) >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_lessons_are_duplicates() {
        assert!(is_duplicate_lesson(
            "use Result for parsing",
            "use Result for parsing",
            0.8
        ));
    }

    #[test]
    fn similar_lessons_are_duplicates() {
        assert!(is_duplicate_lesson(
            "Always use Result handling for parse operations",
            "Always use Result handling for parse operations on user input",
            0.75,
        ));
    }

    #[test]
    fn different_lessons_are_not_duplicates() {
        assert!(!is_duplicate_lesson(
            "use Result for parsing",
            "check array bounds before indexing",
            0.8,
        ));
    }

    #[test]
    fn empty_strings() {
        assert!(is_duplicate_lesson("", "", 0.8));
    }
}
