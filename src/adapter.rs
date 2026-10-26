//! What this adapter says it is, and the kit that serves its contracts.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::{media, owner};

use crate::calibre_web_automated::CalibreWebAutomated;

/// Where Calibre-Web-Automated answers, by its service id and the port it listens on.
pub const ADDRESS: &str = "http://calibre-web-automated:8083";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "Calibre-Web-Automated";

/// The Calibre-Web-Automated release this adapter's recordings were taken from.
pub const RELEASE: &str = "V3.1.4";

/// The digest of the image that release's recordings were taken from.
pub const DIGEST: &str = "sha256:577e846f104fd21453ef306eefb4a95dd95b3b9ddd2463a150944494284da0fd";

/// What this adapter says it is: the contracts it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![
            lemonfiber_contract::spoken(owner::claim::CAPABILITY, owner::claim::MAJOR),
            lemonfiber_contract::spoken(media::serve::CAPABILITY, media::serve::MAJOR),
        ],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contracts, served from `calibre_web_automated` under `key`, ready while it answers.
#[must_use]
pub fn kit(key: String, calibre_web_automated: Arc<CalibreWebAutomated>) -> Kit {
    let asked = Arc::clone(&calibre_web_automated);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(owner::claim::served(Arc::clone(&calibre_web_automated)))
        .serving(media::serve::served(calibre_web_automated))
}

#[cfg(test)]
mod tests;
