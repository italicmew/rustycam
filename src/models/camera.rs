use std::fmt::{Formatter, Debug};

use onvif::{discovery::Device, soap::client::Credentials};
use reqwest::Url;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Camera {
    pub name: String,
    user: Option<String>,
    password: Option<String>,
    urls: Vec<String>
}

impl Debug for Camera {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("name", &self.name)
            .finish()
    }
}

impl From<Device> for Camera {
    fn from(device: Device) -> Self {
        let urls: Vec<String> = device.urls.iter().map(|url| url.to_string()).collect();

        Camera {
            name: device.name.unwrap_or_else(|| String::new()),
            user: None,
            password: None,
            urls,
        }
    }
}

impl Camera {
    pub fn new(name: String) -> Self {
        Camera {
            name,
            user: None,
            password: None,
            urls: Vec::new(),
        }
    }

    pub fn set_credentials(&mut self, user: Option<String>, pwd: Option<String>) {
        self.user = user;
        self.password = pwd;
    }

    pub fn get_onvif_credentials(&self) -> Option<Credentials> {
        match (&self.user, &self.password) {
            (Some(user), Some(pwd)) => Some(Credentials {
                username: user.clone(),
                password: pwd.clone(),
            }),
            _ => None,
        }
    }

    pub fn get_onvif_url(&self) -> Option<Url> {
        self.urls.first().and_then(|url_str| Url::parse(url_str).ok())
    }

    pub fn match_config(&self) {
        
    }

}