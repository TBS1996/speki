use tui::{layout::Rect, widgets::Borders};

use crate::{
    app::{AppData, PopUpState, Tab, TabData, Widget},
    utils::{aliases::Pos, area::split_updown_by_percent},
    widgets::{button::Button, infobox::InfoBox},
    MyKey,
};

pub struct TraitButton<'a> {
    tab: MenuTrait,
    button: Button<'a>,
    in_place: bool, // should the new tab replace current one or be overlaid?
}

impl<'a> TraitButton<'a> {
    pub fn new<T: Into<String>>(tab: MenuTrait, text: T, in_place: bool) -> Self {
        TraitButton {
            tab,
            button: Button::new(text.into()),
            in_place,
        }
    }
}

pub struct Menu<'a> {
    buttons: Vec<TraitButton<'a>>,
    tabdata: TabData,
    xpad: u32,
    ypad: u32,
    prompt: InfoBox<'a>,
}

pub type MenuTrait = Box<dyn FnMut(&AppData) -> Box<dyn Tab>>;

impl<'a> Menu<'a> {
    pub fn new<T>(title: String, prompt: String, xpad: u32, ypad: u32, buttons: T) -> Self
    where
        T: Into<Vec<TraitButton<'a>>>,
    {
        let prompt = InfoBox::new(prompt).borders(Borders::NONE);

        Self {
            buttons: buttons.into(),
            tabdata: TabData::new(title),
            xpad,
            ypad,
            prompt,
        }
    }
}

impl<'a> Tab for Menu<'a> {
    fn set_selection(&mut self, mut area: Rect) {
        let chunks = split_updown_by_percent([20, 10, 70], area);
        area = chunks[2];

        let mut width = 0;

        for x in &self.buttons {
            if width < x.button.inner.txtlen {
                width = x.button.inner.txtlen;
            }
        }
        width += self.xpad as usize;

        let xpos = (area.width - width as u16) / 2;

        let max_height = area.y + area.height;
        for i in 0..self.buttons.len() {
            let y = area.y as i32 + 1 + (i * self.ypad as usize) as i32;
            let area = Rect {
                x: xpos,
                y: y as u16,
                width: width as u16,
                height: self.ypad as u16,
            };

            if area.y + area.height < max_height {
                self.buttons[i].button.set_area(area);
                self.tabdata.view.areas.push(area);
            }
        }
        self.prompt.set_area(chunks[1]);
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &Pos,
    ) {
        self.prompt.render(f, appdata, cursor);
        for button in &mut self.buttons {
            button.button.render(f, appdata, cursor);
        }
    }

    fn get_tabdata(&mut self) -> &mut crate::app::TabData {
        &mut self.tabdata
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey, cursor: &Pos) {
        match key {
            MyKey::Enter | MyKey::KeyPress(_) => {
                for button in &mut self.buttons {
                    if button.button.is_selected(cursor) {
                        if !button.in_place {
                            let obj = (button.tab)(appdata);
                            self.set_popup(obj)
                        } else {
                            self.tabdata.state = PopUpState::Switch((button.tab)(appdata));
                        };
                        return;
                    }
                }
            }
            MyKey::Up | MyKey::Char('k') => self.tabdata.view.move_up(),
            MyKey::Down | MyKey::Char('j') => self.tabdata.view.move_down(),
            _ => {}
        }
    }
}
