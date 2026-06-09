use slugify_smoke::slugify;

#[test]
fn hello_world_basic_example() {
    assert_eq!(slugify("Hello, World!"), "hello-world");
}

#[test]
fn integration_smoke_examples_from_spec() {
    assert_eq!(slugify("Crème brûlée déjà vu"), "creme-brulee-deja-vu");
    assert_eq!(slugify("Æther & Straße"), "aether-strasse");
}

#[test]
fn collapses_repeated_separators() {
    assert_eq!(
        slugify("  Multiple---spaces___and punctuation!!! "),
        "multiple-spaces-and-punctuation"
    );
    // Whitespace, underscores, hyphens, and punctuation mixed.
    assert_eq!(slugify("a - _ . , b"), "a-b");
}

#[test]
fn trims_leading_and_trailing_separators() {
    assert_eq!(slugify("---hello---"), "hello");
    assert_eq!(slugify("!!!world!!!"), "world");
    assert_eq!(slugify("   foo   "), "foo");
    assert_eq!(slugify("..."), "");
}

#[test]
fn preserves_ascii_digits_in_place() {
    assert_eq!(slugify("version 2.0.1"), "version-2-0-1");
    assert_eq!(slugify("123"), "123");
    assert_eq!(slugify("a1b2c3"), "a1b2c3");
    assert_eq!(slugify("Top 10 Items"), "top-10-items");
}

#[test]
fn lowercases_ascii_letters() {
    assert_eq!(slugify("ABCDEFG"), "abcdefg");
    assert_eq!(slugify("MixedCASE"), "mixedcase");
    assert_eq!(slugify("HelloWorld"), "helloworld");
}

#[test]
fn transliterates_latin1_accents() {
    assert_eq!(slugify("à á â ã ä å ā"), "a-a-a-a-a-a-a");
    assert_eq!(slugify("ç ć"), "c-c");
    assert_eq!(slugify("è é ê ë ē"), "e-e-e-e-e");
    assert_eq!(slugify("ì í î ï ī"), "i-i-i-i-i");
    assert_eq!(slugify("ñ"), "n");
    assert_eq!(slugify("ò ó ô õ ö ø ō"), "o-o-o-o-o-o-o");
    assert_eq!(slugify("ù ú û ü ū"), "u-u-u-u-u");
    assert_eq!(slugify("ý ÿ"), "y-y");
}

#[test]
fn transliterates_multi_character_expansions() {
    // Lowercase ligatures.
    assert_eq!(slugify("æ"), "ae");
    assert_eq!(slugify("œ"), "oe");
    assert_eq!(slugify("ß"), "ss");
    // Uppercase ligatures.
    assert_eq!(slugify("Æ"), "ae");
    assert_eq!(slugify("Œ"), "oe");
    // Expansions in context with separators.
    assert_eq!(slugify("æther"), "aether");
    assert_eq!(slugify("œuvre"), "oeuvre");
    assert_eq!(slugify("straße"), "strasse");
}

#[test]
fn returns_empty_for_emoji_only_input() {
    assert_eq!(slugify("💡🔥"), "");
    assert_eq!(slugify("🎉🎊🎈"), "");
    assert_eq!(slugify("😀"), "");
}

#[test]
fn returns_empty_for_whitespace_and_punctuation_only() {
    // Empty string.
    assert_eq!(slugify(""), "");
    // Whitespace only.
    assert_eq!(slugify("   "), "");
    assert_eq!(slugify("\t\n\r "), "");
    // Punctuation only.
    assert_eq!(slugify("!!!---___..."), "");
    assert_eq!(slugify("@#$%^&*()"), "");
}

#[test]
fn does_not_panic_on_mixed_unicode_input() {
    // CJK ideographs.
    let _ = slugify("中文测试");
    // Cyrillic.
    let _ = slugify("Привет мир");
    // Greek.
    let _ = slugify("Γειά σου");
    // Arabic.
    let _ = slugify("مرحبا");
    // Hebrew.
    let _ = slugify("שלום");
    // Devanagari.
    let _ = slugify("नमस्ते");
    // Mixed scripts with ASCII words.
    let _ = slugify("Hello 中文 World");
    // Arrows and symbols.
    let _ = slugify("→ ← ↑ ↓");
    // Mathematical symbols.
    let _ = slugify("∑ ∫ ∂ ∇");
    // Combining diacritics.
    let _ = slugify("e\u{0301}");
    // Zero-width characters.
    let _ = slugify("hello\u{200B}world");
    // Long mixed input.
    let _ = slugify("Æther 中文 Café 123 Straße 🚀");
}
