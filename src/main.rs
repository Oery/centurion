mod auth;
mod minecraft_api;
mod worker;

use std::sync::Arc;
use std::{thread, time::Duration};

use anyhow::Result;
use chrono::{TimeDelta, TimeZone};
use chrono_tz::Europe::Paris;
use tokio::sync::Mutex;

use auth::creds::load_creds;
use auth::login;
use worker::{get_next_worker, Worker};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let credentials = load_creds()?;

    // TODO: Parse NMC droptimes

    let droptime_min = Paris.with_ymd_and_hms(2025, 1, 11, 4, 12, 18).unwrap();
    let seconds_to_login: i64 = (credentials.len() as i64) * 20;

    println!(
        "Time to log {} accounts: {}",
        credentials.len(),
        seconds_to_login
    );

    let login_time = droptime_min - chrono::Duration::seconds(2 * seconds_to_login);

    // Wait until near droptime to log accounts in
    wait_until(login_time).await?;

    let mut workers: Vec<Worker> = vec![];

    for ms_acc in &credentials {
        println!("Logging in...");
        let account = login(&ms_acc.email, &ms_acc.password).await?;
        println!("Logged in!");
        workers.push(Worker::new(account).await?);
        println!(
            "Worker created! : {:?}",
            workers.last().unwrap().account.username
        );

        if credentials.iter().last().unwrap().email == ms_acc.email {
            break;
        }

        println!("Sleeping 20 seconds...");
        thread::sleep(Duration::from_secs(20));
    }

    // Minimum Delay between requests
    // Ideally, you want it to be as low as possible, while avoiding ratelimits
    // Consistency is key to have the shortest delay between name availibility and name claim
    // A fire rate too low will cause workers to not be used as often as they could
    // A fire rate too high will cause downtime and increases the delay between name availibility and name claim

    // A Maths formula could be used but I haven't tried to find one yet

    let fire_rate = TimeDelta::milliseconds(2700);
    let mut downtime_start = None;
    let workers = Arc::new(Mutex::new(workers));

    wait_until(droptime_min).await?;

    println!("Starting to change names...");

    loop {
        let now = chrono::Local::now();
        let last_poll = {
            let workers = workers.lock().await;
            workers.iter().map(|w| w.last_poll).max().unwrap_or(now)
        };

        if now < last_poll + fire_rate {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }

        let mut workers_guard = workers.lock().await;
        if let Some(worker) = get_next_worker(&mut workers_guard) {
            if let Some(downtime_start) = downtime_start {
                let downtime = now - downtime_start;
                println!("Downtime: {downtime:?}");
            }

            if worker.client.is_none() {
                worker.init().await?;
            }

            downtime_start = None;
            println!("Polling worker: {:?} at {:?}", worker.account.username, now);
            worker.last_poll = chrono::Local::now();
            worker.polls += 1;

            let username = worker.account.username.clone();
            let workers_arc = Arc::clone(&workers);
            let username = username.clone();

            tokio::spawn(async move {
                if let Err(e) = poll_worker(&workers_arc, &username).await {
                    eprintln!("Error polling worker {}: {}", username, e);
                }
            });
        } else if downtime_start.is_none() {
            println!("No worker available, this should not happen");
            downtime_start = Some(now);
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn poll_worker(workers: &Arc<Mutex<Vec<Worker>>>, username: &str) -> Result<()> {
    let mut workers = workers.lock().await;
    if let Some(worker) = workers.iter_mut().find(|w| w.account.username == username) {
        worker.poll("Staff").await?;
    }
    Ok(())
}

async fn wait_until(time: chrono::DateTime<chrono_tz::Tz>) -> Result<()> {
    let mut last_print = chrono::Local::now();

    loop {
        let now = chrono::Local::now();
        let remaining = time.naive_local() - now.naive_local();
        if now < time {
            if now - last_print > chrono::Duration::seconds(5) {
                last_print = now;
                let seconds = remaining.num_seconds();
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;
                let seconds = seconds % 60;
                println!("Waiting... | Remaining: {}:{}:{}", hours, minutes, seconds);
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        break;
    }

    Ok(())
}
