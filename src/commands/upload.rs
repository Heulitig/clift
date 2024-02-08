pub(crate) async fn upload(site: Option<&String>) -> UploadResult<()> {
    let current_dir = std::env::current_dir()?;

    let site = match site {
        Some(site) => site.clone(),
        None => get_site_without_parsing_ftd(&current_dir).await?,
    };

    let github_action_id_token_request = clift::commands::github_action_id_token_request()?;

    println!("Initialing Upload....");
    let data =
        initiate_upload(site.as_str(), &current_dir, &github_action_id_token_request).await?;

    upload_(&data, &current_dir).await?;

    println!("Committing Upload...");

    commit_upload(site.as_str(), &data, &github_action_id_token_request).await?;

    println!("Upload Done");
    Ok(())
}

#[derive(serde::Deserialize)]
struct SuccessResponse<T> {
    data: T,
    #[allow(dead_code)]
    success: bool,
}

#[derive(serde::Deserialize, Clone)]
pub struct PreSignedRequest {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(serde::Deserialize)]
struct InitiateUploadResponse {
    new_files: Vec<String>,
    updated_files: Vec<String>,
    upload_session_id: i64,
    tejar_file_id: i64,
    pre_signed_request: PreSignedRequest,
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

async fn initiate_upload(
    site: &str,
    current_dir: &std::path::Path,
    github_action_id_token_request: &clift::commands::GithubActionIdTokenRequest,
) -> UploadResult<InitiateUploadResponse> {
    let content_to_upload = get_local_files(current_dir).await?;

    let response = call_api(
        reqwest::Client::new()
            .post(initiate_upload_api())
            .json(&InitiateUploadRequest {
                site: site.to_string(),
                files: content_to_upload,
            }),
        github_action_id_token_request,
    )
    .await?;

    if !response.status().is_success() {
        return Err(UploadError::APIError {
            url: initiate_upload_api(),
            message: response.text().await?,
        });
    }

    let json: SuccessResponse<InitiateUploadResponse> = response.json().await?;

    Ok(json.data)
}

enum Uploader {
    File(tokio::fs::File),
    S3(PreSignedRequest, Vec<u8>),
}

impl Uploader {
    async fn debug(path: &std::path::Path) -> UploadResult<Uploader> {
        let file = tokio::fs::File::create(path).await?;
        Ok(Uploader::File(file))
    }

    fn s3(sr: PreSignedRequest) -> Uploader {
        Uploader::S3(sr, vec![])
    }

    async fn upload(&mut self, path: &std::path::Path) -> UploadResult<()> {
        use tokio::io::AsyncWriteExt;
        match self {
            Uploader::File(file) => file.write_all(&tokio::fs::read(path).await?).await?,
            Uploader::S3(_, v) => {
                v.append(&mut tokio::fs::read(path).await?);
            }
        }
        Ok(())
    }

    async fn commit(&mut self) -> UploadResult<()> {
        if let Uploader::S3(sr, v) = self {
            let client = reqwest::Client::new();
            let mut request = client.request(
                reqwest::Method::from_bytes(sr.method.as_bytes()).unwrap(),
                &sr.url,
            );
            for (k, v) in sr.headers.iter() {
                request = request.header(k, v);
            }

            request.body(v.clone()).send().await?;
        }

        Ok(())
    }
}

async fn upload_(data: &InitiateUploadResponse, current_dir: &std::path::Path) -> UploadResult<()> {
    let mut uploader = if std::env::var("DEBUG_UPLOAD").is_ok() {
        let path = current_dir
            .parent()
            .unwrap()
            .join(format!("{}.tejar", data.tejar_file_id));
        Uploader::debug(&path).await?
    } else {
        Uploader::s3(data.pre_signed_request.clone())
    };

    upload_stream_in_debug_mode_(
        &mut uploader,
        data.new_files.as_slice(),
        current_dir,
        "Added",
    )
    .await?;
    upload_stream_in_debug_mode_(
        &mut uploader,
        data.updated_files.as_slice(),
        current_dir,
        "Updated",
    )
    .await?;

    uploader.commit().await
}

async fn upload_stream_in_debug_mode_(
    uploader: &mut Uploader,
    files: &[String],
    current_dir: &std::path::Path,
    status: &str,
) -> UploadResult<()> {
    for file_name in files.iter() {
        uploader.upload(&current_dir.join(file_name)).await?;
        println!("{file_name}.... {status}");
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct CommitUploadRequest {
    site: String,
    upload_session_id: i64,
    tejar_file_id: i64,
}

async fn commit_upload(
    site: &str,
    data: &InitiateUploadResponse,
    github_action_id_token_request: &clift::commands::GithubActionIdTokenRequest,
) -> UploadResult<()> {
    let response = call_api(
        reqwest::Client::new()
            .post(commit_upload_api())
            .json(&CommitUploadRequest {
                site: site.to_string(),
                upload_session_id: data.upload_session_id,
                tejar_file_id: data.tejar_file_id,
            }),
        github_action_id_token_request,
    )
    .await?;

    if !response.status().is_success() {
        return Err(UploadError::APIError {
            url: commit_upload_api(),
            message: response.text().await?,
        });
    }

    Ok(())
}

async fn get_local_files(current_dir: &std::path::Path) -> UploadResult<Vec<ContentToUpload>> {
    use tokio::io::AsyncReadExt;

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

        files.push(ContentToUpload {
            file_name: path_without_package_dir,
            // TODO: create the hash using file stream instead of reading entire
            //       file content into memory
            sha256_hash: clift::commands::utils::generate_hash(&content),
            file_size: content.len(),
        });
    }

    Ok(files)
}

async fn call_api(
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

fn api(name: &str) -> String {
    if let Ok(url) = std::env::var("DEBUG_API_FIFTHTRY_COM") {
        return format!("{}/api/{name}/", url);
    }
    format!("{}/api/{name}/", clift::API_FIFTHTRY_COM)
}
fn initiate_upload_api() -> String {
    api("initiate-upload")
}

fn commit_upload_api() -> String {
    api("commit-upload")
}

async fn get_site_without_parsing_ftd(current_dir: &std::path::Path) -> UploadResult<String> {
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

/*
To run in debug mode

```sh
export ACTIONS_ID_TOKEN_REQUEST_URL="url"
export ACTIONS_ID_TOKEN_REQUEST_TOKEN="token"
export DEBUG_UPLOAD="true"
```
*/
