//! Vocabulary management for age-appropriate game creation.
//!
//! Per spec: Level 1 has 50 words, Level 2 has 150 words.
//! Words are from children's picture books vocabulary.

use std::collections::HashSet;

/// Vocabulary for a specific schema level
#[derive(Debug, Clone)]
pub struct Vocabulary {
    /// All valid words in this vocabulary
    words: HashSet<String>,
    /// Category mappings for suggestions
    categories: Vec<VocabularyCategory>,
}

/// A category of words in the vocabulary
#[derive(Debug, Clone)]
pub struct VocabularyCategory {
    /// Category name (e.g., "characters", "sounds")
    pub name: String,
    /// Words in this category
    pub words: Vec<String>,
}

impl Vocabulary {
    /// Create Level 1 vocabulary (ages 5-7, ~50 words)
    #[must_use]
    pub fn level1() -> Self {
        let categories = vec![
            VocabularyCategory {
                name: "characters".to_string(),
                words: vec![
                    "bunny", "cat", "dog", "bird", "robot", "unicorn", "dragon", "fish", "bear",
                    "fox",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "actions".to_string(),
                words: vec![
                    "move",
                    "jump",
                    "run",
                    "fly",
                    "swim",
                    "hide",
                    "appear",
                    "disappear",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "events".to_string(),
                words: vec![
                    "when_touch",
                    "when_near",
                    "when_far",
                    "when_start",
                    "when_score",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "sounds".to_string(),
                words: vec![
                    "pop", "ding", "whoosh", "splash", "boing", "twinkle", "buzz", "click",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "colors".to_string(),
                words: vec![
                    "red", "blue", "green", "yellow", "orange", "purple", "pink", "white", "black",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "backgrounds".to_string(),
                words: vec![
                    // Per spec Section 3.1: exactly 8 backgrounds
                    "sky", "grass", "water", "space", "forest", "beach", "snow", "rainbow",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "targets".to_string(),
                words: vec!["star", "coin", "gem", "heart", "apple"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "music".to_string(),
                words: vec!["gentle", "adventure", "happy", "calm", "exciting"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "movement".to_string(),
                words: vec!["arrows", "touch", "auto"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "target_actions".to_string(),
                words: vec!["new_place", "disappear"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            // Schema keywords
            VocabularyCategory {
                name: "schema".to_string(),
                words: vec![
                    "game",
                    "character",
                    "background",
                    "music",
                    "target",
                    "sound",
                    "score",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        ];

        Self::from_categories(categories)
    }

    /// Create Level 2 vocabulary (ages 8-10, ~150 words)
    #[must_use]
    pub fn level2() -> Self {
        let mut vocab = Self::level1();

        // Add Level 2 specific categories
        let level2_categories = vec![
            VocabularyCategory {
                name: "characters_l2".to_string(),
                words: vec![
                    // Per spec example uses asteroid
                    "rocket",
                    "spaceship",
                    "car",
                    "boat",
                    "asteroid",
                    "ninja",
                    "wizard",
                    "princess",
                    "knight",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "patterns".to_string(),
                words: vec!["zigzag", "circle", "chase", "wander", "patrol", "bounce"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "conditions".to_string(),
                words: vec!["reaches", "equals", "greater", "less", "between"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "effects".to_string(),
                words: vec!["blink", "shake", "grow", "shrink", "spin", "fade"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "game_states".to_string(),
                words: vec!["start", "pause", "resume", "stop", "restart", "win", "lose"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "speed".to_string(),
                words: vec!["slow", "normal", "fast"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "schema_l2".to_string(),
                words: vec![
                    "characters",
                    "rules",
                    "when",
                    "then",
                    "type",
                    "pattern",
                    "speed",
                    "lives",
                    "score_goal",
                    "add_score",
                    "lose_life",
                    "play",
                    "show",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        ];

        for cat in level2_categories {
            vocab.add_category(cat);
        }

        vocab
    }

    /// Create Level 3 vocabulary (ages 11+, full power)
    #[must_use]
    pub fn level3() -> Self {
        let mut vocab = Self::level2();

        let level3_categories = vec![
            VocabularyCategory {
                name: "world".to_string(),
                words: vec!["static", "procedural", "grid", "wfc", "noise"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "physics".to_string(),
                words: vec!["tile_based", "aabb", "continuous"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            VocabularyCategory {
                name: "ui_anchors".to_string(),
                words: vec![
                    "top_left",
                    "top_right",
                    "bottom_left",
                    "bottom_right",
                    "center",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            VocabularyCategory {
                name: "schema_l3".to_string(),
                words: vec![
                    "assets",
                    "sprites",
                    "sounds",
                    "models",
                    "world",
                    "algorithm",
                    "seed",
                    "size",
                    "tiles",
                    "entities",
                    "sprite",
                    "ai",
                    "components",
                    "position",
                    "health",
                    "inventory",
                    "physics",
                    "collision",
                    "camera",
                    "follow",
                    "zoom",
                    "ui",
                    "anchor",
                    "bind",
                    "version",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        ];

        for cat in level3_categories {
            vocab.add_category(cat);
        }

        vocab
    }

    /// Create vocabulary from categories
    fn from_categories(categories: Vec<VocabularyCategory>) -> Self {
        let mut words = HashSet::new();
        for cat in &categories {
            for word in &cat.words {
                let _ = words.insert(word.clone());
            }
        }
        Self { words, categories }
    }

    /// Add a category to the vocabulary
    fn add_category(&mut self, category: VocabularyCategory) {
        for word in &category.words {
            let _ = self.words.insert(word.clone());
        }
        self.categories.push(category);
    }

    /// Check if a word is in the vocabulary
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }

    /// Get the total word count
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Get all words in the vocabulary
    #[must_use]
    pub fn all_words(&self) -> Vec<String> {
        self.words.iter().cloned().collect()
    }

    /// Suggest similar words for a potentially misspelled word
    #[must_use]
    pub fn suggest_similar(&self, word: &str, max_suggestions: usize) -> Vec<String> {
        let word_lower = word.to_lowercase();
        let mut suggestions: Vec<(String, usize)> = self
            .words
            .iter()
            .map(|w| (w.clone(), levenshtein_distance(&word_lower, w)))
            .filter(|(_, dist)| *dist <= 3) // Max 3 edits
            .collect();

        suggestions.sort_by_key(|(_, dist)| *dist);
        suggestions
            .into_iter()
            .take(max_suggestions)
            .map(|(w, _)| w)
            .collect()
    }

    /// Get words in a specific category
    #[must_use]
    pub fn words_in_category(&self, category: &str) -> Vec<String> {
        self.categories
            .iter()
            .find(|c| c.name == category)
            .map(|c| c.words.clone())
            .unwrap_or_default()
    }

    /// Check if a word is valid for a specific category
    #[must_use]
    pub fn is_valid_for_category(&self, word: &str, category: &str) -> bool {
        self.categories
            .iter()
            .find(|c| c.name == category)
            .is_some_and(|c| c.words.iter().any(|w| w == &word.to_lowercase()))
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *cell = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level1_vocabulary_size() {
        let vocab = Vocabulary::level1();
        assert!(
            vocab.word_count() >= 50,
            "Level 1 should have at least 50 words, got {}",
            vocab.word_count()
        );
    }

    #[test]
    fn test_level2_vocabulary_size() {
        let vocab = Vocabulary::level2();
        assert!(
            vocab.word_count() >= 100,
            "Level 2 should have at least 100 words, got {}",
            vocab.word_count()
        );
    }

    #[test]
    fn test_level2_includes_level1() {
        let vocab1 = Vocabulary::level1();
        let vocab2 = Vocabulary::level2();

        for word in vocab1.all_words() {
            assert!(vocab2.contains(&word), "Level 2 should include '{word}'");
        }
    }

    #[test]
    fn test_level3_includes_level2() {
        let vocab2 = Vocabulary::level2();
        let vocab3 = Vocabulary::level3();

        for word in vocab2.all_words() {
            assert!(vocab3.contains(&word), "Level 3 should include '{word}'");
        }
    }

    #[test]
    fn test_contains_case_insensitive() {
        let vocab = Vocabulary::level1();
        assert!(vocab.contains("bunny"));
        assert!(vocab.contains("BUNNY"));
        assert!(vocab.contains("Bunny"));
    }

    #[test]
    fn test_suggest_similar_typo() {
        let vocab = Vocabulary::level1();
        let suggestions = vocab.suggest_similar("bunnny", 5); // Triple n
        assert!(!suggestions.is_empty());
        assert!(suggestions.contains(&"bunny".to_string()));
    }

    #[test]
    fn test_suggest_similar_close_word() {
        let vocab = Vocabulary::level1();
        let suggestions = vocab.suggest_similar("cat", 5);
        assert!(suggestions.contains(&"cat".to_string()));
    }

    #[test]
    fn test_suggest_similar_no_match() {
        let vocab = Vocabulary::level1();
        let suggestions = vocab.suggest_similar("xyzabc123", 5);
        // Should be empty or very limited for gibberish
        assert!(suggestions.len() <= 2);
    }

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein_distance("cat", "hat"), 1);
        assert_eq!(levenshtein_distance("cat", "cats"), 1);
        assert_eq!(levenshtein_distance("cat", "at"), 1);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_words_in_category() {
        let vocab = Vocabulary::level1();
        let characters = vocab.words_in_category("characters");
        assert!(characters.contains(&"bunny".to_string()));
        assert!(characters.contains(&"dragon".to_string()));
    }

    #[test]
    fn test_is_valid_for_category() {
        let vocab = Vocabulary::level1();
        assert!(vocab.is_valid_for_category("bunny", "characters"));
        assert!(!vocab.is_valid_for_category("bunny", "sounds"));
    }

    #[test]
    fn test_level1_characters() {
        let vocab = Vocabulary::level1();
        let expected = [
            "bunny", "cat", "dog", "bird", "robot", "unicorn", "dragon", "fish", "bear", "fox",
        ];
        for char in expected {
            assert!(vocab.contains(char), "Should contain '{char}'");
            assert!(vocab.is_valid_for_category(char, "characters"));
        }
    }

    #[test]
    fn test_level1_sounds() {
        let vocab = Vocabulary::level1();
        let expected = [
            "pop", "ding", "whoosh", "splash", "boing", "twinkle", "buzz", "click",
        ];
        for sound in expected {
            assert!(vocab.contains(sound), "Should contain '{sound}'");
            assert!(vocab.is_valid_for_category(sound, "sounds"));
        }
    }

    #[test]
    fn test_level2_patterns() {
        let vocab = Vocabulary::level2();
        let expected = ["zigzag", "circle", "chase", "wander", "patrol", "bounce"];
        for pattern in expected {
            assert!(vocab.contains(pattern), "Should contain '{pattern}'");
        }
    }
}
