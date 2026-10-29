//! Looking over Calibre-Web-Automated's library again. What it has no equivalent of is
//! unsupported.

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::media::Kind;
use lemonfiber_ports::service::{
    AppKeys, Applied, Dated, Declared, Declaring, Encrypting, Failure, Fronted, Library,
    SeriesHeld, Upkeep,
};

use super::CalibreWebAutomated;

const NO_KINDS: &str = "Calibre-Web-Automated holds neither films nor series";

const NO_REFRESH: &str = "Calibre-Web-Automated looks over its library as a whole, never one item";

const NO_KEYS: &str = "Calibre-Web-Automated holds no keys an app is given";

const NO_PROXIES: &str =
    "Calibre-Web-Automated holds no list of addresses trusted to name the client";

const NO_ORIGINS: &str =
    "Calibre-Web-Automated holds no list of origins a browser may read it from";

const NO_RESTART: &str = "Calibre-Web-Automated is not asked to start again by this adapter";

const NO_GUARD: &str = "Calibre-Web-Automated serves no member through a guard of lemonfiber's";

const NO_TLS: &str = "Calibre-Web-Automated serves no internal TLS port";

const REFRESH: &str = "/cwa-library-refresh";

#[async_trait]
impl Library for CalibreWebAutomated {
    async fn has_item(&self, _kind: Kind, _term: &str) -> Result<bool, Failure> {
        Err(Self::unsupported(NO_KINDS))
    }

    async fn rescan(&self) -> Result<(), Failure> {
        let response = self.signed_in(Method::Post, REFRESH, None).await?;
        self.endpoint.expect_success(&response)
    }
}

#[async_trait]
impl Upkeep for CalibreWebAutomated {
    async fn series_held(&self, _most: u32) -> Result<Vec<SeriesHeld>, Failure> {
        Err(Self::unsupported(NO_KINDS))
    }

    async fn refresh(&self, _id: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_REFRESH))
    }
}

#[async_trait]
impl AppKeys for CalibreWebAutomated {
    async fn filed_as(&self, _app: &str) -> Result<Vec<String>, Failure> {
        Err(Self::unsupported(NO_KEYS))
    }

    async fn mint(&self, _app: &str) -> Result<String, Failure> {
        Err(Self::unsupported(NO_KEYS))
    }

    async fn dated(&self, _app: &str) -> Result<Vec<Dated>, Failure> {
        Err(Self::unsupported(NO_KEYS))
    }

    async fn answers_to(&self, _key: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_KEYS))
    }

    async fn revoke(&self, _key: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_KEYS))
    }
}

#[async_trait]
impl Fronted for CalibreWebAutomated {
    async fn known_proxies(&self) -> Result<Vec<String>, Failure> {
        Err(Self::unsupported(NO_PROXIES))
    }

    async fn trust_only(&self, _address: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_PROXIES))
    }

    async fn allowed_origins(&self) -> Result<Vec<String>, Failure> {
        Err(Self::unsupported(NO_ORIGINS))
    }

    async fn allow_only(&self, _origin: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_ORIGINS))
    }

    async fn restart(&self) -> Result<(), Failure> {
        Err(Self::unsupported(NO_RESTART))
    }
}

#[async_trait]
impl Declaring for CalibreWebAutomated {
    async fn declared(&self) -> Result<Declared, Failure> {
        Err(Self::unsupported(NO_GUARD))
    }
}

#[async_trait]
impl Encrypting for CalibreWebAutomated {
    async fn serve_tls(&self, _path: &str, _pin: &str) -> Result<Applied, Failure> {
        Err(Self::unsupported(NO_TLS))
    }
}
