use std::path::Path;

/// Load all `.md` files from a directory and return their contents.
///
/// Files are returned in sorted order by filename for deterministic context injection.
/// Returns an empty vec if the directory is empty.
pub fn load_wiki_articles(dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    let mut articles = Vec::new();
    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        articles.push(content);
    }
    Ok(articles)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn loads_md_files_from_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("alpha.md"), "# Alpha").unwrap();
        fs::write(dir.path().join("beta.md"), "# Beta").unwrap();

        let articles = load_wiki_articles(dir.path()).unwrap();
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0], "# Alpha");
        assert_eq!(articles[1], "# Beta");
    }

    #[test]
    fn skips_non_md_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("notes.md"), "content").unwrap();
        fs::write(dir.path().join("data.json"), "{}").unwrap();
        fs::write(dir.path().join("readme.txt"), "hello").unwrap();

        let articles = load_wiki_articles(dir.path()).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0], "content");
    }

    #[test]
    fn returns_sorted_order() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("zebra.md"), "Z").unwrap();
        fs::write(dir.path().join("apple.md"), "A").unwrap();
        fs::write(dir.path().join("mango.md"), "M").unwrap();

        let articles = load_wiki_articles(dir.path()).unwrap();
        assert_eq!(articles, vec!["A", "M", "Z"]);
    }

    #[test]
    fn empty_directory_returns_empty_vec() {
        let dir = tempfile::tempdir().unwrap();
        let articles = load_wiki_articles(dir.path()).unwrap();
        assert!(articles.is_empty());
    }
}
