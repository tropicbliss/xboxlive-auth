use anyhow::{bail, Result};
use regex::Regex;
use reqwest::blocking::Client;
use std::{collections::HashMap, time::Duration};

pub struct Auth {
    username: String,
    password: String,
    client: Client,
}

pub struct LoginData {
    ppft: String,
    url_post: String,
}

impl Auth {
    pub fn new(email: String, password: String) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self {
            username: email,
            password,
            client,
        })
    }

    pub fn get_bearer_token(&self) -> Result<String> {
        let login_data = self.get_login_data()?;
        let access_token = self.sign_in(&login_data)?;
        Ok(access_token)
    }

    fn get_login_data(&self) -> Result<LoginData> {
        const URL: &str = "https://login.live.com/oauth20_authorize.srf?client_id=000000004C12AE6F&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=service::user.auth.xboxlive.com::MBI_SSL&display=touch&response_type=token&locale=en";
        let res = self.client.get(URL).send()?;
        let html = res.text()?;
        let ppft_re = Regex::new(r#"value="(.+?)""#).unwrap();
        let ppft_captures = ppft_re.captures(&html).unwrap();
        let ppft = ppft_captures.get(1).unwrap().as_str().to_string();
        let urlpost_re = Regex::new(r#"urlPost:'(.+?)'"#).unwrap();
        let urlpost_captures = urlpost_re.captures(&html).unwrap();
        let url_post = urlpost_captures.get(1).unwrap().as_str().to_string();
        Ok(LoginData { ppft, url_post })
    }

    fn sign_in(&self, login_data: &LoginData) -> Result<String> {
        let params = [
            ("login", &self.username),
            ("loginfmt", &self.username),
            ("passwd", &self.password),
            ("PPFT", &login_data.ppft),
        ];
        let res = self
            .client
            .post(&login_data.url_post)
            .form(&params)
            .send()?;
        let url = res.url().clone();
        let text = res.text()?;
        if !url.to_string().contains("access_token") && url.as_str() == login_data.url_post {
            if text.contains("Sign in to") {
                bail!("Incorrect credentials");
            }
            if text.contains("2FA is enabled but not supported yet!") {
                bail!("Please disable 2FA at https://account.live.com/activity");
            }
        }
        let mut param: HashMap<&str, &str> = url
            .fragment()
            .unwrap()
            .split('&')
            .map(|kv| {
                let mut key_value: Vec<&str> = kv.split('=').collect();
                (key_value.remove(0), key_value.remove(0))
            })
            .collect();
        Ok(param.remove("access_token").unwrap().to_string())
    }
}
