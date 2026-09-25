/// Search list for the local Bronze Notice identity.
/// Other worktree keychains use the same common name, and codesign fails
/// when more than one of them is visible.
pub fn notice_signing_search_list(notice_keychain: &str, previous: &[String]) -> Vec<String> {
    let mut next = Vec::with_capacity(previous.len() + 1);
    next.push(notice_keychain.to_string());
    for path in previous {
        if path == notice_keychain || is_notice_signing_keychain(path) {
            continue;
        }
        next.push(path.clone());
    }
    next
}

fn is_notice_signing_keychain(path: &str) -> bool {
    path.contains("bronze-notice-signing.keychain")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_list_hides_other_worktree_notice_keychains() {
        let ours = "/work/bronze/target/bronze-notice-signing.keychain-db";
        let previous = vec![
            "/other/target/bronze-notice-signing.keychain-db".to_string(),
            "/Users/me/Library/Keychains/login.keychain-db".to_string(),
            ours.to_string(),
        ];
        assert_eq!(
            notice_signing_search_list(ours, &previous),
            vec![
                ours.to_string(),
                "/Users/me/Library/Keychains/login.keychain-db".to_string(),
            ]
        );
    }
}
