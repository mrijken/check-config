pub(crate) fn merge(contents: &str, table: toml::Value) -> Result<String, String> {
    let doc: toml::Value = toml::from_str(contents).unwrap();

    let new_doc = serde_toml_merge::merge(doc, table.clone()).unwrap();
    let new_contents = format!("{}", new_doc.as_table().unwrap());
    Ok(new_contents)
}

pub(crate) fn set(contents: &str, table: toml::Value) -> Result<String, String> {
    let doc: toml::Value = toml::from_str(contents).unwrap();

    let new_doc = serde_toml_merge::merge(doc, table.clone()).unwrap();
    let new_contents = format!("{}", new_doc.as_table().unwrap());
    Ok(new_contents)
}

pub(crate) fn remove_key(contents: &str, table: toml::Value) -> Result<String, String> {
    let mut doc: toml::Value = toml::from_str(contents).unwrap();
    let doc_table = doc.as_table_mut().unwrap();
    for (k, _) in table.as_table().unwrap() {
        // todo: nested
        doc_table.remove(k);
    }

    Ok(format!("{}", doc.as_table().unwrap()))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_merge_simple() {
        let contents = r#"
        [package]
        name = "foo"
        version = "1.0"
        "#;

        let table_contents = r#"
        [package]
        name = "bar"
        "#;

        let table = toml::from_str::<toml::Value>(table_contents).unwrap();

        let new_contents = super::merge(contents, table).unwrap();

        assert_eq!(
            new_contents,
            r#"[package]
name = "bar"
version = "1.0"
"#
        );
    }

    #[test]
    fn test_merge_nested() {
        let contents = r#"
        [package]
        name = "foo"
        version = "1.0"

        [dependencies]
        toml ="1.0"

        [dependencies.bar]
        version = "1.0"
        features = ["foo"]
        "#;

        let table_contents = r#"
        [dependencies.bar]
        version = "2.0"
        features = ["bar"]
        "#;

        let table = toml::from_str::<toml::Value>(table_contents).unwrap();

        let new_contents = super::merge(contents, table).unwrap();

        assert_eq!(
            new_contents,
            r#"[dependencies]
toml = "1.0"

[dependencies.bar]
features = ["foo", "bar"]
version = "2.0"

[package]
name = "foo"
version = "1.0"
"#
        );
    }
}
