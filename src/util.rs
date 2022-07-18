use std::{collections::VecDeque, fs, path::PathBuf};

use serde_json::Value;

use crate::BookMarkEntry;

pub fn get_bookmarks() -> Vec<BookMarkEntry> {
    let bookmarks_str: String = get_bookmarks_str();
    let bookmarks_json: Value = serde_json::from_str(&bookmarks_str).unwrap();
    let bookmarks_roots = get_bookmarks_roots(&bookmarks_json);

    unfold_tree(bookmarks_roots)
}

// read content rome ~/.config/google-chrome/Default/Bookmarks
fn get_bookmarks_str() -> String {
    let mut home = get_home();
    home.push(".config/google-chrome/Default/Bookmarks");
    let bookmarks_path = home.as_path();
    fs::read_to_string(bookmarks_path).expect("can't read the {bookmarks_path}")
}

fn get_home() -> PathBuf {
    dirs::home_dir().expect("$HOME not found")
}

// get roots from bookmarks_json
fn get_bookmarks_roots<'a>(bookmarks_json: &'a Value) -> impl Iterator<Item = &'a Value> {
    let bookmarks_roots = bookmarks_json
        .get("roots")
        .and_then(|r| r.as_object())
        .unwrap();
    bookmarks_roots.values()
}

//  get all book marks from bookmarks tree
fn unfold_tree<'a>(roots: impl Iterator<Item = &'a Value>) -> Vec<BookMarkEntry> {
    let mut result = vec![];
    let mut queue = VecDeque::new();
    queue.extend(roots);

    while let Some(element) = queue.pop_front() {
        match element.get("children") {
            Some(dir) => {
                // has children field is a fold
                // TODO use a other function to instead clone
                queue.extend(dir.as_array().unwrap());
            }
            None => {
                // element is a bookmark
                result.push(element.into());
            }
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn get_all_bookmarks() -> Result<(), anyhow::Error> {
        let bookmarks = get_bookmarks();
        for bookmark in bookmarks.iter() {
            println!("bookmark: {:?}", bookmark)
        }
        println!("size: {}", bookmarks.len());

        Ok(())
    }
}
