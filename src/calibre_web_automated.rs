//! Calibre-Web-Automated, the ebook library, asked through its own web application as the
//! administrator lemonfiber minted for this adapter, signed in with a session.

use std::sync::{Arc, Mutex};

use lemonfiber_adapter_kit::credential::Holding;
use lemonfiber_adapter_kit::endpoint::{form_encoded, Endpoint};
use lemonfiber_ports::http::{Fetched, Http, Method, Request, Response};
use lemonfiber_ports::service::Failure;

mod form;
mod owning;
mod screening;
mod serving;

/// What every failure is blamed on.
pub const SERVICE: &str = "calibre-web-automated";

const LOGIN: &str = "/login";

const SESSION: &str = "session";

const SET_COOKIE: &str = "set-cookie";

const COOKIE: &str = "Cookie";

const LOCATION: &str = "location";

const CONTENT_TYPE: &str = "Content-Type";

const FORM: &str = "application/x-www-form-urlencoded";

const CSRF: &str = "csrf_token";

const SIGN_IN_FORM: &str = "name=\"username\"";

const SIGNED_IN: u16 = 302;

const NO_ADMIN: &str = "this adapter holds no administrator for Calibre-Web-Automated";

/// A client for one Calibre-Web-Automated, signed in as the administrator `username` and
/// `password` supply, holding the session it opened until Calibre-Web-Automated ends it.
pub struct CalibreWebAutomated {
    http: Arc<dyn Http>,
    endpoint: Endpoint,
    username: Arc<dyn Holding>,
    password: Arc<dyn Holding>,
    session: Mutex<Option<String>>,
}

/// What a request carries besides its method and path.
#[derive(Default)]
struct Carried<'a> {
    form: Option<String>,
    session: Option<&'a str>,
    csrf: Option<&'a str>,
}

impl CalibreWebAutomated {
    /// The Calibre-Web-Automated reached at `base` over `http`, signed in as the
    /// administrator `username` and `password` supply.
    #[must_use]
    pub fn new(
        http: Arc<dyn Http>,
        base: impl Into<String>,
        username: Arc<dyn Holding>,
        password: Arc<dyn Holding>,
    ) -> Self {
        Self {
            endpoint: Endpoint::new(Arc::clone(&http), base, SERVICE),
            http,
            username,
            password,
            session: Mutex::new(None),
        }
    }

    /// Whether Calibre-Web-Automated serves its sign-in page, which it serves to anybody.
    pub async fn answers(&self) -> bool {
        self.endpoint
            .send(&self.request(Method::Get, LOGIN, Carried::default()))
            .await
            .is_ok_and(|response| response.is_success())
    }

    fn minted(&self) -> Result<(String, String), Failure> {
        let held = |holding: &Arc<dyn Holding>| {
            holding
                .key()
                .map_err(|why| self.endpoint.refused(&format!("{NO_ADMIN}: {why}")))
        };
        Ok((held(&self.username)?, held(&self.password)?))
    }

    fn request(&self, method: Method, path: &str, carried: Carried<'_>) -> Request {
        let mut headers = Vec::new();
        if let Some(session) = carried.session {
            headers.push((COOKIE.to_owned(), format!("{SESSION}={session}")));
        }
        if let Some(token) = carried.csrf {
            headers.push((form::CSRF_HEADER.to_owned(), token.to_owned()));
        }
        if carried.form.is_some() {
            headers.push((CONTENT_TYPE.to_owned(), FORM.to_owned()));
        }
        Request {
            method,
            url: self.endpoint.url(path),
            headers,
            body: carried.form,
            pinned: None,
        }
    }

    /// The session Calibre-Web-Automated opens for `name` and `password`, or nothing where
    /// it does not sign them in.
    async fn signed_in_as(&self, name: &str, password: &str) -> Result<Option<String>, Failure> {
        let page = self
            .endpoint
            .send(&self.request(Method::Get, LOGIN, Carried::default()))
            .await?;
        self.endpoint.expect_success(&page)?;
        let opened = session_in(&page.headers).ok_or_else(|| {
            self.endpoint
                .refused("Calibre-Web-Automated opened no session to sign in with")
        })?;
        let token = form::fields(&page.body, SIGN_IN_FORM)
            .and_then(|fields| form::value(&fields, CSRF))
            .ok_or_else(|| {
                self.endpoint.refused(
                    "Calibre-Web-Automated's sign-in page carries no token to sign in with",
                )
            })?;
        let asked = form_encoded(&[
            (CSRF, &token),
            ("username", name),
            ("password", password),
            ("next", "/"),
        ]);
        let answer = self
            .endpoint
            .send(&self.request(
                Method::Post,
                LOGIN,
                Carried {
                    form: Some(asked),
                    session: Some(&opened),
                    csrf: None,
                },
            ))
            .await?;
        if answer.status == SIGNED_IN && !signed_out(&answer) {
            return Ok(Some(session_in(&answer.headers).unwrap_or(opened)));
        }
        if answer.is_success() || signed_out(&answer) {
            return Ok(None);
        }
        Err(self.endpoint.refusal(&answer))
    }

    fn keep(&self, session: String) {
        if let Ok(mut held) = self.session.lock() {
            held.replace(session);
        }
    }

    async fn opened(&self) -> Result<String, Failure> {
        let (name, password) = self.minted()?;
        let session = self
            .signed_in_as(&name, &password)
            .await?
            .ok_or_else(|| self.endpoint.unauthorised())?;
        self.keep(session.clone());
        Ok(session)
    }

    async fn session(&self) -> Result<String, Failure> {
        let held = self.session.lock().ok().and_then(|held| held.clone());
        match held {
            Some(session) => Ok(session),
            None => self.opened().await,
        }
    }

    async fn signed_in(
        &self,
        method: Method,
        path: &str,
        form: Option<String>,
    ) -> Result<Response, Failure> {
        let session = self.session().await?;
        let carried = |session| Carried {
            form: form.clone(),
            session: Some(session),
            csrf: None,
        };
        let response = self
            .endpoint
            .send(&self.request(method, path, carried(&session)))
            .await?;
        if !signed_out(&response) {
            return Ok(response);
        }
        let session = self.opened().await?;
        self.endpoint
            .send(&self.request(method, path, carried(&session)))
            .await
    }

    async fn read<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        what: &str,
    ) -> Result<T, Failure> {
        let response = self.signed_in(Method::Get, path, None).await?;
        self.endpoint.decode(&response, what)
    }

    async fn fetched_signed_in(&self, path: &str, most: usize) -> Result<Fetched, Failure> {
        let session = self.session().await?;
        let fetched = self.fetched(path, &session, most).await?;
        if !fetched_signed_out(&fetched) {
            return Ok(fetched);
        }
        let session = self.opened().await?;
        self.fetched(path, &session, most).await
    }

    async fn fetched(&self, path: &str, session: &str, most: usize) -> Result<Fetched, Failure> {
        let carried = Carried {
            session: Some(session),
            ..Carried::default()
        };
        self.http
            .fetch(&self.request(Method::Get, path, carried), most)
            .await
            .map_err(|_| Failure::Unavailable {
                service: SERVICE.to_owned(),
            })
    }

    fn unsupported(detail: &str) -> Failure {
        Failure::Unsupported {
            service: SERVICE.to_owned(),
            detail: detail.to_owned(),
        }
    }
}

/// The session a response sets, where it sets one that is not empty.
fn session_in(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case(SET_COOKIE))
        .filter_map(|(_, cookie)| cookie.split(';').next())
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, value)| *name == SESSION && !value.is_empty())
        .map(|(_, value)| value.to_owned())
}

fn sent_to_sign_in(status: u16, headers: &[(String, String)]) -> bool {
    status == SIGNED_IN
        && headers
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case(LOCATION) && value.contains(LOGIN))
}

/// Whether Calibre-Web-Automated sent the request to its sign-in page.
fn signed_out(response: &Response) -> bool {
    sent_to_sign_in(response.status, &response.headers)
}

fn fetched_signed_out(fetched: &Fetched) -> bool {
    sent_to_sign_in(fetched.status, &fetched.headers)
}

#[cfg(test)]
mod tests;
