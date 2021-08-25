#![warn(clippy::pedantic)]

mod fileio;
mod xbox;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let account_vec = fileio::read_acc_file().with_context(|| "Failed to read accounts.txt")?;
    let mut bearer_vec = Vec::new();
    for account in account_vec {
        let authenticator = xbox::Auth::new(&account.email, &account.password)
            .with_context(|| "Error creating authenticator")?;
        let access_token = authenticator
            .get_access_token()
            .with_context(|| format!("Error getting access token for {}", account.email))?;
        let bearer_token = authenticator
            .get_bearer_token(&access_token)
            .with_context(|| format!("Error getting bearer token for {}", account.email))?;
        bearer_vec.push(bearer_token);
    }
    fileio::write_bearer_file(&bearer_vec).with_context(|| "Failed to write bearers.txt")?;
    Ok(())
}
