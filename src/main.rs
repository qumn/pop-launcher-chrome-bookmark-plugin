use std::{io, os::unix::prelude::CommandExt, process::Command};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use pop_launcher_toolkit::{
    launcher::{PluginResponse, PluginSearchResult},
    plugin_trait::{
        async_trait,
        tracing::{debug, error},
        PluginExt,
    },
};

mod util;

pub struct ChromeBookMarks {
    bookmarks: Vec<BookMarkEntry>,
    fuzzy_matcher: SkimMatcherV2,
}

impl ChromeBookMarks {
    async fn new() -> Self {
        let bookmarks = ChromeBookMarks::load_all_bookmark();
        ChromeBookMarks {
            bookmarks,
            fuzzy_matcher: SkimMatcherV2::default().ignore_case().use_cache(true),
        }
    }

    // load all bookmarks from chrome's bookmarks file
    fn load_all_bookmark() -> Vec<BookMarkEntry> {
        util::get_bookmarks()
    }

    fn sort_match(&mut self, query: &str) {
        self.bookmarks.sort_by(|a, b| {
            let score_a = self.fuzzy_matcher.fuzzy_match(&a.name, query).unwrap_or(-1);
            let score_b = self.fuzzy_matcher.fuzzy_match(&b.name, query).unwrap_or(-1);

            score_b.cmp(&score_a)
        })
    }
}

impl From<&BookMarkEntry> for PluginSearchResult {
    fn from(entry: &BookMarkEntry) -> Self {
        PluginSearchResult {
            id: 0,
            name: entry.name.clone(),
            description: entry.url.clone(),
            keywords: None,
            icon: None,
            exec: None,
            window: None,
        }
    }
}

#[async_trait]
impl PluginExt for ChromeBookMarks {
    fn name(&self) -> &str {
        "cb"
    }

    async fn search(&mut self, query: &str) {
        match query.split_once(' ') {
            Some(("cb", query)) => {
                // sort bookmarks by query string
                self.sort_match(query);
                for (id, entry) in self.bookmarks.iter().enumerate().take(20) {
                    let mut plugin_search_result: PluginSearchResult = entry.into();
                    plugin_search_result.id = id as u32;
                    self.respond_with(PluginResponse::Append(plugin_search_result))
                        .await;
                }
                self.respond_with(PluginResponse::Finished).await;
            }
            _ => {
                self.respond_with(PluginResponse::Finished).await;
            }
        }
    }

    async fn activate(&mut self, id: u32) {
        self.respond_with(PluginResponse::Close).await;
        match self.bookmarks.get(id as usize) {
            Some(history) => {
                history.exec();
                std::process::exit(0);
            }
            None => {
                error!("entry not found at index {id}");
            }
        }
    }
}

#[derive(Debug)]
pub struct BookMarkEntry {
    name: String,
    url: String,
}

impl BookMarkEntry {
    // open url in chrome
    fn exec(&self) -> io::Error {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(&self.url);

        debug!("excute: {:?}", cmd);
        cmd.exec()
    }
}

impl From<&serde_json::Value> for BookMarkEntry {
    fn from(element: &serde_json::Value) -> Self {
        BookMarkEntry {
            name: element
                .get("name")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string(),
            url: element
                .get("url")
                .map(|u| u.as_str().unwrap_or(""))
                .unwrap_or("")
                .to_string(),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    let mut plugin = ChromeBookMarks::new().await;
    plugin.run().await;
}
