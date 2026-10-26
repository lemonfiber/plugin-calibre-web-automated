use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Library as _, Owner, Ownership as _};

use super::super::owning::{shown, taken};
use super::{
    answer, calibre, form_of, header, login_page, page, refused_saying, scripted, signed_in_as,
    wrong_password, Script, PASSWORD, USERNAME,
};

const PROFILE: &str = r#"<form role="form" method="POST"><input type="hidden" name="csrf_token" value="the-profile-token"><input type="text" name="name" value="admin"><input type="email" name="email" value="admin@example.org"><input type="password" name="password" value=""><input type="checkbox" name="show_16" checked><input type="checkbox" name="show_2" checked><input type="checkbox" name="Show_detail_random" checked><select name="locale"><option value="en" selected>English</option></select></form>"#;

const USERS: &str = r#"{"rows":[{"id":4,"name":"administrator","sidebar_view":1},{"id":1,"name":"Admin","role":479,"sidebar_view":3603}]}"#;

const UPDATED: &str = r#"<div class="alert alert-success">Success! Profile Updated</div>"#;

/// A fresh Calibre-Web-Automated: the published administrator signs in, and the minted one
/// signs in as `minted` answers.
fn published(minted: Vec<lemonfiber_ports::http::Response>, rest: Script) -> Script {
    let mut routes: Script = vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (
            Method::Post,
            "/login",
            Some("username=admin&password=admin123"),
            vec![signed_in_as("published")],
        ),
        (Method::Post, "/login", None, minted),
    ];
    routes.extend(rest);
    routes
}

#[test]
fn the_views_a_profile_leaves_ticked_are_its_shown_boxes() {
    let fields: Vec<(String, String)> =
        ["show_16", "show_2", "Show_detail_random", "show_x", "name"]
            .iter()
            .map(|name| ((*name).to_owned(), "on".to_owned()))
            .collect();
    assert_eq!(shown(&fields), 19);
    assert_eq!(taken(3603, 19), [512, 1024, 2048]);
    assert!(taken(19, 19).is_empty());
}

#[tokio::test]
async fn a_claim_renames_the_published_administrator_in_one_change_and_ticks_its_views_again() {
    let http = scripted(published(
        vec![wrong_password(), signed_in_as("ours")],
        vec![
            (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
            (Method::Get, "/me", None, vec![page(PROFILE)]),
            (Method::Post, "/me", None, vec![page(UPDATED)]),
            (
                Method::Post,
                "/ajax/editlistusers/sidebar_view",
                None,
                vec![page("")],
            ),
            (Method::Post, "/cwa-library-refresh", None, vec![page("{}")]),
        ],
    ));
    let calibre = calibre(&http);
    assert_eq!(calibre.claim().await.ok(), Some(Owner::Ours));
    let users = http.sent_to(Method::Get, "/ajax/listusers");
    assert_eq!(
        users.first().map(|one| one.url.as_str()),
        Some("http://calibre-web-automated:8083/ajax/listusers?offset=0&limit=100&search=admin")
    );
    let changed = http.sent_to(Method::Post, "/me");
    let changed = changed.first();
    assert_eq!(
        changed.map(form_of),
        Some(
            [
                ("csrf_token", "the-profile-token"),
                ("name", USERNAME),
                ("email", "admin@example.org"),
                ("password", PASSWORD),
                ("show_16", "on"),
                ("show_2", "on"),
                ("Show_detail_random", "on"),
                ("locale", "en"),
            ]
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect()
        )
    );
    assert_eq!(
        changed.and_then(|one| header(one, "Cookie")).as_deref(),
        Some("session=published")
    );
    let ticked = http.sent_to(Method::Post, "/ajax/editlistusers/sidebar_view");
    assert_eq!(
        ticked.iter().map(form_of).collect::<Vec<_>>(),
        [512, 1024, 2048]
            .iter()
            .map(|view| vec![
                ("pk".to_owned(), "1".to_owned()),
                ("field_index".to_owned(), view.to_string()),
                ("value".to_owned(), "true".to_owned()),
            ])
            .collect::<Vec<_>>()
    );
    assert!(ticked.iter().all(|one| {
        header(one, "X-CSRFToken").as_deref() == Some("the-profile-token")
            && header(one, "Cookie").as_deref() == Some("session=published")
    }));
    assert!(calibre.rescan().await.is_ok());
    assert!(http
        .sent_to(Method::Post, "/cwa-library-refresh")
        .iter()
        .all(|one| header(one, "Cookie").as_deref() == Some("session=ours")));
}

#[tokio::test]
async fn a_claim_changes_nothing_the_minted_administrator_already_holds() {
    let http = scripted(published(vec![signed_in_as("ours")], Vec::new()));
    assert_eq!(calibre(&http).claim().await.ok(), Some(Owner::Ours));
    assert!(http.sent_to(Method::Post, "/me").is_empty());
}

#[tokio::test]
async fn a_claim_whose_password_it_refuses_is_refused_in_its_own_words() {
    let http = scripted(published(
        vec![wrong_password()],
        vec![
            (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
            (Method::Get, "/me", None, vec![page(PROFILE)]),
            (
                Method::Post,
                "/me",
                None,
                vec![page(
                    r#"<div class="alert alert-danger">Password doesn&#39;t comply with password validation rules</div>"#,
                )],
            ),
        ],
    ));
    assert!(refused_saying(
        calibre(&http).claim().await,
        "would not take the administrator lemonfiber minted: Password doesn't comply with password validation rules"
    ));
    assert!(http
        .sent_to(Method::Post, "/ajax/editlistusers/sidebar_view")
        .is_empty());
}

#[tokio::test]
async fn a_claim_taken_that_does_not_sign_in_is_refused() {
    let http = scripted(published(
        vec![wrong_password()],
        vec![
            (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
            (Method::Get, "/me", None, vec![page(PROFILE)]),
            (Method::Post, "/me", None, vec![page(UPDATED)]),
            (
                Method::Post,
                "/ajax/editlistusers/sidebar_view",
                None,
                vec![page("")],
            ),
        ],
    ));
    assert!(refused_saying(
        calibre(&http).claim().await,
        "does not sign it in"
    ));
}

#[tokio::test]
async fn a_claim_it_cannot_make_is_refused() {
    let profile_without = |body: &'static str| -> Script {
        vec![
            (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
            (Method::Get, "/me", None, vec![page(body)]),
        ]
    };
    for (rest, said) in [
        (
            vec![(
                Method::Get,
                "/ajax/listusers",
                None,
                vec![page(r#"{"rows":[{"id":2,"name":"Guest"}]}"#)],
            )],
            "no account under its published name",
        ),
        (
            vec![(Method::Get, "/ajax/listusers", None, vec![page("<html>")])],
            "accounts Calibre-Web-Automated holds could not be read",
        ),
        (profile_without("<html></html>"), "carries no form"),
        (
            profile_without(r#"<form><input name="password"></form>"#),
            "carries no token",
        ),
        (
            vec![
                (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
                (Method::Get, "/me", None, vec![answer(500, &[], "")]),
            ],
            "HTTP 500",
        ),
        (
            vec![
                (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
                (Method::Get, "/me", None, vec![page(PROFILE)]),
                (Method::Post, "/me", None, vec![answer(500, &[], "")]),
            ],
            "HTTP 500",
        ),
        (
            vec![
                (Method::Get, "/ajax/listusers", None, vec![page(USERS)]),
                (Method::Get, "/me", None, vec![page(PROFILE)]),
                (Method::Post, "/me", None, vec![page(UPDATED)]),
                (
                    Method::Post,
                    "/ajax/editlistusers/sidebar_view",
                    None,
                    vec![answer(400, &[], "Malformed request")],
                ),
            ],
            "HTTP 400",
        ),
    ] {
        let http = scripted(published(vec![wrong_password()], rest));
        assert!(refused_saying(calibre(&http).claim().await, said), "{said}");
    }
}
