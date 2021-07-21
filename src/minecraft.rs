use anyhow::{bail, Result};
use reqwest::{blocking::Client, header::ACCEPT};
use serde_json::{json, Value};
use std::time::Duration;

pub struct Auth {
    access_token: String,
    client: Client,
}

struct XBLData {
    token: String,
    userhash: String,
}

impl Auth {
    pub fn new(access_token: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
        Ok(Self {
            access_token,
            client,
        })
    }

    pub fn get_bearer_token(&self) -> Result<String> {
        let xbl_data = self.authenticate_with_xbl()?;
        let xsts_token = self.authenticate_with_xsts(&xbl_data.token)?;
        let bearer_token = self.authenticate_with_minecraft(&xbl_data.userhash, &xsts_token)?;
        Ok(bearer_token)
    }

    fn authenticate_with_xbl(&self) -> Result<XBLData> {
        let url = "https://user.auth.xboxlive.com/user/authenticate";
        let json = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": self.access_token
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
