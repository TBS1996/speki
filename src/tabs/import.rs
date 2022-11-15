use crate::{
    app::{AppData, Tab},
    popups::{
        ankimporter::Ankimporter,
        filepicker::{FilePicker, FilePickerPurpose},
        menu::{Menu, TraitButton},
    },
};

impl<'a> Menu<'a> {
    pub fn new_import_tab() -> Menu<'a> {
        let a = |_appdata: &AppData| -> Box<dyn Tab> { Box::new(Ankimporter::new()) };
        let b = |appdata: &AppData| -> Box<dyn Tab> { Box::new(Menu::new_anki_users(appdata)) };
        let c = |_appdata: &AppData| -> Box<dyn Tab> {
            Box::new(FilePicker::new(
                FilePickerPurpose::LoadCards,
                "Choose a TSV file (tab-separated) with a header".to_string(),
                ["tsv".to_string(), "csv".to_string()],
            ))
        };

        let buttons = [
            TraitButton::new(Box::new(a), "Anki shared decks", false),
            TraitButton::new(Box::new(b), "Anki local useres", false),
            TraitButton::new(Box::new(c), "Load from file", false),
        ];

        Menu::new(
            "Import".to_string(),
            "Choose import method".to_string(),
            30,
            5,
            buttons,
        )
    }
}
