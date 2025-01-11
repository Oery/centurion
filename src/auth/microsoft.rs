use anyhow::Result;
use reqwest::header;
use reqwest::Client;

use crate::auth::requests::{XblAuthPayload, XblAuthProperties, XblLoginPayload};
use crate::auth::requests::{XstsAuthPayload, XstsAuthProperties};
use crate::auth::responses::{
    McStoreResponse, UserLoginResponse, UserProfileResponse, XblLoginResponse, XboxAuthData,
    XboxAuthResponse,
};
use crate::auth::xbox::USER_AGENT;
use crate::minecraft_api::is_giftcard;

pub struct UserProfile {
    pub username: String,
    pub uuid: String,
}

pub struct Microsoft<'a> {
    client: &'a Client,
}

impl<'a> Microsoft<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn xbl_authenticate(&self, login_res: &UserLoginResponse) -> Result<XboxAuthData> {
        let res = self
            .client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::ACCEPT, "application/json")
            .header("x-xbl-contract-version", "0")
            .json(&XblAuthPayload {
                relying_party: "http://auth.xboxlive.com".to_string(),
                token_type: "JWT".to_string(),
                properties: XblAuthProperties {
                    auth_method: "RPS".to_string(),
                    site_name: "user.auth.xboxlive.com".to_string(),
                    rps_ticket: login_res.access_token.clone(),
                },
            })
            .send()
            .await?
            .error_for_status()?;

        let xbl_auth_response: XboxAuthResponse = res.json().await?;

        Ok(XboxAuthData {
            token: xbl_auth_response.token,
            user_hash: xbl_auth_response.display_claims.xui[0].uhs.clone(),
        })
    }

    pub async fn xsts_authenticate(&self, xbl: &XboxAuthData) -> Result<XboxAuthData> {
        let res = self
            .client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::ACCEPT, "application/json")
            .header("x-xbl-contract-version", "1")
            .json(&XstsAuthPayload {
                relying_party: "rp://api.minecraftservices.com/".to_string(),
                token_type: "JWT".to_string(),
                properties: XstsAuthProperties {
                    sandbox_id: "RETAIL".to_string(),
                    user_tokens: vec![xbl.token.clone()],
                },
            })
            .send()
            .await?
            .error_for_status()?;

        let xsts_auth_response: XboxAuthResponse = res.json().await?;

        Ok(XboxAuthData {
            token: xsts_auth_response.token,
            user_hash: xsts_auth_response.display_claims.xui[0].uhs.clone(),
        })
    }

    pub async fn login_with_xbox(&self, token: &str, user_hash: &str) -> Result<String> {
        let res = self
            .client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .header(header::ACCEPT, "application/json")
            .header(header::USER_AGENT, USER_AGENT)
            .json(&XblLoginPayload {
                identity_token: format!("XBL3.0 x={user_hash};{token}"),
            })
            .send()
            .await?
            .error_for_status()?;

        let xbl_login_response: XblLoginResponse = res.json().await?;
        Ok(xbl_login_response.access_token)
    }

    pub async fn user_hash_game(&self, access_token: &str) -> Result<bool> {
        let res = self
            .client
            .get("https://api.minecraftservices.com/entitlements/mcstore")
            .header(header::ACCEPT, "application/json")
            .header(header::USER_AGENT, USER_AGENT)
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()?;

        let mc_store_res: McStoreResponse = res.json().await?;
        Ok(!mc_store_res.items.is_empty())
    }

    pub async fn get_user_profile(&self, access_token: &str) -> Result<UserProfile> {
        let res = self
            .client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .bearer_auth(access_token)
            .send()
            .await?;

        if res.status() == 404 && is_giftcard(access_token).await? {
            return Ok(UserProfile {
                username: "giftcard".to_string(),
                uuid: "giftcard".to_string(),
            });
        }

        let res = res.error_for_status()?;

        let user_profile_res: UserProfileResponse = res.json().await?;

        Ok(UserProfile {
            username: user_profile_res.name,
            uuid: user_profile_res.id,
        })
    }
}
