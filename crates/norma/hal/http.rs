//! http.rs - HTTP Client HAL Implementation
//!
//! Native Rust implementation of the client side of `norma:hal/http`.

use std::collections::HashMap;

use crate::datum::Valor;

pub const ERROR_HEADER: &str = "x-faber-error";

#[derive(Debug, Clone, PartialEq)]
pub struct Replicatio {
    status: i64,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Replicatio {
    pub fn nova(status: i64, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: normalize_headers(headers),
            body,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        let message = message.into();
        let mut headers = HashMap::new();
        headers.insert(ERROR_HEADER.to_string(), message.clone());
        Self::nova(0, headers, message.into_bytes())
    }

    pub fn status(&self) -> i64 {
        self.status
    }

    pub fn corpus(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn corpus_octeti(&self) -> Vec<u8> {
        self.body.clone()
    }

    pub fn corpus_json(&self) -> Valor {
        serde_json::from_slice::<serde_json::Value>(&self.body)
            .ok()
            .and_then(|value| Valor::try_from(value).ok())
            .unwrap_or(Valor::Nihil)
    }

    pub fn capita(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn caput(&self, nomen: impl AsRef<str>) -> Option<String> {
        self.headers
            .get(&normalize_header_name(nomen.as_ref()))
            .cloned()
    }

    pub fn bene(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

pub async fn petet(url: impl AsRef<str>) -> Replicatio {
    rogabit("GET", url, HashMap::new(), "").await
}

pub async fn mittet(url: impl AsRef<str>, corpus: impl Into<String>) -> Replicatio {
    rogabit("POST", url, HashMap::new(), corpus).await
}

pub async fn ponet(url: impl AsRef<str>, corpus: impl Into<String>) -> Replicatio {
    rogabit("PUT", url, HashMap::new(), corpus).await
}

pub async fn delet(url: impl AsRef<str>) -> Replicatio {
    rogabit("DELETE", url, HashMap::new(), "").await
}

pub async fn mutabit(url: impl AsRef<str>, corpus: impl Into<String>) -> Replicatio {
    rogabit("PATCH", url, HashMap::new(), corpus).await
}

pub async fn rogabit(
    modus: impl AsRef<str>,
    url: impl AsRef<str>,
    capita: HashMap<String, String>,
    corpus: impl Into<String>,
) -> Replicatio {
    let method = match reqwest::Method::from_bytes(modus.as_ref().as_bytes()) {
        Ok(method) => method,
        Err(err) => return Replicatio::error(format!("invalid HTTP method: {err}")),
    };

    let client = reqwest::Client::new();
    let mut request = client.request(method, url.as_ref()).body(corpus.into());

    for (name, value) in capita {
        request = match add_header(request, &name, &value) {
            Ok(request) => request,
            Err(message) => return Replicatio::error(message),
        };
    }

    response_snapshot(request.send().await).await
}

async fn response_snapshot(result: Result<reqwest::Response, reqwest::Error>) -> Replicatio {
    let response = match result {
        Ok(response) => response,
        Err(err) => return Replicatio::error(format!("HTTP request failed: {err}")),
    };

    let status = i64::from(response.status().as_u16());
    let headers = response_headers(response.headers());
    match response.bytes().await {
        Ok(bytes) => Replicatio::nova(status, headers, bytes.to_vec()),
        Err(err) => Replicatio::error(format!("HTTP body read failed: {err}")),
    }
}

fn add_header(
    request: reqwest::RequestBuilder,
    name: &str,
    value: &str,
) -> Result<reqwest::RequestBuilder, String> {
    let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
        .map_err(|err| format!("invalid HTTP header name `{name}`: {err}"))?;
    let value = reqwest::header::HeaderValue::from_str(value)
        .map_err(|err| format!("invalid HTTP header value for `{name}`: {err}"))?;
    Ok(request.header(name, value))
}

fn response_headers(headers: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(name, value)| {
            (
                normalize_header_name(name.as_str()),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect()
}

fn normalize_headers(headers: HashMap<String, String>) -> HashMap<String, String> {
    headers
        .into_iter()
        .map(|(name, value)| (normalize_header_name(&name), value))
        .collect()
}

fn normalize_header_name(name: &str) -> String {
    name.to_ascii_lowercase()
}
