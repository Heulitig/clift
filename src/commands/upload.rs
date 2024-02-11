pub async fn upload(site: Option<&String>) -> Result<(), UploadError> {
    let current_dir = std::env::current_dir().map_err(|_| UploadError::CanNotReadCurrentDir)?;

    let site = match site {
        Some(site) => site.clone(),
        None => clift::utils::get_site_name_from_ftd(&current_dir).await?,
    };

    let github_action_id_token_request = clift::utils::github_action_id_token_request()?;

    println!("Initialing Upload....");
    let data =
        clift::api::initiate_upload(site.as_str(), &current_dir, &github_action_id_token_request)
            .await?;

    upload_(&data, &current_dir).await?;

    println!("Committing Upload...");

    clift::api::commit_upload(site.as_str(), &data, &github_action_id_token_request).await?;

    println!("Upload Done");
    Ok(())
}

async fn upload_(
    data: &clift::api::InitiateUploadResponse,
    current_dir: &std::path::Path,
) -> Result<(), UploadError> {
    let mut uploader = match std::env::var("DEBUG_UPLOAD") {
        Ok(path) => {
            let path = std::path::PathBuf::from(path).join(format!("{}.tejar", data.tejar_file_id));
            clift::utils::Uploader::debug(&path).await?
        }
        Err(_) => clift::utils::Uploader::s3(data.pre_signed_request.clone()),
    };

    upload_files(
        &mut uploader,
        data.new_files.as_slice(),
        current_dir,
        "Added",
    )
    .await?;
    upload_files(
        &mut uploader,
        data.updated_files.as_slice(),
        current_dir,
        "Updated",
    )
    .await?;

    Ok(uploader.commit().await?)
}

async fn upload_files(
    uploader: &mut clift::utils::Uploader,
    files: &[String],
    current_dir: &std::path::Path,
    status: &str,
) -> Result<(), UploadError> {
    for file_name in files.iter() {
        uploader.upload(&current_dir.join(file_name)).await?;
        println!("{file_name}.... {status}");
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("CanNotReadCurrentDir")]
    CanNotReadCurrentDir,

    #[error("CantGetSiteNameFromFtd")]
    CantGetSiteNameFromFtd(#[from] clift::utils::GetSiteNameFromFtdError),

    #[error("Cant Read Github Tokens: {0}")]
    CantReadGithubTokens(#[from] clift::utils::GithubActionIdTokenRequestError),

    #[error("CantInitiateUpload: {0}")]
    CantInitiateUpload(#[from] clift::api::InitiateUploadError),

    #[error("CantCommitUpload: {0}")]
    CantCommitUpload(#[from] clift::api::CommitUploadError),

    #[error("CantUpload: {0}")]
    CantUpload(#[from] clift::utils::UploaderError),
}
