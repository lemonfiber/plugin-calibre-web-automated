//! The forms Calibre-Web-Automated's pages carry, read as a browser would post them, and
//! what a page says went wrong.

/// The header Calibre-Web-Automated reads a form's token from where no form carries it.
pub(super) const CSRF_HEADER: &str = "X-CSRFToken";

const FORM_OPENS: &str = "<form";

const FORM_CLOSES: &str = "</form>";

const SELECT_CLOSES: &str = "</select>";

const OPTION: &str = "option";

const REFUSED: &str = "alert-danger\">";

const ESCAPED: [(&str, &str); 7] = [
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&#34;", "\""),
    ("&#39;", "'"),
    ("&#x27;", "'"),
    ("&amp;", "&"),
];

/// One field a form posts: its name, and what it would post.
pub(super) type Field = (String, String);

struct Tag<'a> {
    name: String,
    attributes: Vec<(String, Option<String>)>,
    after: &'a str,
}

impl Tag<'_> {
    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(held, _)| held == name)
            .map(|(_, value)| value.as_deref().unwrap_or_default())
    }

    fn holds(&self, name: &str) -> bool {
        self.attributes.iter().any(|(held, _)| held == name)
    }
}

/// `text` with the character references Calibre-Web-Automated's pages escape written out.
pub(super) fn unescaped(text: &str) -> String {
    ESCAPED
        .iter()
        .fold(text.to_owned(), |text, (escaped, plain)| {
            text.replace(escaped, plain)
        })
}

/// The tag `text` opens with, its attributes, and what follows it.
fn tag(text: &str) -> Option<Tag<'_>> {
    let inside = text.strip_prefix('<')?;
    let named = inside
        .find(|mark: char| mark.is_whitespace() || mark == '>' || mark == '/')
        .unwrap_or(inside.len());
    let (name, mut rest) = inside.split_at(named);
    let mut attributes = Vec::new();
    loop {
        rest = rest.trim_start_matches(|mark: char| mark.is_whitespace() || mark == '/');
        if let Some(after) = rest.strip_prefix('>') {
            return Some(Tag {
                name: name.to_ascii_lowercase(),
                attributes,
                after,
            });
        }
        let ends = rest.find(|mark: char| mark.is_whitespace() || mark == '=' || mark == '>')?;
        let (attribute, after) = rest.split_at(ends);
        let attribute = attribute.to_ascii_lowercase();
        let Some(valued) = after.trim_start().strip_prefix('=') else {
            attributes.push((attribute, None));
            rest = after;
            continue;
        };
        let valued = valued.trim_start();
        let (value, after) = if let Some(quote @ ('"' | '\'')) = valued.chars().next() {
            let quoted = valued.get(1..)?;
            let closes = quoted.find(quote)?;
            (quoted.get(..closes)?, quoted.get(closes + 1..)?)
        } else {
            let ends = valued
                .find(|mark: char| mark.is_whitespace() || mark == '>')
                .unwrap_or(valued.len());
            valued.split_at(ends)
        };
        attributes.push((attribute, Some(unescaped(value))));
        rest = after;
    }
}

/// Every tag in `text`, in order.
fn tags(text: &str) -> Vec<Tag<'_>> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('<') {
        let Some(from) = rest.get(at..) else { break };
        match tag(from) {
            Some(read) => {
                rest = read.after;
                found.push(read);
            }
            None => rest = from.get(1..).unwrap_or_default(),
        }
    }
    found
}

fn input(read: &Tag<'_>) -> Option<Field> {
    let name = read.attribute("name")?.to_owned();
    let value = read.attribute("value").unwrap_or_default().to_owned();
    match read.attribute("type").unwrap_or("text") {
        "checkbox" | "radio" if read.holds("checked") => Some((
            name,
            if value.is_empty() {
                "on".to_owned()
            } else {
                value
            },
        )),
        "checkbox" | "radio" | "submit" | "button" | "file" | "image" | "reset" => None,
        _ => Some((name, value)),
    }
}

fn select(read: &Tag<'_>) -> Option<Field> {
    let name = read.attribute("name")?.to_owned();
    let options = read.after.split(SELECT_CLOSES).next().unwrap_or_default();
    let offered: Vec<Tag<'_>> = tags(options)
        .into_iter()
        .filter(|one| one.name == OPTION)
        .collect();
    offered
        .iter()
        .find(|one| one.holds("selected"))
        .or_else(|| offered.first())
        .map(|chosen| {
            (
                name,
                chosen.attribute("value").unwrap_or_default().to_owned(),
            )
        })
}

/// The fields of the first form in `page` that holds `marker`, as a browser would post them:
/// every field with its value, a box only where it is ticked, and a list at its choice.
pub(super) fn fields(page: &str, marker: &str) -> Option<Vec<Field>> {
    page.split(FORM_OPENS)
        .skip(1)
        .filter_map(|opened| opened.split(FORM_CLOSES).next())
        .find(|form| form.contains(marker))
        .map(|form| {
            tags(&format!("{FORM_OPENS}{form}"))
                .iter()
                .filter_map(|read| match read.name.as_str() {
                    "input" => input(read),
                    "select" => select(read),
                    _ => None,
                })
                .collect()
        })
}

/// What `fields` posts under `name`.
pub(super) fn value(fields: &[Field], name: &str) -> Option<String> {
    fields
        .iter()
        .find(|(held, _)| held == name)
        .map(|(_, value)| value.clone())
}

/// What `page` says went wrong, where it says anything.
pub(super) fn refusal(page: &str) -> Option<String> {
    let (_, after) = page.split_once(REFUSED)?;
    let said = unescaped(after.split('<').next().unwrap_or_default().trim());
    Some(said).filter(|said| !said.is_empty())
}
