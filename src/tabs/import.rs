use crate::{
    app::{AppData, Tab},
    popups::{
        ankimporter::Ankimporter,
        filepicker::{FilePicker, FilePickerPurpose},
        menu::Menu,
    },
};

impl<'a> Menu<'a> {
    pub fn new_import_tab() -> Menu<'a> {
        let anki = |_appdata: &AppData| -> Box<dyn Tab> { Box::new(Ankimporter::new()) };
        let ldc = |_appdata: &AppData| -> Box<dyn Tab> {
            Box::new(FilePicker::new(
                FilePickerPurpose::LoadCards,
                "Choose a TSV file (tab-separated) with a header".to_string(),
                ["tsv".to_string(), "csv".to_string()],
            ))
        };

        let foo = |appdata: &AppData| -> Box<dyn Tab> { Box::new(Menu::new_anki_users(appdata)) };

        let thetraits: Vec<Box<dyn FnMut(&AppData) -> Box<dyn Tab>>> =
            vec![Box::new(anki), Box::new(foo), Box::new(ldc)];

        Menu::new(
            "Import".to_string(),
            "Choose import method".to_string(),
            30,
            5,
            [
                "Anki shared decks".to_string(),
                "Anki local users".to_string(),
                "Local file".to_string(),
            ],
            thetraits,
            false,
        )
    }
}
