use anyhow::{anyhow, bail, Result};
use std::{
    fs::{write, File},
    io::{BufRead, BufReader},
};

pub struct Account {
    pub email: String,
    pub password: String,
}

pub fn read_acc_file() -> Result<Vec<Account>> {
    let file = File::open("accounts.txt")?;
    let lines = BufReader::new(file).lines();
    let mut acc_vec = Vec::new();
    for line in lines {
        let line = line?;
        let line_vec: Vec<&str> = line.split(':').collect();
        if line_vec.len() != 2 {
            bail!("Failed to parse accounts.txt. If there is a colon in your email or password this will not work");
        }
        let email = line_vec
            .get(0)
            .ok_or_else(|| anyhow!("Failed to parse accounts.txt"))?;
        let password = line_vec
            .get(1)
            .ok_or_else(|| anyhow!("Failed to parse accounts.txt"))?;
        let acc = Account {
            email: (*email).to_string(),
            password: (*password).to_string(),
        };
        acc_vec.push(acc);
    }
    Ok(acc_vec)
}

pub fn write_bearer_file(bearers: &[String]) -> Result<()> {
    let data = bearers.join("\n");
    write("bearers.txt", data.as_bytes())?;
    Ok(())
}
