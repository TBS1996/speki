use tui::layout::Rect;

use crate::app::{AppData, Tab, TabData};
use crate::popups::progress_popup::Progress;
use crate::utils::aliases::Pos;
use crate::utils::ankitemplate::{ImportProgress, Template};
use crate::utils::misc::{split_leftright_by_percent, split_updown_by_percent};
use crate::utils::statelist::{StatefulList, TextItem};
use crate::widgets::textinput::Field;
use crate::widgets::topics::TopicList;
use crate::{MyKey, MyType};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub enum UnzipStatus {
    Ongoing(String),
    Done,
}

#[derive(Clone, Debug)]
pub enum LoadState {
    OnGoing,
    Finished,
    Importing, // e.g.  70/500 cards imported...
}

#[derive(Clone, Debug)]
enum Ftype {
    Text,
    Audio,
    Image,
}

pub struct LoadCards<'a> {
    template: Template,
    front_template: Field,
    back_template: Field,
    front_view: Field,
    back_view: Field,
    topics: TopicList,
    importbutton: Button<'a>,
    previewbutton: Button<'a>,
    fields: StatefulList<TextItem>,
    viewpos: usize,
    tabdata: TabData,
}

impl<'a> LoadCards<'a> {
    pub fn new(appdata: &AppData, deckname: String) -> Self {
        let template = Template::new(appdata, deckname);
        let fields = StatefulList::new("Fields".to_string());
        let importbutton = Button::new("import cards".to_string());
        let previewbutton = Button::new("previewing card".to_string());

        let mut ldc = Self {
            template,
            front_view: Field::default(),
            back_view: Field::default(),
            front_template: Field::new("Front template".to_string()),
            back_template: Field::new("Back template".to_string()),
            topics: TopicList::new(&appdata.conn),
            importbutton,
            previewbutton,
            fields,
            viewpos: 0,
            tabdata: TabData::default(),
        };
        ldc.refresh_template_and_view(0);
        ldc
    }

    pub fn new_csv(appdata: &AppData, path: PathBuf) -> Self {
        let template = Template::new_csv(path);
        let fields = StatefulList::new("Fields".to_string());
        let importbutton = Button::new("import cards".to_string());
        let previewbutton = Button::new("previewing card".to_string());

        let mut ldc = Self {
            template,
            front_view: Field::default(),
            back_view: Field::default(),
            front_template: Field::new("Front template".to_string()),
            back_template: Field::new("Back template".to_string()),
            topics: TopicList::new(&appdata.conn),
            importbutton,
            previewbutton,
            fields,
            viewpos: 0,
            tabdata: TabData::default(),
        };
        ldc.refresh_template_and_view(0);
        ldc
    }

    fn refresh_template_and_view(&mut self, viewpos: usize) {
        self.front_template
            .replace_text(self.template.get_front_template(viewpos));
        self.back_template
            .replace_text(self.template.get_back_template(viewpos));

        self.front_view.replace_text(
            self.template
                .fill_front_view(self.front_template.return_text(), viewpos),
        );
        self.back_view.replace_text(
            self.template
                .fill_back_view(self.back_template.return_text(), viewpos),
        );

        self.fields.items = {
            let model = self.template.selected_model(viewpos);
            model
                .fields
                .iter()
                .map(|item| TextItem::new(item.clone()))
                .collect()
        };
    }

    fn update_template(&mut self, viewpos: usize) {
        let ord = {
            let model = &self.template.models[&self.template.selected_model_id(viewpos)];
            if model.is_cloze {
                0
            } else {
                self.template.cards[viewpos].template_ord
            }
        };

        self.template
            .models
            .get_mut(&self.template.selected_model_id(viewpos))
            .unwrap()
            .templates[ord]
            .qfmt = self.front_template.return_text();

        self.template
            .models
            .get_mut(&self.template.selected_model_id(viewpos))
            .unwrap()
            .templates[ord]
            .afmt = self.back_template.return_text();
    }

    fn refresh_views(&mut self, viewpos: usize) {
        self.front_view.replace_text(
            self.template
                .fill_front_view(self.front_template.return_text(), viewpos),
        );
        self.back_view.replace_text(
            self.template
                .fill_back_view(self.back_template.return_text(), viewpos),
        );
    }
}

use crate::app::Widget;

use crate::widgets::button::Button;

impl<'a> Tab for LoadCards<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn get_title(&self) -> String {
        "load cards".to_string()
    }

    fn set_selection(&mut self, area: Rect) {
        let leftright = split_leftright_by_percent([66, 33], area);

        let (left, right) = (leftright[0], leftright[1]);
        let rightcol = split_updown_by_percent([50, 50], right);
        let (thetopics, thefields) = (rightcol[0], rightcol[1]);
        let updown = split_updown_by_percent([20, 30, 30, 20], left);
        let (preview, up, down, button) = (updown[0], updown[1], updown[2], updown[3]);

        let toprow = split_leftright_by_percent([50, 50], up);
        let bottomrow = split_leftright_by_percent([50, 50], down);

        let (topleft, topright) = (toprow[0], toprow[1]);
        let (bottomleft, bottomright) = (bottomrow[0], bottomrow[1]);

        self.tabdata.view.areas.push(topleft);
        self.tabdata.view.areas.push(thetopics);
        self.tabdata.view.areas.push(preview);
        self.tabdata.view.areas.push(topright);
        self.tabdata.view.areas.push(bottomleft);
        self.tabdata.view.areas.push(bottomright);
        self.tabdata.view.areas.push(button);
        self.tabdata.view.areas.push(thefields);

        self.front_template.set_area(topleft);
        self.back_template.set_area(bottomleft);
        self.front_view.set_area(topright);
        self.back_view.set_area(bottomright);
        self.previewbutton.set_area(preview);
        self.importbutton.set_area(button);
        self.topics.set_area(thetopics);
        self.fields.set_area(thefields);
    }

    fn render(&mut self, f: &mut tui::Frame<MyType>, appdata: &AppData, cursor: &Pos) {
        let media = self.template.get_media(self.viewpos);
        let mut frontstring = "Front side ".to_string();
        let mut backstring = "Back side ".to_string();
        if media.frontaudio.is_some() {
            frontstring.push('🔊')
        }
        if media.backaudio.is_some() {
            backstring.push('🔊')
        }
        if media.frontimage.is_some() {
            frontstring.push('📷')
        }
        if media.backimage.is_some() {
            backstring.push('📷')
        }
        self.front_view.title = frontstring;
        self.back_view.title = backstring;

        let buttontext = format!(
            "Previewing card {} out of {}",
            self.viewpos + 1,
            self.template.cards.len()
        );
        self.previewbutton.change_text(buttontext);
        self.previewbutton.render(f, appdata, cursor);
        self.fields.render(f, appdata, cursor);
        self.topics.render(f, appdata, cursor);
        self.front_template.render(f, appdata, cursor);
        self.back_template.render(f, appdata, cursor);
        self.front_view.render(f, appdata, cursor);
        self.back_view.render(f, appdata, cursor);
        self.importbutton.render(f, appdata, cursor);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, cursor: &Pos) {
        use MyKey::*;

        match key {
            Alt('s') => {
                let front_text = self.front_template.return_text();
                let back_text = self.back_template.return_text();
                self.front_template.replace_text(back_text);
                self.back_template.replace_text(front_text);
                self.update_template(self.viewpos);
                self.refresh_template_and_view(self.viewpos);
            }
            Char('l') | Right if self.previewbutton.is_selected(cursor) => {
                if self.viewpos < self.template.notes.len() - 1 {
                    self.viewpos += 1;
                    self.refresh_template_and_view(self.viewpos);
                    self.template.play_front_audio(&appdata.audio, self.viewpos);
                    self.template.play_back_audio(&appdata.audio, self.viewpos);
                }
            }
            Char('h') | Left if self.previewbutton.is_selected(cursor) => {
                if self.viewpos > 0 {
                    self.viewpos -= 1;
                    self.refresh_template_and_view(self.viewpos);
                    self.template.play_front_audio(&appdata.audio, self.viewpos);
                    self.template.play_back_audio(&appdata.audio, self.viewpos);
                }
            }
            Enter if self.importbutton.is_selected(cursor) => {
                let mut tmpclone = self.template.clone();
                let (tx, rx): (
                    std::sync::mpsc::SyncSender<ImportProgress>,
                    Receiver<ImportProgress>,
                ) = std::sync::mpsc::sync_channel(5);
                let connclone = Arc::clone(&appdata.conn);
                let topic = self.topics.get_selected_id().unwrap();
                std::thread::spawn(move || {
                    tmpclone.import_cards(connclone, tx, topic);
                });
                //let max = self.template.cards.len() as u32;
                let prog = Progress::new(rx, "Importing cards".to_string(), None);
                self.set_popup(Box::new(prog));
            }
            key if self.front_template.is_selected(cursor) => {
                self.front_template.keyhandler(appdata, key);
                self.update_template(self.viewpos);
                self.refresh_views(self.viewpos);
            }
            key if self.back_template.is_selected(cursor) => {
                self.back_template.keyhandler(appdata, key);
                self.update_template(self.viewpos);
                self.refresh_views(self.viewpos);
            }
            key if self.topics.is_selected(cursor) => self.topics.keyhandler(appdata, key),
            _ => {}
        }
    }
}
