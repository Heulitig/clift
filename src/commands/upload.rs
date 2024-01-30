#[derive(Debug, serde::Deserialize)]
struct File {
    file_name: String,
    content: Vec<u8>,
}

pub(crate) async fn upload() -> UploadResult<()> {
    let current_dir = std::env::current_dir()?;
    let site = get_site(&current_dir).await?;

    let uploaded_files = {
        let all_files_url = reqwest::Url::parse_with_params(
            format!("{}/api/all-files/", clift::API_FIFTHTRY_COM).as_str(),
            &[("site", site)],
        )?;
        let response = reqwest::get(all_files_url).await?;
        let files: Vec<File> = response.json().await?;
        files
    };

    let local_files = get_local_files(&current_dir).await?;

    Ok(())
}

async fn get_site(current_dir: &std::path::PathBuf) -> UploadResult<String> {
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

async fn get_local_files(current_dir: &std::path::PathBuf) -> UploadResult<Vec<File>> {
    use tokio::io::AsyncReadExt;

    let ignore_path = ignore::WalkBuilder::new(current_dir);

    let mut files = vec![];
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

        files.push(File {
            file_name: path_without_package_dir,
            content,
        })
    }

    Ok(files)
}

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("UploadError: PackageNotFound")]
    PackageNotFound,
    #[error("UploadError: IOError: {}", _0)]
    IOError(#[from] std::io::Error),
    #[error("UploadError: {}", _0)]
    ReqwestError(#[from] reqwest::Error),
    #[error("URLParseError: {}", _0)]
    UrlParseError(#[from] url::ParseError),
}

type UploadResult<T> = std::result::Result<T, UploadError>;
