#[derive(Debug, serde::Deserialize)]
struct File {
    file_name: String,
    content: Vec<u8>,
}

pub(crate) async fn upload(site: Option<&String>) -> UploadResult<()> {
    let current_dir = std::env::current_dir()?;
    let site = match site {
        Some(site) => site.clone(),
        None => get_site(&current_dir).await?,
    };

    let uploaded_files = get_uploaded_files(site.as_str()).await?;
    let local_files = get_local_files(&current_dir).await?;

    let _files = compare_files(&uploaded_files, &local_files);

    Ok(())
}

async fn get_site(current_dir: &std::path::Path) -> UploadResult<String> {
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(current_dir.join("FASTN.ftd")).await?;
    let mut fastn_content = String::new();
    file.read_to_string(&mut fastn_content).await?;

    // Split the input into lines
    let lines: Vec<&str> = fastn_content.lines().collect();

    // Iterate over the lines to find the one containing the package name
    let mut package_name = None;
    for line in lines {
        if line.contains("-- fastn.package: ") {
            // Split the line by ':' and get the second part
            package_name = line.split_once(':').map(|(_, v)| v.trim().to_string());
            break;
        }
    }

    package_name.ok_or(UploadError::PackageNotFound)
}

async fn get_uploaded_files(
    site: &str,
) -> UploadResult<std::collections::HashMap<String, Vec<u8>>> {
    let all_files_url =
        reqwest::Url::parse_with_params(all_files_api().as_str(), &[("site", site)])?;
    let response = reqwest::get(all_files_url).await?;
    if response.status().is_client_error() {
        return Err(UploadError::APIError {
            message: response.text().await?,
        });
    }
    let files: Vec<File> = response.json().await?;
    Ok(files
        .into_iter()
        .map(|file| (file.file_name, file.content))
        .collect::<std::collections::HashMap<String, Vec<u8>>>())
}

async fn get_local_files(
    current_dir: &std::path::Path,
) -> UploadResult<std::collections::HashMap<String, Vec<u8>>> {
    use tokio::io::AsyncReadExt;

    let ignore_path = ignore::WalkBuilder::new(current_dir);

    let mut files: std::collections::HashMap<String, Vec<u8>> = Default::default();
    for path in ignore_path.build().flatten() {
        if path.path().is_dir() {
            continue;
        }
        let mut file = tokio::fs::File::open(path.path()).await?;
        let mut content = vec![];
        file.read_to_end(&mut content).await?;

        let path_without_package_dir = path
            .path()
            .to_str()
            .unwrap()
            .to_string()
            .trim_start_matches(current_dir.to_str().unwrap())
            .trim_start_matches('/')
            .to_string();

        files.insert(path_without_package_dir, content);
    }

    Ok(files)
}

fn compare_files(
    uploaded_files: &std::collections::HashMap<String, Vec<u8>>,
    local_files: &std::collections::HashMap<String, Vec<u8>>,
) -> std::collections::HashMap<String, Vec<u8>> {
    let mut files_to_be_uploaded: std::collections::HashMap<String, Vec<u8>> = Default::default();

    // Get added or updated files
    for (file_name, content) in local_files {
        if let Some(uploaded_file) = uploaded_files.get(file_name) {
            if content.eq(uploaded_file) {
                continue;
            }
        }
        files_to_be_uploaded.insert(file_name.clone(), content.clone());
    }

    // Get deleted files
    let mut deleted_files = vec![];
    for file_name in uploaded_files.keys() {
        if !local_files.contains_key(file_name) {
            deleted_files.push(file_name.clone());
        }
    }
    if !deleted_files.is_empty() {
        files_to_be_uploaded.insert(
            "deleted".to_string(),
            serde_json::to_vec(&deleted_files).unwrap(),
        );
    }

    files_to_be_uploaded
}

fn all_files_api() -> String {
    format!("{}/api/all-files/", clift::API_FIFTHTRY_COM)
}

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("UploadError: PackageNotFound")]
    PackageNotFound,
    #[error("UploadError: IOError: {}", _0)]
    IOError(#[from] std::io::Error),
    #[error("UploadError: {}", _0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("UploadError: URLParseError: {}", _0)]
    UrlParseError(#[from] url::ParseError),
    #[error("UploadError: APIError: {message}")]
    APIError { message: String },
}

type UploadResult<T> = std::result::Result<T, UploadError>;
