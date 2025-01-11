use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponse {
    pub refresh_token: String,
    pub access_token: String,
    pub expires_in: u64,
    pub logged_in: bool,
}

#[derive(Debug)]
pub struct PreAuthResponse {
    pub url_post: String,
    pub ppft: String,
}

#[derive(Debug, Deserialize)]
pub struct XblLoginResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxAuthResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
    pub display_claims: DisplayClaims,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayClaims {
    pub xui: Vec<DisplayClaimProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayClaimProperty {
    pub uhs: String,
}

#[derive(Debug)]
pub struct XboxAuthData {
    pub token: String,
    pub user_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McStoreResponse {
    pub items: Vec<McStoreItem>,
    pub key_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McStoreItem {
    pub name: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileResponse {
    pub id: String,
    pub name: String,
    pub skins: Vec<Skin>,
    pub capes: Vec<Cape>,
    pub profile_actions: ProfileActions,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub texture_key: Option<String>,
    pub variant: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileActions {}
