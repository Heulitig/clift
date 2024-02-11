#[derive(Debug, thiserror::Error)]
pub enum GetLocalFilesError {
    #[error("CanNotReadFile {1}: {0}")]
    CantReadFile(std::io::Error, String),
}

pub async fn get_local_files(
    current_dir: &std::path::Path,
) -> Result<Vec<clift::api::ContentToUpload>, GetLocalFilesError> {
    let ignore_path = ignore::WalkBuilder::new(current_dir)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .ignore(true)
        .parents(true)
        .build();

    let mut files = vec![];
    for path in ignore_path.flatten() {
        if path.path().is_dir() {
            continue;
        }
        let content = tokio::fs::read(path.path()).await.map_err(|e| {
            GetLocalFilesError::CantReadFile(e, path.path().to_string_lossy().to_string())
        })?;

        let path_without_package_dir = path
            .path()
            .to_str()
            .unwrap()
            .to_string()
            .trim_start_matches(current_dir.to_str().unwrap())
            .trim_start_matches('/')
            .to_string();

        if path_without_package_dir.starts_with(".git/")
            || path_without_package_dir.starts_with(".github/")
            || path_without_package_dir.eq(".gitignore")
        {
            continue;
        }

        files.push(clift::api::ContentToUpload {
            file_name: path_without_package_dir,
            // TODO: create the hash using file stream instead of reading entire
            //       file content into memory
            sha256_hash: clift::utils::generate_hash(&content),
            file_size: content.len(),
        });
    }

    Ok(files)
}
