use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lemonfiber_ports::http::{Http, Method, Request, Response, Unreachable};
use lemonfiber_ports::service::{Failure, Library as _, Owner, Ownership as _};

use super::{session_in, CalibreWebAutomated};

mod form;
mod owning;
mod screening;

const BASE: &str = "http://calibre-web-automated:8083";

const USERNAME: &str = "qrstuvwx";

const PASSWORD: &str = "Ab-cdefghijklmnop1";

const LOGIN_PAGE: &str = r#"<html><form class="search"><input name="query"></form><form method="POST" action="/login"><input type="hidden" name="csrf_token" value="the-login-token"><input type="text" name="username"><input type="password" name="password"><input type="checkbox" name="remember_me"></form></html>"#;

struct Route {
    method: Method,
    fragment: &'static str,
    body: Option<&'static str>,
    answers: Mutex<VecDeque<Response>>,
}

/// A transport answering each route from its own script, the last answer repeating, with
/// the headers a session and a redirect travel in.
struct Scripted {
    routes: Vec<Route>,
    seen: Mutex<Vec<Request>>,
}

#[async_trait]
impl Http for Scripted {
    async fn send(&self, request: &Request) -> Result<Response, Unreachable> {
        if let Ok(mut seen) = self.seen.lock() {
            seen.push(request.clone());
        }
        let route = self.routes.iter().find(|route| {
            route.method == request.method
                && request.url.contains(route.fragment)
                && route.body.is_none_or(|wanted| {
                    request
                        .body
                        .as_deref()
                        .is_some_and(|body| body.contains(wanted))
                })
        });
        let answer = route.and_then(|route| {
            route.answers.lock().ok().and_then(|mut answers| {
                if answers.len() > 1 {
                    answers.pop_front()
                } else {
                    answers.front().cloned()
                }
            })
        });
        answer.ok_or_else(|| Unreachable {
            url: request.url.clone(),
            reason: "nothing scripted".to_owned(),
            attempts: 1,
            connected: false,
        })
    }
}

impl Scripted {
    fn requests(&self) -> Vec<Request> {
        self.seen
            .lock()
            .map(|seen| seen.clone())
            .unwrap_or_default()
    }

    fn asked(&self) -> Vec<(Method, String)> {
        self.requests()
            .into_iter()
            .map(|one| {
                let path = one.url.trim_start_matches(BASE);
                let path = path.split_once('?').map_or(path, |(path, _)| path);
                (one.method, path.to_owned())
            })
            .collect()
    }

    fn sent_to(&self, method: Method, fragment: &str) -> Vec<Request> {
        self.requests()
            .into_iter()
            .filter(|one| one.method == method && one.url.contains(fragment))
            .collect()
    }
}

fn answer(status: u16, headers: &[(&str, &str)], body: &str) -> Response {
    Response {
        status,
        headers: headers
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect(),
        body: body.to_owned(),
    }
}

fn page(body: &str) -> Response {
    answer(200, &[], body)
}

fn login_page() -> Response {
    answer(
        200,
        &[(
            "Set-Cookie",
            "session=opened; HttpOnly; Path=/; SameSite=Lax",
        )],
        LOGIN_PAGE,
    )
}

fn signed_in_as(session: &str) -> Response {
    answer(
        302,
        &[
            ("Location", "/"),
            (
                "Set-Cookie",
                &format!("session={session}; HttpOnly; Path=/"),
            ),
        ],
        "",
    )
}

fn sent_to_sign_in() -> Response {
    answer(302, &[("Location", "/login?next=%2F")], "")
}

fn wrong_password() -> Response {
    page(r#"<div class="alert alert-danger">Wrong Username or Password</div>"#)
}

type Script = Vec<(Method, &'static str, Option<&'static str>, Vec<Response>)>;

fn scripted(routes: Script) -> Arc<Scripted> {
    Arc::new(Scripted {
        routes: routes
            .into_iter()
            .map(|(method, fragment, body, answers)| Route {
                method,
                fragment,
                body,
                answers: Mutex::new(answers.into()),
            })
            .collect(),
        seen: Mutex::new(Vec::new()),
    })
}

/// A Calibre-Web-Automated that signs the minted administrator in, and answers the rest
/// of `routes`.
fn signing_in(mut routes: Script) -> Arc<Scripted> {
    routes.insert(0, (Method::Get, "/login", None, vec![login_page()]));
    routes.insert(
        1,
        (Method::Post, "/login", None, vec![signed_in_as("held")]),
    );
    scripted(routes)
}

fn calibre(http: &Arc<Scripted>) -> CalibreWebAutomated {
    CalibreWebAutomated::new(
        http.clone(),
        BASE,
        Arc::new(USERNAME.to_owned()),
        Arc::new(PASSWORD.to_owned()),
    )
}

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers
        .iter()
        .find(|(held, _)| held == name)
        .map(|(_, value)| value.clone())
}

fn form_of(request: &Request) -> Vec<(String, String)> {
    form_urlencoded::parse(request.body.as_deref().unwrap_or_default().as_bytes())
        .into_owned()
        .collect()
}

fn refused_saying(outcome: Result<impl std::fmt::Debug, Failure>, said: &str) -> bool {
    matches!(outcome, Err(Failure::Refused { detail, .. }) if detail.contains(said))
}

struct Missing;

impl lemonfiber_adapter_kit::credential::Holding for Missing {
    fn key(&self) -> Result<String, String> {
        Err("upstream.json could not be read: not found".to_owned())
    }
}

#[tokio::test]
async fn it_answers_while_it_serves_its_sign_in_page() {
    let up = scripted(vec![(Method::Get, "/login", None, vec![login_page()])]);
    assert!(calibre(&up).answers().await);
    assert_eq!(up.asked(), [(Method::Get, "/login".to_owned())]);
    for down in [
        scripted(Vec::new()),
        scripted(vec![(
            Method::Get,
            "/login",
            None,
            vec![answer(500, &[], "")],
        )]),
    ] {
        assert!(!calibre(&down).answers().await);
    }
}

#[tokio::test]
async fn a_session_is_signed_in_with_the_page_s_token_and_carried_on_every_read_after() {
    let http = signing_in(vec![(
        Method::Post,
        "/cwa-library-refresh",
        None,
        vec![page("{}")],
    )]);
    let calibre = calibre(&http);
    assert!(calibre.rescan().await.is_ok());
    assert!(calibre.rescan().await.is_ok());
    assert_eq!(
        http.asked(),
        [
            (Method::Get, "/login".to_owned()),
            (Method::Post, "/login".to_owned()),
            (Method::Post, "/cwa-library-refresh".to_owned()),
            (Method::Post, "/cwa-library-refresh".to_owned()),
        ]
    );
    let signed = http.sent_to(Method::Post, "/login");
    let signed = signed.first();
    assert_eq!(
        signed.map(form_of),
        Some(vec![
            ("csrf_token".to_owned(), "the-login-token".to_owned()),
            ("username".to_owned(), USERNAME.to_owned()),
            ("password".to_owned(), PASSWORD.to_owned()),
            ("next".to_owned(), "/".to_owned()),
        ])
    );
    assert_eq!(
        signed.and_then(|one| header(one, "Cookie")).as_deref(),
        Some("session=opened")
    );
    assert_eq!(
        signed
            .and_then(|one| header(one, "Content-Type"))
            .as_deref(),
        Some("application/x-www-form-urlencoded")
    );
    assert!(http
        .sent_to(Method::Post, "/cwa-library-refresh")
        .iter()
        .all(|one| header(one, "Cookie").as_deref() == Some("session=held")));
}

#[tokio::test]
async fn a_sign_in_that_sets_no_new_session_keeps_the_one_it_was_made_on() {
    let http = scripted(vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (
            Method::Post,
            "/login",
            None,
            vec![answer(302, &[("Location", "/")], "")],
        ),
        (Method::Post, "/cwa-library-refresh", None, vec![page("{}")]),
    ]);
    assert!(calibre(&http).rescan().await.is_ok());
    assert!(http
        .sent_to(Method::Post, "/cwa-library-refresh")
        .iter()
        .all(|one| header(one, "Cookie").as_deref() == Some("session=opened")));
}

#[tokio::test]
async fn a_session_it_ends_is_signed_in_again_once() {
    let http = scripted(vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (
            Method::Post,
            "/login",
            None,
            vec![signed_in_as("first"), signed_in_as("second")],
        ),
        (
            Method::Post,
            "/cwa-library-refresh",
            None,
            vec![sent_to_sign_in(), page("{}")],
        ),
    ]);
    assert!(calibre(&http).rescan().await.is_ok());
    let refreshed = http.sent_to(Method::Post, "/cwa-library-refresh");
    assert_eq!(
        refreshed
            .iter()
            .map(|one| header(one, "Cookie"))
            .collect::<Vec<_>>(),
        [
            Some("session=first".to_owned()),
            Some("session=second".to_owned())
        ]
    );
}

#[tokio::test]
async fn an_administrator_it_will_not_sign_in_is_unauthorised() {
    let http = scripted(vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (Method::Post, "/login", None, vec![wrong_password()]),
    ]);
    assert!(matches!(
        calibre(&http).rescan().await,
        Err(Failure::Unauthorised { .. })
    ));
}

#[tokio::test]
async fn a_sign_in_page_it_cannot_sign_in_from_is_refused() {
    for (login, said) in [
        (page(LOGIN_PAGE), "opened no session"),
        (
            answer(200, &[("Set-Cookie", "session=; Path=/")], LOGIN_PAGE),
            "opened no session",
        ),
        (
            answer(200, &[("Set-Cookie", "session=opened")], "<html></html>"),
            "carries no token",
        ),
        (answer(500, &[], ""), "HTTP 500"),
    ] {
        let http = scripted(vec![(Method::Get, "/login", None, vec![login])]);
        assert!(
            refused_saying(calibre(&http).rescan().await, said),
            "{said}"
        );
    }
    let limited = scripted(vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (Method::Post, "/login", None, vec![answer(429, &[], "")]),
    ]);
    assert!(refused_saying(calibre(&limited).rescan().await, "HTTP 429"));
}

#[tokio::test]
async fn an_administrator_it_does_not_hold_is_refused_before_anything_is_asked() {
    let holdings: [(
        Arc<dyn lemonfiber_adapter_kit::credential::Holding>,
        Arc<dyn lemonfiber_adapter_kit::credential::Holding>,
    ); 2] = [
        (Arc::new(Missing), Arc::new(PASSWORD.to_owned())),
        (Arc::new(USERNAME.to_owned()), Arc::new(Missing)),
    ];
    for (username, password) in holdings {
        let http = scripted(Vec::new());
        let calibre = CalibreWebAutomated::new(http.clone(), BASE, username, password);
        assert!(refused_saying(
            calibre.owner().await,
            "holds no administrator"
        ));
        assert!(refused_saying(
            calibre.rescan().await,
            "holds no administrator"
        ));
        assert!(http.requests().is_empty());
    }
}

#[test]
fn the_session_is_the_cookie_of_that_name_a_response_sets_and_not_empty() {
    let set = |cookies: &[&str]| -> Vec<(String, String)> {
        cookies
            .iter()
            .map(|cookie| ("set-cookie".to_owned(), (*cookie).to_owned()))
            .collect()
    };
    assert_eq!(
        session_in(&set(&["remember_token=x; Path=/", "session=abc; HttpOnly"])),
        Some("abc".to_owned())
    );
    assert_eq!(session_in(&set(&["session=; Expires=0"])), None);
    assert_eq!(session_in(&set(&["other=abc"])), None);
    assert_eq!(
        session_in(&[("Content-Type".to_owned(), "session=abc".to_owned())]),
        None
    );
}

#[tokio::test]
async fn it_is_ours_where_the_minted_administrator_signs_in() {
    let http = signing_in(Vec::new());
    assert_eq!(calibre(&http).owner().await.ok(), Some(Owner::Ours));
    assert_eq!(http.sent_to(Method::Post, "/login").len(), 1);
}

#[tokio::test]
async fn nobody_holds_it_while_the_published_administrator_still_signs_in() {
    let http = scripted(vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (
            Method::Post,
            "/login",
            Some("username=admin&password=admin123"),
            vec![signed_in_as("published")],
        ),
        (Method::Post, "/login", None, vec![wrong_password()]),
    ]);
    assert_eq!(calibre(&http).owner().await.ok(), Some(Owner::Open));
}

#[tokio::test]
async fn it_is_theirs_where_neither_signs_in() {
    for refused in [wrong_password(), sent_to_sign_in()] {
        let http = scripted(vec![
            (Method::Get, "/login", None, vec![login_page()]),
            (Method::Post, "/login", None, vec![refused]),
        ]);
        assert_eq!(calibre(&http).owner().await.ok(), Some(Owner::Theirs));
        assert_eq!(calibre(&http).claim().await.ok(), Some(Owner::Theirs));
    }
}
