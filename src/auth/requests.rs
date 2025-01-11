use serde::Serialize;

#[derive(Serialize)]
pub struct XblLoginPayload {
    #[serde(rename = "identityToken")]
    pub identity_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XblAuthPayload {
    pub relying_party: String,
    pub token_type: String,
    pub properties: XblAuthProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XblAuthProperties {
    pub auth_method: String,
    pub site_name: String,
    pub rps_ticket: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XstsAuthPayload {
    pub relying_party: String,
    pub token_type: String,
    pub properties: XstsAuthProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XstsAuthProperties {
    pub sandbox_id: String,
    pub user_tokens: Vec<String>,
}
