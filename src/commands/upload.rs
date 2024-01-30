#[derive(serde::Deserialize)]
struct File {
    file_name: String,
    content: Vec<u8>,
}

pub(crate) async fn upload() -> clift::Result<()> {
    let site = get_site().await?;
    let response = reqwest::get(reqwest::Url::parse(
        format!("{}/api/all-files/?site={site}", clift::API_FIFTHTRY_COM).as_str(),
    )?)
    .await?;

    let files: Vec<File> = response.json().await?;

    Ok(())
}

async fn get_site() -> UploadResult<String> {
    use tokio::io::AsyncReadExt;

    let current_dir = std::env::current_dir()?;
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
    #[error("UploadError: PackageNotFound")]
    PackageNotFound,
    #[error("UploadError: IOError: {}", _0)]
    IOError(#[from] std::io::Error),
}

type UploadResult<T> = std::result::Result<T, UploadError>;
