#![warn(clippy::pedantic)]

mod cli;
mod xbox;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    env_logger::init();
    let account_data = cli::Args::parse_args();
    let xbox = xbox::Auth::new(account_data.email, account_data.password)
        .with_context(|| "Error creating an authenticator")?;
    let access_token = xbox
        .get_access_token()
        .with_context(|| "Error getting access token")?;
    let bearer_token = xbox
        .get_bearer_token(&access_token)
        .with_context(|| "Error getting bearer token")?;
    println!("{}", bearer_token);
    Ok(())
}
