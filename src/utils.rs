use std::path::Path;

pub fn sanitize_filename(name: &str) -> String {
    sanitize_filename::sanitize_with_options(
        name,
        sanitize_filename::Options {
            truncate: true,
            windows: true,
            replacement: "_",
        },
    )
}

pub fn replace_dot_suffix(s: &str) -> String {
    if s.ends_with('.') {
        s.trim_end_matches('.').to_string() + "_"
    } else {
        s.to_string()
    }
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn get_file_extension(url: &str) -> Option<String> {
    let parsed_url = url::Url::parse(url).ok()?;
    let path = parsed_url.path();
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext))
}

pub async fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> crate::Result<()> {
    tokio::fs::create_dir_all(path).await?;
    Ok(())
}
