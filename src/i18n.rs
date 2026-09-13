//! Locale-file parity test — ensures `en-US.ftl` and `fr-FR.ftl` (the dioxus-i18n
//! bundles loaded in `main.rs`) define the same key set, so a key added to one
//! locale cannot silently be missing from the other and render as its own name.

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    fn keys(ftl: &str) -> HashSet<String> {
        ftl.lines()
            // Column zero only: an indented line is a continuation or a plural
            // variant, never a key of its own.
            .filter(|line| !line.starts_with(char::is_whitespace))
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|line| line.split_once('='))
            .map(|(key, _)| key.trim().to_owned())
            .collect()
    }

    #[test]
    fn unit_locale_files_have_matching_keys() {
        let en = keys(include_str!("./i18n/en-US.ftl"));
        let fr = keys(include_str!("./i18n/fr-FR.ftl"));
        let only_en: Vec<_> = en.difference(&fr).collect();
        let only_fr: Vec<_> = fr.difference(&en).collect();
        assert!(
            only_en.is_empty() && only_fr.is_empty(),
            "locale key mismatch — only in en-US.ftl: {only_en:?}, only in fr-FR.ftl: {only_fr:?}"
        );
    }
}
