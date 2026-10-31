use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{
    Failure, Holds, Image, Item, ItemDetail, Library as _, Medium, Picture, Screening as _,
};

use super::super::screening::{numbered, plain};
use super::{
    answer, calibre, header, login_page, page, refused_saying, scripted, sent_to_sign_in,
    signed_in_as, signing_in,
};

const BOOKS: &str = r#"{"totalNotFiltered":2,"total":2,"rows":[{"id":3,"title":"Bare Book","has_cover":0,"comments":"","tags":""},{"id":2,"title":"Test Book","has_cover":1,"comments":"<p>A book of <b>one</b>\n tone &amp; more</p>","tags":"Ambient, Drone,"}]}"#;

const ONE_BOOK: &str = r#"{"rows":[{"id":2,"title":"Test Book","has_cover":1,"comments":"<p>A book of <b>one</b>\n tone &amp; more</p>","tags":"Ambient, Drone,"}]}"#;

const BARE_BOOK: &str = r#"{"rows":[{"id":3,"title":"Bare Book","has_cover":0}]}"#;

fn book(id: &str, title: &str, poster: bool) -> Item {
    Item {
        id: id.to_owned(),
        title: title.to_owned(),
        year: None,
        medium: Medium::Other,
        holds: Holds {
            poster,
            backdrop: false,
            plays: false,
        },
    }
}

fn not_held() -> lemonfiber_ports::http::Response {
    answer(302, &[("Location", "/")], "")
}

#[tokio::test]
async fn holdings_are_its_books_by_title_as_many_as_asked_for() {
    let http = signing_in(vec![(
        Method::Get,
        "/ajax/listbooks",
        None,
        vec![page(BOOKS)],
    )]);
    assert_eq!(
        calibre(&http).holdings(None, 1).await.ok(),
        Some(vec![book("3", "Bare Book", false)])
    );
    assert_eq!(
        http.sent_to(Method::Get, "/ajax/listbooks")
            .first()
            .map(|one| one.url.clone())
            .as_deref(),
        Some("http://calibre-web-automated:8083/ajax/listbooks?offset=0&limit=1&sort=title&order=asc")
    );
}

#[tokio::test]
async fn holdings_of_none_ask_for_nothing() {
    let http = scripted(Vec::new());
    assert_eq!(
        calibre(&http).holdings(None, 0).await.ok(),
        Some(Vec::new())
    );
    assert!(http.requests().is_empty());
}

#[tokio::test]
async fn a_title_is_one_book_read_once_it_is_known_to_be_held() {
    let http = signing_in(vec![
        (Method::Get, "/book/2", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(ONE_BOOK)]),
    ]);
    assert_eq!(
        calibre(&http).title(None, "2").await.ok().flatten(),
        Some(ItemDetail {
            item: book("2", "Test Book", true),
            overview: Some("A book of one tone & more".to_owned()),
            minutes: None,
            genres: vec!["Ambient".to_owned(), "Drone".to_owned()],
            certificate: None,
            released: None,
            seasons: Vec::new(),
        })
    );
    assert_eq!(
        http.sent_to(Method::Get, "/ajax/listbooks")
            .first()
            .map(|one| one.url.clone())
            .as_deref(),
        Some("http://calibre-web-automated:8083/ajax/listbooks?offset=0&limit=1&sort=state&state=%5B2%5D")
    );
}

#[tokio::test]
async fn a_title_with_nothing_said_of_it_says_nothing() {
    let http = signing_in(vec![
        (Method::Get, "/book/3", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(BARE_BOOK)]),
    ]);
    let detail = calibre(&http).title(None, "3").await.ok().flatten();
    assert_eq!(
        detail.map(|detail| (detail.overview, detail.genres)),
        Some((None, Vec::new()))
    );
}

#[tokio::test]
async fn a_title_it_does_not_hold_is_none() {
    let http = signing_in(vec![(Method::Get, "/book/99", None, vec![not_held()])]);
    assert_eq!(calibre(&http).title(None, "99").await.ok(), Some(None));
    assert!(http.sent_to(Method::Get, "/ajax/listbooks").is_empty());
    let other = signing_in(vec![
        (Method::Get, "/book/2", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(BARE_BOOK)]),
    ]);
    assert_eq!(calibre(&other).title(None, "2").await.ok(), Some(None));
    for id in ["", "two", "../admin", "99999999999999999999999"] {
        let silent = scripted(Vec::new());
        assert_eq!(calibre(&silent).title(None, id).await.ok(), Some(None));
        assert!(silent.requests().is_empty());
    }
}

#[tokio::test]
async fn a_book_page_that_fails_is_a_refusal() {
    let http = signing_in(vec![(
        Method::Get,
        "/book/2",
        None,
        vec![answer(500, &[], "")],
    )]);
    assert!(refused_saying(
        calibre(&http).title(None, "2").await,
        "HTTP 500"
    ));
    let still_out = signing_in(vec![(
        Method::Get,
        "/book/2",
        None,
        vec![sent_to_sign_in()],
    )]);
    assert!(refused_saying(
        calibre(&still_out).title(None, "2").await,
        "HTTP 302"
    ));
}

#[tokio::test]
async fn a_poster_is_the_book_s_cover() {
    let http = signing_in(vec![
        (Method::Get, "/book/2", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(ONE_BOOK)]),
        (
            Method::Get,
            "/cover/2",
            None,
            vec![answer(
                200,
                &[("content-type", "image/gif")],
                "GIF89a-bytes",
            )],
        ),
    ]);
    assert_eq!(
        calibre(&http)
            .picture(None, "2", Picture::Poster)
            .await
            .ok()
            .flatten(),
        Some(Image {
            media_type: "image/gif".to_owned(),
            bytes: b"GIF89a-bytes".to_vec(),
        })
    );
    assert!(http
        .sent_to(Method::Get, "/cover/2")
        .iter()
        .all(|one| header(one, "Cookie").as_deref() == Some("session=held")));
}

#[tokio::test]
async fn a_cover_asked_for_on_an_ended_session_is_asked_for_again_signed_in_afresh() {
    let http = scripted(vec![
        (Method::Get, "/login", None, vec![login_page()]),
        (
            Method::Post,
            "/login",
            None,
            vec![signed_in_as("first"), signed_in_as("second")],
        ),
        (Method::Get, "/book/2", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(ONE_BOOK)]),
        (
            Method::Get,
            "/cover/2",
            None,
            vec![
                sent_to_sign_in(),
                answer(200, &[("content-type", "image/webp")], "RIFF0000WEBPVP8 "),
            ],
        ),
    ]);
    let image = calibre(&http)
        .picture(None, "2", Picture::Poster)
        .await
        .ok()
        .flatten();
    assert_eq!(
        image.map(|image| image.media_type),
        Some("image/webp".to_owned())
    );
}

#[tokio::test]
async fn a_book_with_no_cover_or_a_cover_that_is_no_picture_has_no_poster() {
    let bare = signing_in(vec![
        (Method::Get, "/book/3", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(BARE_BOOK)]),
    ]);
    assert_eq!(
        calibre(&bare)
            .picture(None, "3", Picture::Poster)
            .await
            .ok(),
        Some(None)
    );
    assert!(bare.sent_to(Method::Get, "/cover/").is_empty());
    for (served, body) in [
        ("text/html", "<html>"),
        (
            "image/svg+xml",
            r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#,
        ),
        ("image/png", "<html><script>alert(1)</script></html>"),
    ] {
        let text = signing_in(vec![
            (Method::Get, "/book/2", None, vec![page("<html>")]),
            (Method::Get, "/ajax/listbooks", None, vec![page(ONE_BOOK)]),
            (
                Method::Get,
                "/cover/2",
                None,
                vec![answer(200, &[("content-type", served)], body)],
            ),
        ]);
        assert_eq!(
            calibre(&text)
                .picture(None, "2", Picture::Poster)
                .await
                .ok(),
            Some(None),
            "{served}"
        );
    }
}

#[tokio::test]
async fn a_cover_refused_or_out_of_reach_is_a_failure() {
    let refused = signing_in(vec![
        (Method::Get, "/book/2", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(ONE_BOOK)]),
        (Method::Get, "/cover/2", None, vec![answer(500, &[], "")]),
    ]);
    assert!(refused_saying(
        calibre(&refused).picture(None, "2", Picture::Poster).await,
        "would not answer the cover"
    ));
    let unreachable = signing_in(vec![
        (Method::Get, "/book/2", None, vec![page("<html>")]),
        (Method::Get, "/ajax/listbooks", None, vec![page(ONE_BOOK)]),
    ]);
    assert!(matches!(
        calibre(&unreachable)
            .picture(None, "2", Picture::Poster)
            .await,
        Err(Failure::Unavailable { .. })
    ));
}

#[tokio::test]
async fn a_book_holds_no_backdrop() {
    let http = scripted(Vec::new());
    assert_eq!(
        calibre(&http)
            .picture(None, "2", Picture::Backdrop)
            .await
            .ok(),
        Some(None)
    );
    assert!(http.requests().is_empty());
}

#[tokio::test]
async fn a_library_refresh_it_refuses_is_a_refusal() {
    let http = signing_in(vec![(
        Method::Post,
        "/cwa-library-refresh",
        None,
        vec![answer(500, &[], "")],
    )]);
    assert!(refused_saying(calibre(&http).rescan().await, "HTTP 500"));
}

#[test]
fn a_book_is_numbered_in_digits_alone() {
    assert_eq!(numbered("42"), Some(42));
    assert_eq!(numbered(""), None);
    assert_eq!(numbered("4a"), None);
    assert_eq!(numbered("-1"), None);
}

#[test]
fn what_is_said_of_a_book_is_read_as_text() {
    assert_eq!(
        plain("<p>One &amp; <i>two</i></p>\n<p>three</p>"),
        "One & two three"
    );
    assert_eq!(plain("a > b"), "a > b");
    assert_eq!(plain("<br/>"), "");
}
