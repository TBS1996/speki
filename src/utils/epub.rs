#[derive(Debug)]
struct Chapter {
    name: String,
    path: PathBuf,
    content: String,
}
use ::epub::doc::EpubDoc;
use epub::doc::NavPoint;

use crate::app::AppData;
use minidom::{Element, Node};
use std::path::PathBuf;

pub fn load_book(_appdata: &AppData, path: &PathBuf) -> (String, Vec<String>) {
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

    let mut chapters: Vec<Chapter> = vec![];
    let mut chaptercontent = vec![];

    for x in &doc.toc {
        print_labels(&mut chapters, x);
    }

    for chapter in &mut chapters {
        if let Ok(content) = doc.get_resource_str_by_path(&chapter.path) {
            let source = get_body_contents(content);
            chaptercontent.push(source);
        }
    }
    (desc, chaptercontent)
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
    for node in body.nodes() {
        get_text(&mut thetext, node);
        thetext = thetext.trim().to_string();
        thetext.push('\n');
    }
    thetext
}

fn get_text(thetext: &mut String, node: &Node) {
    match node {
        Node::Element(element) => {
            for child in element.nodes() {
                get_text(thetext, child);
            }
        }
        Node::Text(text) => {
            thetext.push_str(&text);
        }
    }
}
