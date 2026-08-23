use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::InvalidHeaderValue;
use serde::Serialize;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

pub enum Error {
    InvalidAuthToken(InvalidHeaderValue),
    Request(reqwest::Error),
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAuthToken(error) => Debug::fmt(error, formatter),
            Self::Request(error) => Debug::fmt(error, formatter),
        }
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(error: InvalidHeaderValue) -> Self {
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
    service_id: String,
    consul_service_port: u16,
    consul_service_scheme: &'a str,
    headers: HeaderMap,
}

pub struct RegistrationBuilder<'a, HttpAddress = (), ServiceAddress = (), ServiceName = (), ServiceId = ()> {
    config: RegistrationConfig<'a>,
    state: PhantomData<(HttpAddress, ServiceAddress, ServiceName, ServiceId)>,
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
        Self {
            config: RegistrationConfig {
                consul_http_addr: None,
                consul_service_address: None,
                service_name: "",
                service_id: String::new(),
                consul_service_port: 443,
                consul_service_scheme: "https",
                headers: HeaderMap::new(),
            },
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceAddress, ServiceName, ServiceId>
    RegistrationBuilder<'a, HttpAddress, ServiceAddress, ServiceName, ServiceId>
{
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

impl<'a, ServiceAddress, ServiceName, ServiceId> RegistrationBuilder<'a, (), ServiceAddress, ServiceName, ServiceId> {
    pub fn http_addr(
        mut self,
        consul_http_addr: Option<&'a str>,
    ) -> RegistrationBuilder<'a, Option<&'a str>, ServiceAddress, ServiceName, ServiceId> {
        self.config.consul_http_addr = consul_http_addr;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceName, ServiceId> RegistrationBuilder<'a, HttpAddress, (), ServiceName, ServiceId> {
    pub fn service_address(
        mut self,
        consul_service_address: Option<&'a str>,
    ) -> RegistrationBuilder<'a, HttpAddress, Option<&'a str>, ServiceName, ServiceId> {
        self.config.consul_service_address = consul_service_address;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceAddress, ServiceId> RegistrationBuilder<'a, HttpAddress, ServiceAddress, (), ServiceId> {
    pub fn service_name(
        mut self,
        service_name: &'a str,
    ) -> RegistrationBuilder<'a, HttpAddress, ServiceAddress, &'a str, ServiceId> {
        self.config.service_name = service_name;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a, HttpAddress, ServiceAddress, ServiceName> RegistrationBuilder<'a, HttpAddress, ServiceAddress, ServiceName> {
    pub fn service_id(
        mut self,
        service_id: String,
    ) -> RegistrationBuilder<'a, HttpAddress, ServiceAddress, ServiceName, String> {
        self.config.service_id = service_id;

        RegistrationBuilder {
            config: self.config,
            state: PhantomData,
        }
    }
}

impl<'a> RegistrationBuilder<'a, Option<&'a str>, Option<&'a str>, &'a str, String> {
    pub async fn register(self) -> Result<Option<Registration>, Error> {
        let Some(consul_http_addr) = self.config.consul_http_addr.filter(|address| !address.is_empty()) else {
            return Ok(None);
        };

        let Some(consul_service_address) = self.config.consul_service_address.filter(|address| !address.is_empty())
        else {
            return Ok(None);
        };

        let service_endpoint = format!("{}/v1/agent/service", consul_http_addr.trim_end_matches('/'));

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
            id: &self.config.service_id,
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
            service_id: self.config.service_id,
        }))
    }
}
