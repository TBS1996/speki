#[derive(Debug)]
struct Chapter {
    name: String,
    path: PathBuf,
    content: String,
}
use crate::utils::aliases::*;
use ::epub::doc::EpubDoc;
use epub::doc::NavPoint;

use crate::app::AppData;
use minidom::Element;
use std::path::PathBuf;

use super::sql::insert::new_incread;
pub fn load_book(appdata: &AppData, path: &PathBuf, topic: TopicID) {
    let doc = EpubDoc::new(path);
    let mut doc = doc.unwrap();

    let mut desc = String::new();

    if let Some(val) = doc.metadata.get("title") {
        for x in val {
            desc.push_str(&x.clone());
            desc.push('\n');
            desc.push('\n');
            desc.push('\n');
        }
    }

    if let Some(val) = doc.metadata.get("description") {
        for x in val {
            desc.push_str(&x.clone());
        }
    }

    let parent = new_incread(&appdata.conn, 0, topic, desc, false);

    let mut chapters: Vec<Chapter> = vec![];

    for x in &doc.toc {
        print_labels(&mut chapters, x);
    }

    for chapter in &mut chapters {
        if let Ok(content) = doc.get_resource_str_by_path(&chapter.path) {
            let source = get_body_contents(content);
            if source.as_bytes().len() > 100 {
                new_incread(&appdata.conn, parent, topic, source, false);
            }
        }
    }
}

fn print_labels(chapters: &mut Vec<Chapter>, navpoint: &NavPoint) {
    chapters.push(Chapter {
        name: navpoint.label.clone(),
        path: navpoint.content.clone(),
        content: String::new(),
    });
    for x in &navpoint.children {
        print_labels(chapters, x);
    }
}
const NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
fn get_body_contents(xml: String) -> String {
    let root: Element = xml.trim().parse().unwrap();

    let mut thetext = String::new();

    let body = root.get_child("body", NAMESPACE).unwrap();
    for x in body.children() {
        if x.is("p", NAMESPACE) {
            thetext.push_str(&x.text());
        }
        thetext.push('\n');
    }
    thetext
}
