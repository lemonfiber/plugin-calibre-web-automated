use super::super::form::{fields, refusal, unescaped, value};

const PROFILE: &str = r#"<form class="search"><input name="query" value="q"></form>
<form role="form" method="POST" autocomplete="off">
  <input type="hidden" name="csrf_token" value="the-token">
  <input type="text" class="form-control" name="name" id="name" value="admin">
  <input type="email" name="email" value="a&amp;b&#39;c@example.org">
  <input type="password" name="password" value="">
  <input type=text name=kindle_mail value=k@example.org>
  <input type="checkbox" name="show_16" checked>
  <input type="checkbox" name="show_32">
  <input type='checkbox' name='show_2' value='yes' checked='checked'>
  <input type="radio" name="theme_choice" value="dark">
  <input type="submit" name="submit" value="Save">
  <input value="nameless">
  <select name="locale"><option value="de">Deutsch</option><option value="en" selected>English</option></select>
  <select name="default_language"><option value="all">All</option><option value="eng">English</option></select>
  <select name="empty"></select>
  <textarea name="note">kept out</textarea>
  <img src="x" alt="1 < 2">
</form>"#;

fn as_posted(found: Option<Vec<(String, String)>>) -> Vec<(String, String)> {
    found.unwrap_or_default()
}

#[test]
fn the_form_holding_the_marker_is_read_as_a_browser_posts_it() {
    let posted = as_posted(fields(PROFILE, r#"name="password""#));
    let expected: Vec<(String, String)> = [
        ("csrf_token", "the-token"),
        ("name", "admin"),
        ("email", "a&b'c@example.org"),
        ("password", ""),
        ("kindle_mail", "k@example.org"),
        ("show_16", "on"),
        ("show_2", "yes"),
        ("locale", "en"),
        ("default_language", "all"),
    ]
    .iter()
    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
    .collect();
    assert_eq!(posted, expected);
    assert_eq!(value(&posted, "csrf_token").as_deref(), Some("the-token"));
    assert_eq!(value(&posted, "absent"), None);
}

#[test]
fn a_page_with_no_form_holding_the_marker_has_none() {
    assert_eq!(fields(PROFILE, r#"name="nowhere""#), None);
    assert_eq!(fields("<html></html>", "x"), None);
}

#[test]
fn a_tag_left_open_is_read_as_far_as_it_goes() {
    let posted = as_posted(fields(
        r#"<form x><input name="a" value="1"><input name="b" value="2"#,
        r#"name="a""#,
    ));
    assert_eq!(posted, [("a".to_owned(), "1".to_owned())]);
    let unclosed = as_posted(fields(
        r#"<form x><input name="a" value="1" "#,
        r#"name="a""#,
    ));
    assert!(unclosed.is_empty());
}

#[test]
fn escaped_text_is_written_out_once() {
    assert_eq!(
        unescaped("&lt;b&gt; &quot;x&#34; &#39;y&#x27; &amp;amp;"),
        "<b> \"x\" 'y' &amp;"
    );
}

#[test]
fn what_a_page_says_went_wrong_is_its_error_alert() {
    assert_eq!(
        refusal(r#"<div class="alert alert-danger">Password doesn&#39;t comply</div>"#).as_deref(),
        Some("Password doesn't comply")
    );
    assert_eq!(refusal(r#"<div class="alert alert-danger"> </div>"#), None);
    assert_eq!(
        refusal(r#"<div class="alert alert-success">Saved</div>"#),
        None
    );
}
