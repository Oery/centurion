use anyhow::{anyhow, Result};
use chrono::{Duration, TimeDelta};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    auth::UserProfileData,
    minecraft_api::{
        change_name, {create_profile, is_giftcard},
    },
};

#[derive(Debug, Deserialize)]
pub struct IpResponse {
    query: String,
}

#[derive(Debug)]
pub struct Worker {
    pub account: UserProfileData,
    pub proxy: String,
    pub last_poll: chrono::DateTime<chrono::Local>,
    pub polls: i32,
    pub min_delay: Duration,
    pub client: Option<Client>,
    pub ip: String,
    pub target_name: String,
}

impl Worker {
    pub async fn new(account: UserProfileData) -> Result<Self> {
        let min_delay = match is_giftcard(&account.access_token).await? {
            true => Duration::seconds(18),
            false => Duration::seconds(25),
        };

        let proxy_username = std::env::var("PROXY_USERNAME")?;
        let session_id = account.uuid[..8].to_string();

        Ok(Self {
            account,
            last_poll: chrono::Local::now(),
            min_delay,
            proxy: format!("{proxy_username}-session-{session_id}-lifetime-60"),
            client: None,
            ip: "".into(),
            polls: 0,
            target_name: std::env::var("TARGET_NAME")?,
        })
    }

    pub async fn init(&mut self) -> Result<()> {
        let proxy_password = std::env::var("PROXY_PASSWORD")?;
        let proxy_host = std::env::var("PROXY_URL")?;

        let proxy_url = format!("http://{}:{}@{}", self.proxy, proxy_password, proxy_host);

        self.client = Some(
            Client::builder()
                .cookie_store(true)
                .proxy(reqwest::Proxy::all(&proxy_url)?)
                .build()?,
        );

        let Some(ref client) = self.client else {
            return Err(anyhow!("Failed to create client."));
        };

        // Fetch Proxy IP
        let res = client.get("http://ip-api.com/json").send().await?;
        self.ip = res.json::<IpResponse>().await?.query;

        let res = match self.account.uuid == "giftcard" {
            true => create_profile(&self.target_name, &self.account.access_token, client).await,
            false => change_name(&self.target_name, &self.account.access_token, client).await,
        };

        if let Err(e) = res {
            if e.to_string().contains("429 Too Many Requests") {
                println!("IP is banned, trying another one");

                let proxy_username = std::env::var("PROXY_USERNAME")?;
                let session_id = format!("{:x}", rand::random::<u32>() & 0xFFFFFF);
                self.proxy = format!("{proxy_username}-session-{session_id}-lifetime-60");

                let proxy_url = format!("http://{}:{}@{}", self.proxy, proxy_password, proxy_host);

                self.client = Some(
                    Client::builder()
                        .cookie_store(true)
                        .proxy(reqwest::Proxy::all(&proxy_url)?)
                        .build()?,
                );
            }
        }

        Ok(())
    }

    pub async fn poll(&self, target_name: &str) -> Result<()> {
        let Some(ref client) = self.client else {
            return Err(anyhow!("No client available, skipping"));
        };

        let result = match self.account.uuid.as_str() {
            "giftcard" => create_profile(target_name, &self.account.access_token, client).await,
            _ => change_name(target_name, &self.account.access_token, client).await,
        };

        if result.is_ok() {
            println!(
                "Claimed username on {} | IP: {}",
                self.account.username, self.ip
            );
            return Ok(());
        }

        let error = result.unwrap_err().to_string();
        let username = &self.account.username;
        let ip = &self.ip;

        if error.contains("400") || error.contains("403") {
            println!("Name not available {username} | IP: {ip}",);
        } else {
            eprintln!("Error claiming username on {username} | IP: {ip}: {error}",)
        }

        Ok(())
    }
}

pub fn get_next_worker(workers: &mut [Worker]) -> Option<&mut Worker> {
    workers
        .iter_mut()
        .find(|worker| worker.last_poll + worker.min_delay < chrono::Utc::now())
}
