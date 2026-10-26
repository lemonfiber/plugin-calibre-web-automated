//! What Calibre-Web-Automated holds, read as its administrator reads it. A member of the
//! household is somebody `identity.source` names, and Calibre-Web-Automated speaks for none
//! of them, so everything asked for one member is unsupported. It plays nothing, and what is
//! being read is not something it says, so what is playing is unsupported too.

use async_trait::async_trait;
use lemonfiber_adapter_kit::endpoint::form_encoded;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{
    Failure, Holds, HowFar, Image, Item, ItemDetail, ItemProgress, Medium, Picture, Playback,
    Screening, PICTURE_MOST,
};
use serde::Deserialize;

use super::form::unescaped;
use super::{signed_out, CalibreWebAutomated, SIGNED_IN};

const NO_MEMBERS: &str =
    "Calibre-Web-Automated speaks for none of the household's accounts, so nothing is asked for one";

const NO_DEVICES: &str = "Calibre-Web-Automated opens and ends no device's session for lemonfiber";

const NO_PLAYING: &str = "Calibre-Web-Automated plays nothing: what it holds is read";

const BOOKS: &str = "/ajax/listbooks";

const BOOK: &str = "/book";

const COVER: &str = "/cover";

const UNREAD: &str = "the books Calibre-Web-Automated holds could not be read";

const TAGS_BETWEEN: char = ',';

const PICTURE_PREFIX: &str = "image/";

#[derive(Deserialize)]
struct Books {
    rows: Vec<Book>,
}

#[derive(Deserialize)]
struct Book {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    has_cover: u8,
    #[serde(default)]
    comments: String,
    #[serde(default)]
    tags: String,
}

impl Book {
    fn item(&self) -> Item {
        Item {
            id: self.id.to_string(),
            title: self.title.clone(),
            year: None,
            medium: Medium::Other,
            holds: Holds {
                poster: self.has_cover > 0,
                backdrop: false,
                plays: false,
            },
        }
    }

    fn detail(self) -> ItemDetail {
        let item = self.item();
        ItemDetail {
            item,
            overview: Some(plain(&self.comments)).filter(|said| !said.is_empty()),
            minutes: None,
            genres: self
                .tags
                .split(TAGS_BETWEEN)
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_owned)
                .collect(),
            certificate: None,
            released: None,
            seasons: Vec::new(),
        }
    }
}

/// `said` with its markup taken out and its whitespace run together, as text.
pub(super) fn plain(said: &str) -> String {
    let mut text = String::new();
    let mut in_markup = false;
    for mark in said.chars() {
        match mark {
            '<' => {
                in_markup = true;
                text.push(' ');
            }
            '>' if in_markup => in_markup = false,
            _ if !in_markup => text.push(mark),
            _ => {}
        }
    }
    unescaped(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The book `id` names, where it names one as Calibre-Web-Automated numbers them.
pub(super) fn numbered(id: &str) -> Option<u64> {
    Some(id)
        .filter(|id| !id.is_empty() && id.bytes().all(|mark| mark.is_ascii_digit()))
        .and_then(|id| id.parse().ok())
}

impl CalibreWebAutomated {
    async fn holds_book(&self, id: u64) -> Result<bool, Failure> {
        let response = self
            .signed_in(Method::Get, &format!("{BOOK}/{id}"), None)
            .await?;
        if response.is_success() {
            return Ok(true);
        }
        if response.status == SIGNED_IN && !signed_out(&response) {
            return Ok(false);
        }
        Err(self.endpoint.refusal(&response))
    }

    async fn book(&self, id: &str) -> Result<Option<Book>, Failure> {
        let Some(id) = numbered(id) else {
            return Ok(None);
        };
        if !self.holds_book(id).await? {
            return Ok(None);
        }
        let asked = form_encoded(&[
            ("offset", "0"),
            ("limit", "1"),
            ("sort", "state"),
            ("state", &format!("[{id}]")),
        ]);
        let books: Books = self.read(&format!("{BOOKS}?{asked}"), UNREAD).await?;
        Ok(books.rows.into_iter().find(|book| book.id == id))
    }

    async fn poster(&self, id: &str) -> Result<Option<Image>, Failure> {
        let Some(book) = self.book(id).await?.filter(|book| book.has_cover > 0) else {
            return Ok(None);
        };
        let fetched = self
            .fetched_signed_in(&format!("{COVER}/{}", book.id), PICTURE_MOST)
            .await?;
        if !fetched.is_success() {
            return Err(self
                .endpoint
                .refused("Calibre-Web-Automated would not answer the cover"));
        }
        let Some(media_type) = fetched
            .header("content-type")
            .filter(|kind| kind.starts_with(PICTURE_PREFIX))
            .map(str::to_owned)
        else {
            return Ok(None);
        };
        Ok(fetched.bytes.map(|bytes| Image { media_type, bytes }))
    }

    async fn books(&self, most: u32) -> Result<Vec<Item>, Failure> {
        if most == 0 {
            return Ok(Vec::new());
        }
        let asked = form_encoded(&[
            ("offset", "0"),
            ("limit", &most.to_string()),
            ("sort", "title"),
            ("order", "asc"),
        ]);
        let books: Books = self.read(&format!("{BOOKS}?{asked}"), UNREAD).await?;
        Ok(books
            .rows
            .iter()
            .take(usize::try_from(most).unwrap_or(usize::MAX))
            .map(Book::item)
            .collect())
    }
}

#[async_trait]
impl Screening for CalibreWebAutomated {
    async fn signed_in(&self, _member: &str, _device: &str) -> Result<Option<String>, Failure> {
        Err(Self::unsupported(NO_MEMBERS))
    }

    async fn title(&self, member: Option<&str>, id: &str) -> Result<Option<ItemDetail>, Failure> {
        if member.is_some() {
            return Err(Self::unsupported(NO_MEMBERS));
        }
        Ok(self.book(id).await?.map(Book::detail))
    }

    async fn part_way(&self, _member: &str, _most: u32) -> Result<Vec<ItemProgress>, Failure> {
        Err(Self::unsupported(NO_MEMBERS))
    }

    async fn progressed(&self, _member: &str, _id: &str, _how_far: &HowFar) -> Result<(), Failure> {
        Err(Self::unsupported(NO_MEMBERS))
    }

    async fn sign_out(&self, _device: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_DEVICES))
    }

    async fn picture(
        &self,
        member: Option<&str>,
        id: &str,
        which: Picture,
    ) -> Result<Option<Image>, Failure> {
        if member.is_some() {
            return Err(Self::unsupported(NO_MEMBERS));
        }
        match which {
            Picture::Poster => self.poster(id).await,
            Picture::Backdrop => Ok(None),
        }
    }

    async fn holdings(&self, member: Option<&str>, most: u32) -> Result<Vec<Item>, Failure> {
        if member.is_some() {
            return Err(Self::unsupported(NO_MEMBERS));
        }
        self.books(most).await
    }

    async fn playing(&self, _member: Option<&str>) -> Result<Vec<Playback>, Failure> {
        Err(Self::unsupported(NO_PLAYING))
    }
}
