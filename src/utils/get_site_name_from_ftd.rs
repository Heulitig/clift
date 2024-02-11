#[derive(thiserror::Error, Debug)]
pub enum GetSiteNameFromFtdError {
    #[error("package not found")]
    PackageNotFound,
    #[error("cant open FASTN.ftd {0}")]
    CantOpenFASTNFile(std::io::Error),
    #[error("cant read FASTN.ftd {0}")]
    CantReadFASTNFile(std::io::Error),
}

pub async fn get_site_name_from_ftd(
    current_dir: &std::path::Path,
) -> Result<String, GetSiteNameFromFtdError> {
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(current_dir.join("FASTN.ftd"))
        .await
        .map_err(GetSiteNameFromFtdError::CantOpenFASTNFile)?;

    let mut fastn_content = String::new();
    file.read_to_string(&mut fastn_content)
        .await
        .map_err(GetSiteNameFromFtdError::CantReadFASTNFile)?;

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

    package_name.ok_or(GetSiteNameFromFtdError::PackageNotFound)
}
