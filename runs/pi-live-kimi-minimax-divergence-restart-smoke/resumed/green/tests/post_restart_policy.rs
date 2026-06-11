use post_restart_slugify_smoke::slugify;

#[test]
fn t001_ascii_basics_and_empty_fallback() {
    assert_eq!(slugify("Hello, World!"), "hello-world");
    assert_eq!(slugify("  Multiple---spaces___and punctuation!!! "), "multiple-spaces-and-punctuation");
    assert_eq!(slugify("Top 10 Items"), "top-10-items");
    assert_eq!(slugify("already-a-slug"), "already-a-slug");
    assert_eq!(slugify(""), "untitled");
    assert_eq!(slugify("   \t\n"), "untitled");
}

#[test]
fn t002_accented_latin_transliterates_before_separator_handling() {
    assert_eq!(slugify("Crème brûlée"), "creme-brulee");
    assert_eq!(slugify("naïve approach"), "naive-approach");
    assert_eq!(slugify("São Paulo guide"), "sao-paulo-guide");
    assert_eq!(slugify("Jalapeño año"), "jalapeno-ano");
}

#[test]
fn t003_non_latin_and_emoji_fallback_or_separator() {
    assert_eq!(slugify("💡🔥"), "untitled");
    assert_eq!(slugify("中文测试"), "untitled");
    assert_eq!(slugify("Hello 中文 World"), "hello-world");
    assert_eq!(slugify("rocket 🚀 launch"), "rocket-launch");
}

#[test]
fn t004_combining_marks_are_absorbed_after_ascii_base() {
    assert_eq!(slugify("Cafe\u{0301}"), "cafe");
    assert_eq!(slugify("Sao\u{0303} Paulo"), "sao-paulo");
}
