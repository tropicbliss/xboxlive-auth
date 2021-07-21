use anyhow::{bail, Result};
use regex::Regex;
use reqwest::{blocking::Client, header::ACCEPT};
use serde_json::{json, Value};
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

struct XBLData {
    token: String,
    userhash: String,
}

impl Auth {
    pub fn new(email: String, password: String) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .connection_verbose(true)
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self {
            username: email,
            password,
            client,
        })
    }

    pub fn get_access_token(&self) -> Result<String> {
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
        let status = res.status().clone().as_u16();
        if status != 200 {
            bail!("Something went wrong: Status code: {}", status);
        }
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

    pub fn get_bearer_token(&self, access_token: &str) -> Result<String> {
        let xbl_data = self.authenticate_with_xbl(access_token)?;
        let xsts_token = self.authenticate_with_xsts(&xbl_data.token)?;
        let bearer_token = self.authenticate_with_minecraft(&xbl_data.userhash, &xsts_token)?;
        Ok(bearer_token)
    }

    fn authenticate_with_xbl(&self, access_token: &str) -> Result<XBLData> {
        let url = "https://user.auth.xboxlive.com/user/authenticate";
        let json = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": access_token
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });
        let res = self
            .client
            .post(url)
            .json(&json)
            .header(ACCEPT, "application/json")
            .send()?;
        let status = res.status().clone().as_u16();
        if status != 200 {
            bail!("Something went wrong: Status code: {}", status);
        }
        let text = res.text()?;
        let v: Value = serde_json::from_str(&text)?;
        let token = v["Token"].as_str().unwrap().to_string();
        let userhash = v["DisplayClaims"]["xui"][0]["uhs"]
            .as_str()
            .unwrap()
            .to_string();
        Ok(XBLData { token, userhash })
    }

    fn authenticate_with_xsts(&self, token: &str) -> Result<String> {
        let url = "https://xsts.auth.xboxlive.com/xsts/authorize";
        let json = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });
        let res = self
            .client
            .post(url)
            .header(ACCEPT, "application/json")
            .json(&json)
            .send()?;
        let status = res.status().clone().as_u16();
        let text = res.text()?;
        let v: Value = serde_json::from_str(&text)?;
        if status == 401 {
            let err = v["XErr"].as_u64().unwrap();
            if err == 2_148_916_233 {
                bail!("The account doesn't have an Xbox account. Once they sign up for one (or login through minecraft.net to create one) then they can proceed with the login. This shouldn't happen with accounts that have purchased Minecraft with a Microsoft account, as they would've already gone through that Xbox signup process.");
            }
            if err == 2_148_916_238 {
                bail!("The account is a child (under 18) and cannot proceed unless the account is added to a Family by an adult. This only seems to occur when using a custom Microsoft Azure application. When using the Minecraft launchers client id, this doesn't trigger.");
            }
            bail!("Something went wrong.");
        } else if status == 200 {
            let token = v["Token"].as_str().unwrap().to_string();
            Ok(token)
        } else {
            bail!("Something went wrong: Status code: {}", status);
        }
    }

    fn authenticate_with_minecraft(&self, userhash: &str, xsts_token: &str) -> Result<String> {
        let url = "https://api.minecraftservices.com/authentication/login_with_xbox";
        let json = json!({ "identityToken": format!("XBL3.0 x={};{}", userhash, xsts_token) });
        let res = self.client.post(url).json(&json).send()?;
        let status = res.status().clone().as_u16();
        if status != 200 {
            bail!("Something went wrong: Status code: {}", status);
        }
        let text = res.text()?;
        let v: Value = serde_json::from_str(&text)?;
        let bearer = v["access_token"].as_str().unwrap().to_string();
        Ok(bearer)
    }
}
