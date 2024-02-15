fn endpoint() -> String {
    clift::api::endpoint("initiate-upload")
}

#[derive(serde::Serialize)]
struct InitiateUploadRequest {
    site: String,
    files: Vec<ContentToUpload>,
}

#[derive(serde::Deserialize)]
pub struct InitiateUploadResponse {
    pub new_files: Vec<String>,
    pub updated_files: Vec<String>,
    pub upload_session_id: i64,
    pub tejar_file_id: i64,
    pub pre_signed_request: PreSignedRequest,
}

#[derive(serde::Deserialize, Clone)]
pub struct PreSignedRequest {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ContentToUpload {
    pub file_name: String,   // name of the file
    pub sha256_hash: String, // hash of the file
    pub file_size: usize,    // size of the file
}

#[derive(Debug, thiserror::Error)]
pub enum InitiateUploadError {
    #[error("cant get local files: {0}")]
    CantGetLocalFiles(#[from] clift::utils::GetLocalFilesError),
    #[error("cant call api: {0}")]
    CantCallAPI(#[from] reqwest::Error),
    #[error("cant read body during error: {0}")]
    CantReadBodyDuringError(reqwest::Error),
    #[error("got error from api: {0}")]
    APIError(String),
    #[error("cant parse json: {0}")]
    CantParseJson(#[from] serde_json::Error),
    #[error("got failure from ft: {0:?}")]
    GotFailure(std::collections::HashMap<String, String>),
}

pub async fn initiate_upload(
    site: &str,
    current_dir: &std::path::Path,
    update_token: &clift::utils::UpdateToken,
) -> Result<InitiateUploadResponse, InitiateUploadError> {
    let content_to_upload = clift::utils::get_local_files(current_dir).await?;

    let response = clift::utils::call_api(
        reqwest::Client::new()
            .post(clift::api::initiate_upload::endpoint())
            .json(&InitiateUploadRequest {
                site: site.to_string(),
                files: content_to_upload,
            }),
        update_token,
    )
    .await
    .map_err(InitiateUploadError::CantCallAPI)?;

    if !response.status().is_success() {
        return Err(InitiateUploadError::APIError(
            response
                .text()
                .await
                .map_err(InitiateUploadError::CantReadBodyDuringError)?,
        ));
    }

    let json: clift::api::ApiResponse<InitiateUploadResponse> = response.json().await?;

    if !json.success {
        // TODO: remove unwrap
        return Err(InitiateUploadError::GotFailure(json.errors.unwrap()));
    }

    Ok(json.data.unwrap()) // TODO: remove unwrap
}
