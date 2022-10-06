#[test]
fn test_readme_deps() {
    version_sync::assert_markdown_deps_updated!("README.md");
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}

#[test]
fn test_changelog_mentions_version() {
    version_sync::assert_contains_regex!("CHANGELOG.md", "^## [0-9]{4}-[01][0-9]-[0-3][0-9] -- {version}");
}
