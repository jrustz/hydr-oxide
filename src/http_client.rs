use std::{str::FromStr, time::Duration};

use anyhow::{Context, Error, Ok};
use reqwest::{
    Client, Response,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT},
};

#[derive(Debug, Clone, Copy)]
pub enum HttpMethods {
    GET,
    POST,
}

impl FromStr for HttpMethods {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethods::GET),
            "POST" => Ok(HttpMethods::POST),
            _ => Err(anyhow::anyhow!("Unsupported HTTP method: {}", s)),
        }
    }
}
impl ToString for HttpMethods {
    fn to_string(&self) -> String {
        match self {
            Self::POST => "POST",
            Self::GET => "GET",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct HttpClient {
    url: String,
    client: Client,
}

impl HttpClient {
    pub fn new(
        base_url: &str,
        timeout: Duration,
        max_idle_connection: usize,
    ) -> Result<Self, Error> {
        let base_url = Self::normolize_url(base_url);
        if !Self::validate_shema(base_url.as_str()) {
            Err(anyhow::anyhow!("Error to validate_shema schema"))
        } else {
            let headers = Self::get_default_headers();
            let client = Client::builder()
                .timeout(timeout)
                .pool_max_idle_per_host(max_idle_connection)
                .default_headers(headers)
                .build()
                .context("Failed to build http client")?;
            Ok(Self {
                url: base_url,
                client,
            })
        }
    }

    pub fn get_default_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0",
            ),
        );
        //TODO:: add flag or logic to send another content-type add this by defaultl
        //If do logic, you shoud use payload context
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers
    }

    pub fn default(base_url: &str) -> Result<Self, Error> {
        let default_timeout = Duration::from_secs(30);
        let default_max_connections = 100;
        Self::new(base_url, default_timeout, default_max_connections)
    }

    pub fn validate_shema(base_url: &str) -> bool {
        //TODO: in big futures add another protocols, like ssh,tcp ...e.t.c
        //In the futures create common client struct for this.
        base_url.starts_with("http") || base_url.starts_with("https")
    }

    pub async fn send_request(
        &self,
        method: HttpMethods,
        template: &str,
        headers: HeaderMap,
    ) -> Result<Response, Error> {
        let request_builder = match method {
            HttpMethods::GET => self.client.get(&format!("{}?{}", self.url, template)),
            HttpMethods::POST => self.client.post(&self.url).body(template.to_string()),
        };
        let response = request_builder
            .headers(headers)
            .send()
            .await
            .context("Failed to send request")?;
        Ok(response)
    }

    pub fn normolize_url(url: &str) -> String {
        url.trim().trim_end_matches("/").to_string()
    }
}
