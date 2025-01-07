use reqwest::{self, blocking::Client, header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT}};
use serde_json;
use serde::Deserialize;
use std::str::FromStr;
use std::collections::HashMap;

pub enum VaultVersion {
  V1,
  V2
}

#[derive(Debug)]
pub struct VaultClient {
    address: String,
    client: reqwest::blocking::Client
}

impl VaultClient {

    pub fn new(v_addr: String, v_token: String) -> Result<Self, VaultClientError> {
        let mut headers = HeaderMap::new();
        let u_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        headers.insert(USER_AGENT, HeaderValue::from_str(&u_agent).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(HeaderName::from_str("X-Vault-Token").unwrap(), HeaderValue::from_str(&v_token).unwrap());
        
        let cli = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            address: v_addr,
            client: cli
        })    
    }

    pub fn is_sealed(&self) -> Result<bool, VaultClientError> {
        
        #[derive(Deserialize, Debug)]
        struct ApiResponse {
            sealed: bool
        }

        #[derive(Deserialize, Debug)]
        struct ApiErrorResponse {
            errors: Vec<String>
        }

        let url = format!("{}/v1/sys/seal-status", self.address);
        
        let resp = self.client.get(url).send()?.text()?;
        if let Ok(p) = serde_json::from_str::<ApiResponse>(resp.as_str()) {
            Ok(p.sealed)
        } else if let Ok(e) = serde_json::from_str::<ApiErrorResponse>(resp.as_str()) {
            let err_str = format!("Error: {}", e.errors.join(", "));
            Err(VaultClientError::Error(err_str))
        } else {
            Err(VaultClientError::Error(String::from("Invalid vault response")))
        }
    }

    pub fn get_secret_version(&self, mount: String) -> Result<VaultVersion, VaultClientError> {
        
        #[derive(Deserialize, Debug)]
        struct ApiResponse {
            data: ApiResponseData,
        }

        #[derive(Deserialize, Debug)]
        struct ApiResponseData {
            options: Option<ApiMountOptions>
        }

        #[derive(Deserialize, Debug)]
        struct ApiMountOptions {
            version: String
        }

        #[derive(Deserialize, Debug)]
        struct ApiErrorResponse {
            errors: Vec<String>
        }

        let url = format!("{}/v1/sys/internal/ui/mounts/{}", self.address, mount);
        let resp = self.client.get(url).send()?.text()?;
        if let Ok(r) = serde_json::from_str::<ApiResponse>(resp.as_str()) {
            if let Some(opt) = r.data.options {
                match opt.version.as_str() {
                    "1" => Ok(VaultVersion::V1),
                    "2" => Ok(VaultVersion::V2),
                    _ => Err(VaultClientError::Error(String::from("Can't determine KV version")))
                }
            } else {
                Err(VaultClientError::Error(String::from("Invalid vault response")))
            }
        } else if let Ok(e) = serde_json::from_str::<ApiErrorResponse>(resp.as_str()) {
            let err_str = format!("Error: {}", e.errors.join(", "));
            Err(VaultClientError::Error(err_str))
        } else {
            Err(VaultClientError::Error(String::from("Invalid vault response")))
        }
    }

    pub fn get_secret(&self, ver: VaultVersion, mount: String, path: String) -> Result<HashMap<String, String>, VaultClientError> {

        #[derive(Deserialize, Debug)]
        struct ApiResponse {
            data: ApiSecret
        }

        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum ApiSecret {
            V1(HashMap<String, String>),
            V2(ApiSecretData)
        }        

        #[derive(Deserialize, Debug)]
        struct ApiSecretData {
            data: HashMap<String, String>
        }

        #[derive(Deserialize, Debug)]
        struct ApiErrorResponse {
            errors: Vec<String>
        }

        let url: String = match ver {
            VaultVersion::V1 => format!("{}/v1/{}/{}", self.address, mount, path),
            VaultVersion::V2 =>format!("{}/v1/{}/data/{}", self.address, mount, path)
        };
        
        let resp = self.client.get(url).send()?.text()?;
        if let Ok(r) = serde_json::from_str::<ApiResponse>(resp.as_str()) {
            match r.data {
                ApiSecret::V1( s) => Ok(s),
                ApiSecret::V2(s) => Ok(s.data)
            }
        } else if let Ok(e) = serde_json::from_str::<ApiErrorResponse>(resp.as_str()) {
            let err_str = format!("Error: {}", e.errors.join(", "));
            Err(VaultClientError::Error(err_str))
        } else {
            Err(VaultClientError::Error(String::from("Invalid vault response")))
        }        
    }

    pub fn err(error: VaultClientError) -> String {
        match error {
            VaultClientError::Error(e) => e,
            VaultClientError::ReqwestError(e) => {
                if e.is_connect() {
                    String::from("Connection refused")
                } else if e.is_timeout() {
                    String::from("Connection timeout")
                } else if let Some(sc) = e.status() {
                    format!("Status: {}", sc)
                } else {
                    e.to_string()
                }
            },
            VaultClientError::JSONError(e) => {
                format!("JSON: {}", e.to_string())
            }
        }
    }
}

#[derive(Debug)]
pub enum VaultClientError {
    ReqwestError(reqwest::Error),
    JSONError(serde_json::Error),
    Error(String)
}

impl From<String> for VaultClientError {
    fn from(input: String) -> Self {
        VaultClientError::Error(input)
    }
}

impl From<reqwest::Error> for VaultClientError {
    fn from(input: reqwest::Error) -> Self {
        VaultClientError::ReqwestError(input)
    }
}

impl From<serde_json::Error> for VaultClientError {
    fn from(input: serde_json::Error) -> Self {
        VaultClientError::JSONError(input)
    }
}