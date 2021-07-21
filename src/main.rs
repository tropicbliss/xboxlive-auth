#![warn(clippy::pedantic)]

mod cli;
mod minecraft;
mod xbox;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let account_data = cli::Args::parse_args();
    let xbox = xbox::Auth::new(account_data.email, account_data.password)
        .with_context(|| "Error creating an authenticator")
        .unwrap();
    let access_token = xbox
        .get_bearer_token()
        .with_context(|| "Error getting access token")
        .unwrap();
    let minecraft = minecraft::Auth::new(access_token)?;
    let bearer_token = minecraft
        .get_bearer_token()
        .with_context(|| "Error getting bearer token")
        .unwrap();
    println!("{}", bearer_token);
    Ok(())
}
