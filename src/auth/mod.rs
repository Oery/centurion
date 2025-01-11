pub mod creds;
mod microsoft;
pub mod requests;
pub mod responses;
mod xbox;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub use microsoft::Microsoft;
pub use xbox::XboxLive;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileData {
    pub access_token: String,
    pub username: String,
    pub uuid: String,
}

pub async fn login(email: &str, password: &str) -> Result<UserProfileData> {
    let client = Client::builder().cookie_store(true).build()?;

    let xbx = XboxLive::new(&client);
    let mic = Microsoft::new(&client);

    let pre_auth = xbx.pre_auth().await?;
    let login_res = xbx.user_login(email, password, pre_auth).await?;
    let xbl = mic.xbl_authenticate(&login_res).await?;
    let xsts = mic.xsts_authenticate(&xbl).await?;

    let access_token = mic.login_with_xbox(&xsts.token, &xsts.user_hash).await?;
    if !mic.user_hash_game(&access_token).await? {
        return Err(anyhow!("Account is not premium"));
    }

    let profile = mic.get_user_profile(&access_token).await?;
    Ok(UserProfileData {
        access_token,
        username: profile.username,
        uuid: profile.uuid,
    })
}
