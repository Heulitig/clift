pub(crate) async fn upload(site: Option<&String>) -> UploadResult<()> {
    let current_dir = std::env::current_dir()?;

    let site = match site {
        Some(site) => site.clone(),
        None => get_site(&current_dir).await?,
    };

    let local_files = get_local_files(&current_dir).await?;
    let github_action_id_token_request = clift::commands::github_action_id_token_request()?;

    calling_initiate_upload_api(site.as_str(), &local_files, &github_action_id_token_request)
        .await?;

    todo!()
}

async fn calling_initiate_upload_api(
    site: &str,
    local_files: &std::collections::HashMap<String, Vec<u8>>,
    github_action_id_token_request: &clift::commands::GithubActionIdTokenRequest,
) -> UploadResult<()> {
    let content_to_upload: Vec<ContentToUpload> = local_files
        .iter()
        .map(|(file_name, content)| ContentToUpload {
            file_name: file_name.to_string(),
            sha256_hash: clift::commands::utils::generate_hash(&content),
            file_size: content.len(),
        })
        .collect();

    let upload_url = reqwest::Url::parse(initiate_upload_api().as_str())?;
    let upload_url_str = upload_url.as_str().to_string();
    let client = reqwest::Client::new();

    let response = calling_apis(
        client.post(upload_url).json(&InitiateUploadRequest {
            site: site.to_string(),
            files: content_to_upload,
        }),
        github_action_id_token_request,
    )
    .await?;

    todo!()
}

#[derive(Debug, serde::Serialize)]
struct ContentToUpload {
    pub file_name: String,   // name of the file
    pub sha256_hash: String, // hash of the file
    pub file_size: usize,    // size of the file
}

#[derive(serde::Serialize)]
struct InitiateUploadRequest {
    site: String,
    files: Vec<ContentToUpload>,
}

// Returns hashmap of file_name and
async fn get_local_files(
    current_dir: &std::path::Path,
) -> UploadResult<std::collections::HashMap<String, Vec<u8>>> {
    use tokio::io::AsyncReadExt;

    let ignore_path = ignore::WalkBuilder::new(current_dir)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .ignore(true)
        .parents(true)
        .build();

    let mut files: std::collections::HashMap<String, Vec<u8>> = Default::default();
    for path in ignore_path.flatten() {
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

        if path_without_package_dir.starts_with(".git/")
            || path_without_package_dir.starts_with(".github/")
            || path_without_package_dir.eq(".gitignore")
        {
            continue;
        }

        files.insert(path_without_package_dir, content);
    }

    Ok(files)
}

async fn calling_apis(
    mut request_builder: reqwest::RequestBuilder,
    github_action_id_token_request: &clift::commands::GithubActionIdTokenRequest,
) -> UploadResult<reqwest::Response> {
    request_builder = request_builder
        .header(
            "X-FIFTHTRY-GH-ACTIONS-ID-TOKEN-REQUEST-TOKEN",
            github_action_id_token_request.token.clone(),
        )
        .header(
            "X-FIFTHTRY-GH-ACTIONS-ID-TOKEN-REQUEST-URL",
            github_action_id_token_request.url.clone(),
        );
    Ok(request_builder.send().await?)
}

fn initiate_upload_api() -> String {
    format!("{}/api/initiate-upload/", clift::API_FIFTHTRY_COM)
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

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("PackageNotFound")]
    PackageNotFound,
    #[error("IOError: {}", _0)]
    IOError(#[from] std::io::Error),
    #[error("{}", _0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("URLParseError: {}", _0)]
    UrlParseError(#[from] url::ParseError),
    #[error("APIError: `{url}` API fails {message}")]
    APIError { url: String, message: String },
    #[error("GithubTokenNotFound, Help: need `ACTIONS_ID_TOKEN_REQUEST_TOKEN` and `ACTIONS_ID_TOKEN_REQUEST_URL` environment variables")]
    GithubTokenNotFound,
    #[error("Environment Variable Error {0}")]
    EnvVarError(#[from] std::env::VarError),
}

type UploadResult<T> = std::result::Result<T, UploadError>;
