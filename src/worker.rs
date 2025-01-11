use anyhow::Result;
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

        if let Some(ref client) = self.client {
            let res = client.get("http://ip-api.com/json").send().await?;
            let ip: IpResponse = res.json().await?;
            self.ip = ip.query;

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
                    let proxy_url =
                        format!("http://{}:{}@{}", self.proxy, proxy_password, proxy_host);

                    self.client = Some(
                        Client::builder()
                            .cookie_store(true)
                            .proxy(reqwest::Proxy::all(&proxy_url)?)
                            .build()?,
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn poll(&self, target_name: &str) -> Result<()> {
        if self.client.is_none() {
            println!("No client available, skipping");
        }

        if let Some(ref client) = self.client {
            if self.account.uuid == "giftcard" {
                match create_profile(target_name, &self.account.access_token, client).await {
                    Ok(_) => println!(
                        "Claimed username on {} | IP: {}",
                        self.account.username, self.ip
                    ),
                    Err(e) => {
                        if e.to_string().contains("400 Bad Request") {
                            println!(
                                "Name not available {} | IP: {}",
                                self.account.username, self.ip
                            );
                        } else {
                            println!(
                                "Error claiming username on {} | IP: {}: {e}",
                                self.account.username, self.ip
                            )
                        }
                    }
                }
            } else {
                match change_name(target_name, &self.account.access_token, client).await {
                    Ok(_) => println!(
                        "Changed name on {} | IP: {}",
                        self.account.username, self.ip
                    ),
                    Err(e) => {
                        if e.to_string().contains("403 Forbidden") {
                            println!(
                                "Name not available {} | IP: {}",
                                self.account.username, self.ip
                            );
                        } else {
                            println!(
                                "Error changing name on {} | IP: {}: {e}",
                                self.account.username, self.ip
                            )
                        }
                    }
                }
            };
        }

        Ok(())
    }
}

pub fn get_next_worker(workers: &mut [Worker]) -> Option<&mut Worker> {
    workers
        .iter_mut()
        .find(|worker| worker.last_poll + worker.min_delay < chrono::Utc::now())
}

pub fn get_last_poll(workers: &[Worker]) -> chrono::DateTime<chrono::Local> {
    workers.iter().fold(
        chrono::Local::now() - TimeDelta::days(30),
        |last, worker| {
            if worker.last_poll > last {
                worker.last_poll
            } else {
                last
            }
        },
    )
}
