pub(crate) fn assert_text_contains_all(text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            text.contains(needle),
            "expected {needle:?} to be present in {text:?}"
        );
    }
}

pub(crate) fn assert_text_contains_none(text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            !text.contains(needle),
            "expected {needle:?} to be absent from {text:?}"
        );
    }
}
