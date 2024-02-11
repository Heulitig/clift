pub async fn call_api(
    mut request_builder: reqwest::RequestBuilder,
    github_action_id_token_request: &clift::utils::GithubActionIdTokenRequest,
) -> reqwest::Result<reqwest::Response> {
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
