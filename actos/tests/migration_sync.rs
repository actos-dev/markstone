use markstone_actos::{TAG_PATTERN, USERNAME_PATTERN, is_valid_tag, is_valid_username};
use regex::Regex;
use std::path::Path;

#[test]
fn test_migration_regex_patterns_exact_match() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let actors_sql_path =
        Path::new(manifest_dir).join("../../actos-backend/migrations/0002_actors.up.sql");
    let tags_sql_path =
        Path::new(manifest_dir).join("../../actos-backend/migrations/0007_tags.up.sql");

    // 1. Verify against 0002_actors.up.sql (ck_actors_username_format)
    if actors_sql_path.exists() {
        let content = std::fs::read_to_string(&actors_sql_path)
            .unwrap_or_else(|e| panic!("Failed to read {actors_sql_path:?}: {e}"));

        // Match: CONSTRAINT ck_actors_username_format CHECK ((username)::text ~ '^[a-z0-9_]{3,32}$')
        let re = Regex::new(r"CONSTRAINT\s+ck_actors_username_format\s+CHECK\s*\(\(username\)::text\s*~\s*'([^']+)'\)").unwrap();
        let caps = re
            .captures(&content)
            .expect("Failed to find ck_actors_username_format constraint in 0002_actors.up.sql");
        let sql_pattern = &caps[1];

        assert_eq!(
            sql_pattern, USERNAME_PATTERN,
            "markstone-actos USERNAME_PATTERN must stay in exact sync with ck_actors_username_format in 0002_actors.up.sql"
        );
    } else {
        panic!("Migration file not found at expected path: {actors_sql_path:?}");
    }

    // 2. Verify against 0007_tags.up.sql (ck_tags_name_format)
    if tags_sql_path.exists() {
        let content = std::fs::read_to_string(&tags_sql_path)
            .unwrap_or_else(|e| panic!("Failed to read {tags_sql_path:?}: {e}"));

        // Match: CONSTRAINT ck_tags_name_format CHECK ((name)::text ~ '^[a-z0-9][a-z0-9-]{0,31}$')
        let re = Regex::new(
            r"CONSTRAINT\s+ck_tags_name_format\s+CHECK\s*\(\(name\)::text\s*~\s*'([^']+)'\)",
        )
        .unwrap();
        let caps = re
            .captures(&content)
            .expect("Failed to find ck_tags_name_format constraint in 0007_tags.up.sql");
        let sql_pattern = &caps[1];

        assert_eq!(
            sql_pattern, TAG_PATTERN,
            "markstone-actos TAG_PATTERN must stay in exact sync with ck_tags_name_format in 0007_tags.up.sql"
        );
    } else {
        panic!("Migration file not found at expected path: {tags_sql_path:?}");
    }
}

#[test]
fn test_username_validation_parity_with_regex() {
    let re = Regex::new(USERNAME_PATTERN).unwrap();

    let s31 = "a".repeat(31);
    let s32 = "a".repeat(32);
    let s33 = "a".repeat(33);
    let s50 = "a".repeat(50);

    let candidates: &[&str] = &[
        "",
        "a",
        "ab",
        "abc",
        "abcd",
        "alice_123",
        "___",
        "_abc",
        "abc_",
        "Alice",
        "ALICE",
        "a-b",
        "alice-bob",
        "alice@bob",
        "alice.bob",
        "alice 123",
        &s31,
        &s32,
        &s33,
        &s50,
        "123",
        "0_0",
        "user!name",
        "user?name",
    ];

    for candidate in candidates {
        let regex_matches = re.is_match(candidate);
        let fn_matches = is_valid_username(candidate);
        assert_eq!(
            regex_matches, fn_matches,
            "Parity failure between Regex({USERNAME_PATTERN}) and is_valid_username for {candidate:?}"
        );
    }
}

#[test]
fn test_tag_validation_parity_with_regex() {
    let re = Regex::new(TAG_PATTERN).unwrap();

    let s31 = "a".repeat(31);
    let s32 = "a".repeat(32);
    let s33 = "a".repeat(33);

    let candidates: &[&str] = &[
        "",
        "a",
        "1",
        "ab",
        "rust",
        "c-sharp",
        "web-dev-123",
        "-tag",
        "--tag",
        "tag-",
        "tag--",
        "tag_name",
        "Rust",
        "TAG",
        &s31,
        &s32,
        &s33,
        "123",
        "0-1-2",
        "-",
        "--",
        "tag.name",
        "tag!name",
    ];

    for candidate in candidates {
        let regex_matches = re.is_match(candidate);
        let fn_matches = is_valid_tag(candidate);
        assert_eq!(
            regex_matches, fn_matches,
            "Parity failure between Regex({TAG_PATTERN}) and is_valid_tag for {candidate:?}"
        );
    }
}
