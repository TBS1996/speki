pub mod import_tab {
    use crate::{
        app::{AppData, Tab},
        popups::{
            ankimporter::Ankimporter,
            filepicker::{FilePicker, FilePickerPurpose},
            menu::Menu,
        },
    };

    pub fn new<'a>() -> Menu<'a> {
        let anki = |_appdata: &AppData| -> Box<dyn Tab> { Box::new(Ankimporter::new()) };
        let ldc = |_appdata: &AppData| -> Box<dyn Tab> {
            Box::new(FilePicker::new(
                FilePickerPurpose::LoadCards,
                "Choose a TSV file (tab-separated) with a header".to_string(),
                ["tsv".to_string(), "csv".to_string()],
            ))
        };

        let thetraits: Vec<Box<dyn FnMut(&AppData) -> Box<dyn Tab>>> =
            vec![Box::new(anki), Box::new(ldc)];

        Menu::new(
            "Import".to_string(),
            "Choose import method".to_string(),
            30,
            5,
            ["Anki".to_string(), "Local".to_string()],
            thetraits,
            false,
        )
    }
}
