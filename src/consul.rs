use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Serialize;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Error {
    InvalidAuthToken(reqwest::header::InvalidHeaderValue),
    Request(reqwest::Error),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAuthToken(_) => formatter.write_str("invalid Consul authentication token"),
            Self::Request(_) => formatter.write_str("Consul request failed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidAuthToken(error) => Some(error),
            Self::Request(error) => Some(error),
        }
    }
}

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(error: reqwest::header::InvalidHeaderValue) -> Self {
        Self::InvalidAuthToken(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct HealthCheck<'a> {
    #[serde(rename = "HTTP")]
    http: &'a str,
    interval: &'static str,
    timeout: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Service<'a> {
    name: &'a str,
    #[serde(rename = "ID")]
    id: &'a str,
    tags: [&'static str; 1],
    address: &'a str,
    port: u16,
    check: HealthCheck<'a>,
}

pub struct Registration {
    headers: HeaderMap,
    service_endpoint: String,
    service_id: String,
}

struct RegistrationConfig<'a> {
    consul_http_addr: Option<&'a str>,
    consul_service_address: Option<&'a str>,
    service_name: &'a str,
    consul_service_port: u16,
    consul_service_scheme: &'a str,
    headers: HeaderMap,
}

pub struct RegistrationBuilder<'a, HttpAddress = (), ServiceAddress = (), ServiceName = ()> {
    config: RegistrationConfig<'a>,
    state: PhantomData<(HttpAddress, ServiceAddress, ServiceName)>,
}

impl Registration {
    pub async fn deregister(self) {
        let Ok(client) = reqwest::Client::builder().default_headers(self.headers).build() else {
            return;
        };

        let _ = client
            .put(format!("{}/deregister/{}", self.service_endpoint, self.service_id))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status);
    }
}

impl<'a> RegistrationBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Default for RegistrationBuilder<'a> {
    fn default() -> Self {
        Self {
            config: RegistrationConfig {
                consul_http_addr: None,
                consul_service_address: None,
                service_name: "",
                consul_service_port: 443,
                consul_service_scheme: "https",
                headers: HeaderMap::new(),
            },
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceAddress, ServiceName> RegistrationBuilder<'a, HttpAddress, ServiceAddress, ServiceName> {
    pub fn auth_token(mut self, consul_auth_token: Option<&str>) -> Result<Self, Error> {
        if let Some(token) = consul_auth_token {
            let mut value = HeaderValue::from_str(&format!("Bearer {token}"))?;
            value.set_sensitive(true);
            self.config.headers.insert(AUTHORIZATION, value);
        }

        Ok(self)
    }

    pub fn service_port(mut self, consul_service_port: u16) -> Self {
        self.config.consul_service_port = consul_service_port;
        self
    }

    pub fn service_scheme(mut self, consul_service_scheme: &'a str) -> Self {
        self.config.consul_service_scheme = consul_service_scheme;
        self
    }
}

impl<'a, ServiceAddress, ServiceName> RegistrationBuilder<'a, (), ServiceAddress, ServiceName> {
    pub fn http_addr(
        mut self,
        consul_http_addr: Option<&'a str>,
    ) -> RegistrationBuilder<'a, Option<&'a str>, ServiceAddress, ServiceName> {
        self.config.consul_http_addr = consul_http_addr;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceName> RegistrationBuilder<'a, HttpAddress, (), ServiceName> {
    pub fn service_address(
        mut self,
        consul_service_address: Option<&'a str>,
    ) -> RegistrationBuilder<'a, HttpAddress, Option<&'a str>, ServiceName> {
        self.config.consul_service_address = consul_service_address;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceAddress> RegistrationBuilder<'a, HttpAddress, ServiceAddress, ()> {
    pub fn service_name(
        mut self,
        service_name: &'a str,
    ) -> RegistrationBuilder<'a, HttpAddress, ServiceAddress, &'a str> {
        self.config.service_name = service_name;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a> RegistrationBuilder<'a, Option<&'a str>, Option<&'a str>, &'a str> {
    pub async fn register(self) -> Result<Option<Registration>, Error> {
        let Some(consul_http_addr) = self.config.consul_http_addr.filter(|address| !address.is_empty()) else {
            return Ok(None);
        };

        let Some(consul_service_address) = self.config.consul_service_address.filter(|address| !address.is_empty())
        else {
            return Ok(None);
        };

        let service_endpoint = format!("{}/v1/agent/service", consul_http_addr.trim_end_matches('/'));
        let service_id = format!(
            "{}-{:04x}",
            self.config.service_name,
            uuid::Uuid::new_v4().as_u128() as u16
        );

        let health_endpoint = format!(
            "{}://{}:{}/api/health",
            self.config.consul_service_scheme, consul_service_address, self.config.consul_service_port
        );

        let health_check = HealthCheck {
            http: &health_endpoint,
            interval: "30s",
            timeout: "10s",
        };

        let payload = Service {
            name: self.config.service_name,
            id: &service_id,
            tags: ["prometheus"],
            address: consul_service_address,
            port: self.config.consul_service_port,
            check: health_check,
        };

        reqwest::Client::builder()
            .default_headers(self.config.headers.clone())
            .build()?
            .put(format!("{service_endpoint}/register"))
            .query(&[("replace-existing-checks", "true")])
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(Some(Registration {
            headers: self.config.headers,
            service_endpoint,
            service_id,
        }))
    }
}
