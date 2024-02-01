#[derive(Debug, serde::Deserialize)]
struct File {
    file_name: String,
    sha256_hash: String,
}

pub(crate) async fn upload(site: Option<&String>) -> UploadResult<()> {
    let current_dir = std::env::current_dir()?;

    let site = match site {
        Some(site) => site.clone(),
        None => get_site(&current_dir).await?,
    };

    let github_action_id_token_request = github_action_id_token_request();

    let uploaded_files = get_uploaded_files(site.as_str(), &github_action_id_token_request).await?;
    let local_files = get_local_files(&current_dir).await?;

    let (found_changes, form_data) = compare_files(&uploaded_files, &local_files);

    if found_changes {
        return calling_upload(form_data, site.as_str(), &github_action_id_token_request).await;
    }
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
    github_action_id_token_request: &Option<GithubActionIdTokenRequest>,
) -> UploadResult<std::collections::HashMap<String, String>> {
    #[derive(serde::Deserialize)]
    struct SuccessResponse {
        data: Vec<File>,
    }

    let all_files_url =
        reqwest::Url::parse_with_params(all_files_api().as_str(), &[("site", site)])?;
    let all_files_url_str = all_files_url.as_str().to_string();

    let response = calling_apis(
        reqwest::Client::new().get(all_files_url),
        github_action_id_token_request,
    )
    .await?;

    if !response.status().is_success() {
        return Err(UploadError::APIError {
            url: all_files_url_str,
            message: response.text().await?,
        });
    }
    let files: SuccessResponse = response.json().await?;

    Ok(files
        .data
        .into_iter()
        .map(|file| (file.file_name, file.sha256_hash))
        .collect::<std::collections::HashMap<String, String>>())
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
    uploaded_files: &std::collections::HashMap<String, String>,
    local_files: &std::collections::HashMap<String, Vec<u8>>,
) -> (bool, reqwest::multipart::Form) {
    let mut files_to_be_uploaded = reqwest::multipart::Form::new();
    let mut found_changes = false;

    // Get added or updated files
    for (file_name, content) in local_files {
        let mut file_status = "Adding";
        if let Some(uploaded_file) = uploaded_files.get(file_name) {
            let sha256_hash = clift::commands::utils::generate_hash(content);
            if sha256_hash.to_uppercase().eq(&uploaded_file.to_uppercase()) {
                continue;
            }
            file_status = "Updating";
        }
        println!("{file_status}... {file_name}");
        files_to_be_uploaded = files_to_be_uploaded.part(
            file_name.clone(),
            reqwest::multipart::Part::bytes(content.to_vec()).file_name(file_name.clone()),
        );
        found_changes = true;
    }

    // Get deleted files
    let mut deleted_files = vec![];
    for file_name in uploaded_files.keys() {
        if !local_files.contains_key(file_name) {
            println!("Deleting... {}", file_name);
            deleted_files.push(file_name.clone());
        }
    }

    if !deleted_files.is_empty() {
        files_to_be_uploaded = files_to_be_uploaded.part(
            "deleted",
            reqwest::multipart::Part::bytes(serde_json::to_vec(&deleted_files).unwrap())
                .file_name("deleted"),
        );
        found_changes = true;
    }

    (found_changes, files_to_be_uploaded)
}

async fn calling_upload(
    form_data: reqwest::multipart::Form,
    site: &str,
    github_action_id_token_request: &Option<GithubActionIdTokenRequest>,
) -> UploadResult<()> {
    let client = reqwest::Client::new();
    let upload_url = reqwest::Url::parse_with_params(upload_api().as_str(), &[("site", site)])?;
    let upload_url_str = upload_url.as_str().to_string();

    let response = calling_apis(
        client.post(upload_url).multipart(form_data),
        github_action_id_token_request,
    )
    .await?;
    if !response.status().is_success() {
        return Err(UploadError::APIError {
            url: upload_url_str,
            message: response.text().await?,
        });
    }

    println!("Done");
    Ok(())
}

struct GithubActionIdTokenRequest {
    token: String,
    url: String,
}

fn github_action_id_token_request() -> Option<GithubActionIdTokenRequest> {
    let token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN").ok()?;
    let url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").ok()?;

    Some(GithubActionIdTokenRequest { token, url })
}

async fn calling_apis(
    mut request_builder: reqwest::RequestBuilder,
    github_action_id_token_request: &Option<GithubActionIdTokenRequest>,
) -> UploadResult<reqwest::Response> {
    if let Some(github_action_id_token_request) = github_action_id_token_request {
        request_builder = request_builder
            .header(
                "X-FIFTHTRY-GH-ACTIONS-ID-TOKEN-REQUEST-TOKEN",
                github_action_id_token_request.token.clone(),
            )
            .header(
                "X-FIFTHTRY-GH-ACTIONS-ID-TOKEN-REQUEST-URL",
                github_action_id_token_request.url.clone(),
            );
    }
    Ok(request_builder.send().await?)
}

fn all_files_api() -> String {
    format!("{}/api/all-files/", clift::API_FIFTHTRY_COM)
}

fn upload_api() -> String {
    format!("{}/api/upload/", clift::API_FIFTHTRY_COM)
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
    #[error("UploadError: APIError: `{url}` API fails {message}")]
    APIError { url: String, message: String },
}

type UploadResult<T> = std::result::Result<T, UploadError>;
