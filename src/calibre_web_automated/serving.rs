//! Looking over Calibre-Web-Automated's library again. What it has no equivalent of is
//! unsupported.

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::media::TitleKind;
use lemonfiber_ports::service::{
    Allowing, AppKeys, Applied, Astray, Channel, Dated, Declared, Declaring, Encrypting, Failure,
    FiledAt, Fronted, Furnished, Furnishing, Heard, Library, OutsideAccount, OutsideServer,
    Recounting, SeriesHeld, Tuned, Tuning, Upkeep,
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

const NO_LIBRARIES: &str =
    "Calibre-Web-Automated reads the one Calibre library its configuration names";

const NO_OUTSIDE: &str =
    "Calibre-Web-Automated holds its own accounts, and no outside service holds them";

const NO_EVENTS: &str = "Calibre-Web-Automated tells nobody what it adds or what goes wrong";

const NO_LIVE_TV: &str = "Calibre-Web-Automated has no live TV";

const REFRESH: &str = "/cwa-library-refresh";

#[async_trait]
impl Library for CalibreWebAutomated {
    async fn has_item(&self, kind: TitleKind, term: &str) -> Result<bool, Failure> {
        if kind == TitleKind::Books {
            self.has_book(term).await
        } else {
            Ok(false)
        }
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

    async fn astray(&self, _app: &str) -> Result<Vec<Astray>, Failure> {
        Err(Self::unsupported(NO_KEYS))
    }

    async fn account(&self, _app: &str) -> Result<OutsideAccount, Failure> {
        Err(Self::unsupported(NO_OUTSIDE))
    }

    async fn listed_as(&self) -> Result<OutsideServer, Failure> {
        Err(Self::unsupported(NO_OUTSIDE))
    }
}

#[async_trait]
impl Allowing for CalibreWebAutomated {
    async fn allow_origins(&self, _origins: &[String]) -> Result<(), Failure> {
        Err(Self::unsupported(NO_ORIGINS))
    }
}

#[async_trait]
impl Recounting for CalibreWebAutomated {
    async fn events(
        &self,
        _address: Option<&str>,
        _pin: Option<&str>,
        _key: Option<&str>,
        _every: Option<u32>,
    ) -> Result<Option<Heard>, Failure> {
        Err(Self::unsupported(NO_EVENTS))
    }
}

#[async_trait]
impl Tuning for CalibreWebAutomated {
    async fn channels(&self, _member: &str) -> Result<Vec<Channel>, Failure> {
        Err(Self::unsupported(NO_LIVE_TV))
    }

    async fn tune(&self, _member: &str, _channel: &str) -> Result<Option<Tuned>, Failure> {
        Err(Self::unsupported(NO_LIVE_TV))
    }

    async fn untune(&self, _closes_by: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_LIVE_TV))
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

#[async_trait]
impl Furnishing for CalibreWebAutomated {
    async fn furnish(&self, _filed: &[FiledAt]) -> Result<Vec<Furnished>, Failure> {
        Err(Self::unsupported(NO_LIBRARIES))
    }
}
