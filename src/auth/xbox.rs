use anyhow::{anyhow, Result};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use reqwest::header;
use reqwest::Client;
use std::collections::HashMap;

use super::responses::{PreAuthResponse, UserLoginResponse};

pub const USER_AGENT: &str = "Mozilla/5.0 (XboxReplay; XboxLiveAuth/3.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/71.0.3578.98 Safari/537.36";

pub const AUTHORIZE: &str = "https://login.live.com/oauth20_authorize.srf?client_id=000000004C12AE6F&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=service::user.auth.xboxlive.com::MBI_SSL&display=touch&response_type=token&locale=en";

pub struct XboxLive<'a> {
    client: &'a Client,
}

impl<'a> XboxLive<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn pre_auth(&self) -> Result<PreAuthResponse> {
        let res = self
            .client
            .get(AUTHORIZE)
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await?
            .error_for_status()?;

        let Ok(res_text) = res.text().await else {
            return Err(anyhow!("Failed to get pre auth response."));
        };

        let sft_tag_regex = Regex::new(r"sFTTag:'(.*?)'")?;
        let sft_tag_match = sft_tag_regex.captures(&res_text).unwrap();
        let sft_tag_value = sft_tag_match.get(1).unwrap().as_str();

        let ppft_regex = Regex::new(r#"value="(.*?)""#)?;
        let ppft_value = ppft_regex
            .captures(sft_tag_value)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str());

        let url_post_regex = Regex::new(r"urlPost:'([^']+)").unwrap();
        let url_post_value = url_post_regex
            .captures(&res_text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str());

        let Some(url_post_value) = url_post_value else {
            return Err(anyhow!("Failed to extract urlPost value."));
        };

        let Some(ppft_value) = ppft_value else {
            return Err(anyhow!("Failed to convert urlPost value to string."));
        };

        Ok(PreAuthResponse {
            url_post: url_post_value.to_string(),
            ppft: ppft_value.to_string(),
        })
    }

    pub async fn user_login(
        &self,
        email: &str,
        password: &str,
        pre_auth: PreAuthResponse,
    ) -> Result<UserLoginResponse> {
        let post_data = format!(
            "login={}&loginfmt={}&passwd={}&PPFT={}",
            self.encode(email),
            self.encode(email),
            self.encode(password),
            pre_auth.ppft
        );

        let res = self
            .client
            .post(&pre_auth.url_post)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::USER_AGENT, USER_AGENT)
            .body(post_data)
            .send()
            .await?
            .error_for_status()?;

        let url = res.url().as_str().to_string();
        let res_text = res.text().await?;

        if !url.contains("access_token") && url == pre_auth.url_post {
            match res_text {
                _ if res_text.contains("Sign in to") => {
                    return Err(anyhow!("Provided credentials was invalid."));
                }
                _ if res_text.contains("Help us protect your account") => {
                    return Err(anyhow!("2FA is enabled but not supported yet."));
                }
                _ => {
                    return Err(anyhow!("Something went wrong"));
                }
            }
        }

        let fragment = url.split('#').nth(1).unwrap();

        let params: HashMap<String, String> = fragment
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                let key = parts.next()?;
                let value = parts.next()?;
                Some((key.to_string(), value.to_string()))
            })
            .collect();

        Ok(UserLoginResponse {
            refresh_token: params.get("refresh_token").unwrap().to_string(),
            access_token: params.get("access_token").unwrap().to_string(),
            expires_in: params.get("expires_in").unwrap().parse()?,
            logged_in: true,
        })
    }

    fn encode(&self, data: &str) -> String {
        utf8_percent_encode(data, NON_ALPHANUMERIC).to_string()
    }
}
