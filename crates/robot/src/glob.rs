//! Name patterns in robot.toml: `*` matches any run of characters, `?`
//! one character, everything else itself (case-sensitive, as CAD names
//! are).

pub fn glob_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    // Iterative wildcard match with one backtrack point for the last `*`.
    let (mut i, mut j) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while j < n.len() {
        if i < p.len() && (p[i] == '?' || p[i] == n[j]) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == '*' {
            star = Some(i);
            mark = j;
            i += 1;
        } else if let Some(s) = star {
            i = s + 1;
            mark += 1;
            j = mark;
        } else {
            return false;
        }
    }
    p[i..].iter().all(|&c| c == '*')
}

#[cfg(test)]
mod tests {
    use super::glob_match;

    #[test]
    fn literals_stars_and_questions() {
        assert!(glob_match("wheel_L", "wheel_L"));
        assert!(!glob_match("wheel_L", "wheel_R"));
        assert!(glob_match("wheel_*", "wheel_R"));
        assert!(glob_match("*blade*", "tail-blade"));
        assert!(glob_match("wheel_?", "wheel_L"));
        assert!(!glob_match("wheel_?", "wheel_LL"));
        assert!(glob_match("*", ""));
        assert!(!glob_match("Wheel*", "wheel_L"));
    }

    #[test]
    fn star_backtracks() {
        assert!(glob_match("a*b*c", "a-x-b-y-b-c"));
        assert!(!glob_match("a*b*c", "a-x-b-y"));
    }
}
