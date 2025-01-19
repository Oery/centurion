use std::fs::File;
use std::io::Read;

use anyhow::Result;

pub struct MicrosoftCredentials {
    pub email: String,
    pub password: String,
}

pub fn load_creds() -> Result<Vec<MicrosoftCredentials>> {
    let mut creds = Vec::new();

    let mut file = File::open("accs.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    for line in contents.lines() {
        let mut parts = line.split(':');
        let email = parts.next().unwrap_or_default();
        let password = parts.next().unwrap_or_default();

        creds.push(MicrosoftCredentials {
            email: email.to_string(),
            password: password.to_string(),
        });
    }

    Ok(creds)
}
