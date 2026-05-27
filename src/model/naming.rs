//! Naming convention parser for Elite Dangerous body hierarchy.
//!
//! Elite names bodies by appending a hierarchical suffix to the system name:
//!   - Stars: "A", "B", "C" (single uppercase) or "ABC" (barycenter groups)
//!   - Planets: "1", "2" (under unnamed star) or "A 1", "B 3" (under named star)
//!   - Moons: "A 2 a" (lowercase after planet number)
//!   - Sub-moons: "A 2 e a" (another lowercase after moon letter)
//!   - Belt clusters: "A Belt Cluster 1" (special pattern)
//!
//! The parser converts a short name (system prefix already stripped) into a
//! hierarchical sort key that produces correct depth-first ordering.

/// A parsed position in the body hierarchy.
///
/// Each level alternates between uppercase star designations and numeric
/// planet indices, with lowercase moon suffixes. The sort key ensures
/// correct ordering when bodies are sorted lexicographically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyPosition {
    /// Display-friendly short name (e.g., "A 2 a").
    pub short_name: String,
    /// Hierarchy depth: 0 = star/root, 1 = planet, 2 = moon, 3 = sub-moon.
    pub depth: u32,
    /// Sortable key for ordering within the tree.
    /// Format: segments separated by '.' — stars as uppercase, planets as
    /// zero-padded numbers, moons as lowercase.
    pub sort_key: String,
    /// Whether this is a belt cluster (special naming pattern).
    pub is_belt_cluster: bool,
}

/// Parse a body's short name (system prefix stripped) into a hierarchy position.
///
/// The short name should be the result of stripping the system name from the
/// body name, e.g., "Prudgeou VD-B e1 A 2 a" → "A 2 a".
///
/// An empty string indicates the body IS the primary star.
pub fn parse_body_name(short_name: &str) -> BodyPosition {
    let trimmed = short_name.trim();

    // Empty name = the primary star (body name matches system name)
    if trimmed.is_empty() {
        return BodyPosition {
            short_name: String::new(),
            depth: 0,
            sort_key: String::new(),
            is_belt_cluster: false,
        };
    }

    // Belt cluster: "A Belt Cluster 1", "B A Belt Cluster 2", "Belt Cluster 3"
    if let Some(pos) = trimmed.find("Belt Cluster") {
        let prefix = trimmed[..pos].trim();
        let suffix = trimmed[pos + "Belt Cluster".len()..].trim();
        let cluster_num: u32 = suffix.parse().unwrap_or(0);

        let sort_key = if prefix.is_empty() {
            format!("belt.{:03}", cluster_num)
        } else {
            let prefix_key = build_sort_key_from_tokens(&tokenize(prefix));
            format!("{}.belt.{:03}", prefix_key, cluster_num)
        };

        return BodyPosition {
            short_name: trimmed.to_string(),
            depth: if prefix.is_empty() { 0 } else { count_token_depth(&tokenize(prefix)) },
            sort_key,
            is_belt_cluster: true,
        };
    }

    // Normal body: tokenize and build hierarchy
    let tokens = tokenize(trimmed);
    let depth = count_token_depth(&tokens);
    let sort_key = build_sort_key_from_tokens(&tokens);

    BodyPosition {
        short_name: trimmed.to_string(),
        depth,
        sort_key,
        is_belt_cluster: false,
    }
}

/// Token types in a body name.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NameToken {
    /// Uppercase star designation: "A", "B", "ABC", "CDE"
    Star(String),
    /// Numeric planet index: 1, 2, 10, ...
    Planet(u32),
    /// Lowercase moon letter: "a", "b", ...
    Moon(String),
}

/// Tokenize a body short name into a sequence of typed tokens.
///
/// Rules:
/// - Uppercase alphabetic segments → Star
/// - Numeric segments → Planet
/// - Lowercase single letter segments → Moon
fn tokenize(name: &str) -> Vec<NameToken> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    let mut tokens = Vec::new();

    for part in parts {
        if let Ok(num) = part.parse::<u32>() {
            tokens.push(NameToken::Planet(num));
        } else if part.chars().all(|c| c.is_ascii_uppercase()) {
            tokens.push(NameToken::Star(part.to_string()));
        } else if part.chars().all(|c| c.is_ascii_lowercase()) && part.len() == 1 {
            tokens.push(NameToken::Moon(part.to_string()));
        } else {
            // Unrecognized token — treat as star designation (fallback)
            tokens.push(NameToken::Star(part.to_string()));
        }
    }

    tokens
}

/// Count the hierarchy depth from tokens.
///
/// - Star designations are depth 0 (root level)
/// - First planet number is depth 1
/// - First moon letter is depth 2
/// - Second moon letter (sub-moon) is depth 3
/// - etc.
fn count_token_depth(tokens: &[NameToken]) -> u32 {
    let mut depth = 0u32;
    for token in tokens {
        match token {
            NameToken::Star(_) => {
                // Stars don't add depth beyond 0, but this is the star's position
            }
            NameToken::Planet(_) => {
                depth = depth.max(1);
            }
            NameToken::Moon(_) => {
                depth += 1;
                depth = depth.max(2);
            }
        }
    }
    depth
}

/// Build a sortable key from tokens.
///
/// Stars sort alphabetically, planets by zero-padded number, moons by letter.
fn build_sort_key_from_tokens(tokens: &[NameToken]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for token in tokens {
        match token {
            NameToken::Star(s) => parts.push(s.clone()),
            NameToken::Planet(n) => parts.push(format!("{:03}", n)),
            NameToken::Moon(m) => parts.push(m.clone()),
        }
    }
    parts.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Primary star (empty name)
    // ---------------------------------------------------------------

    #[test]
    fn empty_name_is_primary_star() {
        let pos = parse_body_name("");
        assert_eq!(pos.depth, 0);
        assert_eq!(pos.sort_key, "");
        assert!(!pos.is_belt_cluster);
    }

    // ---------------------------------------------------------------
    // Star designations
    // ---------------------------------------------------------------

    #[test]
    fn single_star_designation() {
        let pos = parse_body_name("A");
        assert_eq!(pos.depth, 0);
        assert_eq!(pos.sort_key, "A");
        assert!(!pos.is_belt_cluster);
    }

    #[test]
    fn multi_star_barycenter() {
        let pos = parse_body_name("CDE");
        assert_eq!(pos.depth, 0);
        assert_eq!(pos.sort_key, "CDE");
    }

    #[test]
    fn star_designations_sort_correctly() {
        let a = parse_body_name("A");
        let b = parse_body_name("B");
        let c = parse_body_name("C");
        let cde = parse_body_name("CDE");
        assert!(a.sort_key < b.sort_key);
        assert!(b.sort_key < c.sort_key);
        assert!(c.sort_key < cde.sort_key);
    }

    // ---------------------------------------------------------------
    // Planets under single star (no star prefix)
    // ---------------------------------------------------------------

    #[test]
    fn planet_under_unnamed_star() {
        let pos = parse_body_name("1");
        assert_eq!(pos.depth, 1);
        assert_eq!(pos.sort_key, "001");
    }

    #[test]
    fn planets_sort_numerically() {
        let p1 = parse_body_name("1");
        let p2 = parse_body_name("2");
        let p10 = parse_body_name("10");
        assert!(p1.sort_key < p2.sort_key);
        assert!(p2.sort_key < p10.sort_key);
    }

    // ---------------------------------------------------------------
    // Planets under named star
    // ---------------------------------------------------------------

    #[test]
    fn planet_under_named_star() {
        let pos = parse_body_name("A 1");
        assert_eq!(pos.depth, 1);
        assert_eq!(pos.sort_key, "A.001");
    }

    #[test]
    fn planet_under_barycenter() {
        let pos = parse_body_name("CDE 2");
        assert_eq!(pos.depth, 1);
        assert_eq!(pos.sort_key, "CDE.002");
    }

    // ---------------------------------------------------------------
    // Moons
    // ---------------------------------------------------------------

    #[test]
    fn moon_under_unnamed_planet() {
        let pos = parse_body_name("5 c");
        assert_eq!(pos.depth, 2);
        assert_eq!(pos.sort_key, "005.c");
    }

    #[test]
    fn moon_under_named_star_planet() {
        let pos = parse_body_name("A 2 a");
        assert_eq!(pos.depth, 2);
        assert_eq!(pos.sort_key, "A.002.a");
    }

    // ---------------------------------------------------------------
    // Sub-moons (moon of a moon)
    // ---------------------------------------------------------------

    #[test]
    fn sub_moon() {
        let pos = parse_body_name("A 2 e a");
        assert_eq!(pos.depth, 3);
        assert_eq!(pos.sort_key, "A.002.e.a");
    }

    #[test]
    fn sub_moon_under_unnamed_star() {
        let pos = parse_body_name("6 b a");
        assert_eq!(pos.depth, 3);
        assert_eq!(pos.sort_key, "006.b.a");
    }

    // ---------------------------------------------------------------
    // Belt clusters
    // ---------------------------------------------------------------

    #[test]
    fn belt_cluster_under_star() {
        let pos = parse_body_name("A Belt Cluster 1");
        assert!(pos.is_belt_cluster);
        assert_eq!(pos.sort_key, "A.belt.001");
    }

    #[test]
    fn belt_cluster_under_binary() {
        let pos = parse_body_name("B A Belt Cluster 2");
        assert!(pos.is_belt_cluster);
        assert_eq!(pos.sort_key, "B.A.belt.002");
    }

    // ---------------------------------------------------------------
    // Hierarchy ordering: parent < child
    // ---------------------------------------------------------------

    #[test]
    fn star_sorts_before_its_planets() {
        let star = parse_body_name("A");
        let planet = parse_body_name("A 1");
        assert!(star.sort_key < planet.sort_key);
    }

    #[test]
    fn planet_sorts_before_its_moons() {
        let planet = parse_body_name("A 2");
        let moon = parse_body_name("A 2 a");
        assert!(planet.sort_key < moon.sort_key);
    }

    #[test]
    fn moon_sorts_before_its_submoons() {
        let moon = parse_body_name("A 2 e");
        let submoon = parse_body_name("A 2 e a");
        assert!(moon.sort_key < submoon.sort_key);
    }

    // ---------------------------------------------------------------
    // Real-world examples from journal data
    // ---------------------------------------------------------------

    #[test]
    fn real_system_ordering() {
        // From Prudgeou VD-B e1 journal data
        let mut bodies: Vec<BodyPosition> = vec![
            parse_body_name(""),       // primary star
            parse_body_name("1"),      // planet 1
            parse_body_name("2"),      // planet 2
            parse_body_name("2 a"),    // moon 2a
            parse_body_name("5 c"),    // moon 5c
            parse_body_name("8 a"),    // moon 8a
            parse_body_name("8 c"),    // moon 8c
            parse_body_name("8 c a"),  // sub-moon 8ca
            parse_body_name("9"),      // planet 9
        ];

        let original_order: Vec<String> = bodies.iter().map(|b| b.sort_key.clone()).collect();
        bodies.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        let sorted_order: Vec<String> = bodies.iter().map(|b| b.sort_key.clone()).collect();

        assert_eq!(original_order, sorted_order, "Bodies should already be in correct order");
    }

    #[test]
    fn multi_star_system_ordering() {
        // From a multi-star system in journal data
        let mut bodies: Vec<BodyPosition> = vec![
            parse_body_name("A"),
            parse_body_name("A 1"),
            parse_body_name("A 2"),
            parse_body_name("A 2 a"),
            parse_body_name("B"),
            parse_body_name("B 6"),
            parse_body_name("B 7"),
            parse_body_name("B 7 a"),
            parse_body_name("C"),
            parse_body_name("CDE 1"),
            parse_body_name("CDE 2"),
        ];

        let original_order: Vec<String> = bodies.iter().map(|b| b.sort_key.clone()).collect();
        bodies.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        let sorted_order: Vec<String> = bodies.iter().map(|b| b.sort_key.clone()).collect();

        assert_eq!(original_order, sorted_order, "Multi-star system ordering");
    }
}
