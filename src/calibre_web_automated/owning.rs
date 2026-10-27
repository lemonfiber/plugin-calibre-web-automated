//! Who holds Calibre-Web-Automated's administrator, and claiming it. A fresh one signs its
//! administrator in with the name and password its project publishes, so anybody can take
//! it: claiming it renames that administrator and gives it the password lemonfiber minted.

use async_trait::async_trait;
use lemonfiber_adapter_kit::endpoint::form_encoded;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Failure, Owner, Ownership};
use serde::Deserialize;

use super::form::{self, Field};
use super::{CalibreWebAutomated, Carried, CSRF};

/// The administrator a fresh Calibre-Web-Automated is made with.
pub(super) const PUBLISHED_NAME: &str = "admin";

/// The password that administrator signs in with until somebody changes it.
pub(super) const PUBLISHED_PASSWORD: &str = "admin123";

const PROFILE: &str = "/me";

const PROFILE_FORM: &str = "name=\"password\"";

const USERS: &str = "/ajax/listusers";

const USERS_MOST: &str = "100";

const SIDEBAR: &str = "/ajax/editlistusers/sidebar_view";

const SHOWN: &str = "show_";

const SHOWN_AT_RANDOM: &str = "Show_detail_random";

const AT_RANDOM: u32 = 1;

const SIDEBAR_BITS: u32 = 18;

#[derive(Deserialize)]
struct Users {
    rows: Vec<User>,
}

#[derive(Deserialize)]
struct User {
    id: u64,
    name: String,
    #[serde(default)]
    sidebar_view: u32,
}

/// The sidebar views `fields` would leave ticked.
pub(super) fn shown(fields: &[Field]) -> u32 {
    fields
        .iter()
        .filter_map(|(name, _)| {
            if name == SHOWN_AT_RANDOM {
                return Some(AT_RANDOM);
            }
            name.strip_prefix(SHOWN)?.parse::<u32>().ok()
        })
        .fold(0, |shown, view| shown | view)
}

/// The views `before` held that a profile posting only `shown` takes away.
pub(super) fn taken(before: u32, shown: u32) -> Vec<u32> {
    (0..SIDEBAR_BITS)
        .map(|bit| 1 << bit)
        .filter(|view| before & view != 0 && shown & view == 0)
        .collect()
}

impl CalibreWebAutomated {
    async fn published(&self, session: &str) -> Result<User, Failure> {
        let response = self
            .endpoint
            .send(&self.request(
                Method::Get,
                &format!(
                    "{USERS}?{}",
                    form_encoded(&[
                        ("offset", "0"),
                        ("limit", USERS_MOST),
                        ("search", PUBLISHED_NAME)
                    ])
                ),
                Carried {
                    session: Some(session),
                    ..Carried::default()
                },
            ))
            .await?;
        let users: Users = self.endpoint.decode(
            &response,
            "the accounts Calibre-Web-Automated holds could not be read",
        )?;
        users
            .rows
            .into_iter()
            .find(|user| user.name.eq_ignore_ascii_case(PUBLISHED_NAME))
            .ok_or_else(|| {
                self.endpoint
                    .refused("Calibre-Web-Automated holds no account under its published name")
            })
    }

    /// Rename the administrator signed in on `session` and give it `password`, in the one
    /// change its profile makes, then tick again the views that change took away.
    async fn renamed(&self, session: &str, name: &str, password: &str) -> Result<(), Failure> {
        let before = self.published(session).await?;
        let signed_in = |form| Carried {
            form,
            session: Some(session),
            csrf: None,
        };
        let page = self
            .endpoint
            .send(&self.request(Method::Get, PROFILE, signed_in(None)))
            .await?;
        self.endpoint.expect_success(&page)?;
        let mut fields = form::fields(&page.body, PROFILE_FORM).ok_or_else(|| {
            self.endpoint
                .refused("Calibre-Web-Automated's profile page carries no form to change it with")
        })?;
        let token = form::value(&fields, CSRF).ok_or_else(|| {
            self.endpoint
                .refused("Calibre-Web-Automated's profile form carries no token")
        })?;
        for (field, value) in &mut fields {
            if field == "name" {
                name.clone_into(value);
            } else if field == "password" {
                password.clone_into(value);
            }
        }
        let posted: Vec<(&str, &str)> = fields
            .iter()
            .map(|(field, value)| (field.as_str(), value.as_str()))
            .collect();
        let answer = self
            .endpoint
            .send(&self.request(
                Method::Post,
                PROFILE,
                signed_in(Some(form_encoded(&posted))),
            ))
            .await?;
        self.endpoint.expect_success(&answer)?;
        if let Some(said) = form::refusal(&answer.body) {
            return Err(self.endpoint.refused(&format!(
                "Calibre-Web-Automated would not take the administrator lemonfiber minted: {said}"
            )));
        }
        let pk = before.id.to_string();
        for view in taken(before.sidebar_view, shown(&fields)) {
            let asked = form_encoded(&[
                ("pk", &pk),
                ("field_index", &view.to_string()),
                ("value", "true"),
            ]);
            let response = self
                .endpoint
                .send(&self.request(
                    Method::Post,
                    SIDEBAR,
                    Carried {
                        form: Some(asked),
                        session: Some(session),
                        csrf: Some(&token),
                    },
                ))
                .await?;
            self.endpoint.expect_success(&response)?;
        }
        Ok(())
    }
}

#[async_trait]
impl Ownership for CalibreWebAutomated {
    async fn owner(&self) -> Result<Owner, Failure> {
        let (name, password) = self.minted()?;
        if let Some(session) = self.signed_in_as(&name, &password).await? {
            self.keep(session);
            return Ok(Owner::Ours);
        }
        if self
            .signed_in_as(PUBLISHED_NAME, PUBLISHED_PASSWORD)
            .await?
            .is_some()
        {
            return Ok(Owner::Open);
        }
        Ok(Owner::Theirs)
    }

    async fn claim(&self) -> Result<Owner, Failure> {
        let (name, password) = self.minted()?;
        if let Some(session) = self.signed_in_as(&name, &password).await? {
            self.keep(session);
            return Ok(Owner::Ours);
        }
        let Some(session) = self
            .signed_in_as(PUBLISHED_NAME, PUBLISHED_PASSWORD)
            .await?
        else {
            return Ok(Owner::Theirs);
        };
        self.renamed(&session, &name, &password).await?;
        let session = self.signed_in_as(&name, &password).await?.ok_or_else(|| {
            self.endpoint.refused(
                "Calibre-Web-Automated took the administrator lemonfiber minted and does not sign it in",
            )
        })?;
        self.keep(session);
        Ok(Owner::Ours)
    }
}
