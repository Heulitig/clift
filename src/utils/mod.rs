mod call_api;
mod generate_hash;
mod get_local_files;
mod get_site_name_from_ftd;
mod github_token;
mod site_token;
mod uploader;
mod version;

pub use call_api::call_api;
pub use generate_hash::generate_hash;
pub use get_local_files::{get_local_files, GetLocalFilesError};
pub use get_site_name_from_ftd::{get_site_name_from_ftd, GetSiteNameFromFtdError};
pub use github_token::{
    github_oidc_action_token, GithubActionIdTokenRequestError, GithubOidcActionToken,
};
pub use site_token::SiteToken;
pub use uploader::{Uploader, UploaderError};
pub use version::version;

pub enum UpdateToken {
    SiteToken(SiteToken),
    GithubToken(GithubOidcActionToken),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTokenError {
    #[error("SiteToken: {0}")]
    SiteToken(#[from] std::env::VarError),
    #[error("GithubToken: {0}")]
    GithubToken(#[from] GithubActionIdTokenRequestError),
}

pub fn get_update_token() -> Result<UpdateToken, UpdateTokenError> {
    match github_oidc_action_token() {
        Ok(token) => Ok(UpdateToken::GithubToken(token)),
        Err(GithubActionIdTokenRequestError::TokenMissing(e)) => {
            eprintln!("Github OIDC Token missing: {e}, trying SiteToken...");
            Ok(UpdateToken::SiteToken(SiteToken::from_env()?))
        }
        Err(e) => Err(e.into()),
    }
}
