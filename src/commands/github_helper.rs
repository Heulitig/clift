pub(crate) struct GithubActionIdTokenRequest {
    pub token: String,
    pub url: String,
}

pub(crate) fn github_action_id_token_request(
) -> Result<GithubActionIdTokenRequest, std::env::VarError> {
    let token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN")?;
    let url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL")?;

    Ok(GithubActionIdTokenRequest { token, url })
}
