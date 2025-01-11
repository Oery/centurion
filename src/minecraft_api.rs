use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn change_name(name: &str, access_token: &str, client: &Client) -> Result<()> {
    client
        .put(format!(
            "https://api.minecraftservices.com/minecraft/profile/name/{name}"
        ))
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CanChangeNameResponse {
    created_at: String,
    name_change_allowed: bool,
}

pub async fn can_change_name(access_token: &str) -> Result<bool> {
    let res = Client::new()
        .get("https://api.minecraftservices.com/minecraft/profile/namechange")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?;

    let data: CanChangeNameResponse = res.json().await?;
    Ok(data.name_change_allowed)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    profile_name: String,
}

pub async fn create_profile(target_name: &str, access_token: &str, client: &Client) -> Result<()> {
    client
        .post("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(access_token)
        .json(&Payload {
            profile_name: target_name.to_string(),
        })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn is_giftcard(access_token: &str) -> Result<bool> {
    let res = Client::new()
        .get("https://api.minecraftservices.com/minecraft/profile/namechange")
        .bearer_auth(access_token)
        .send()
        .await?;

    Ok(res.status() == 404)
}
