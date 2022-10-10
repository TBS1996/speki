pub mod add_card;
pub mod browse;
pub mod import;
pub mod incread;
pub mod review;

use crate::app::App;

use crate::ui::browse::draw_browse;

use crate::utils::widgets::textinput::Field;
use crate::MyType;
use tui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame<MyType>, app: &mut App) {
    let mut area = f.size();
    if app.display_help {
        area = render_help(f, app, area);
    }

    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let block = Block::default().style(Style::default().bg(Color::Rgb(20, 31, 31)));

    f.render_widget(block, f.size());

    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        //  .style(Style::default().bg(Color::Blue))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);

    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => app.review.render(f, chunks[1], &app.conn),
        1 => app.add_card.render(f, chunks[1]),
        2 => app.incread.render(f, chunks[1]),
        3 => app.importer.render(&app.conn, f, chunks[1], &app.paths),
        4 => draw_browse(f, app, chunks[1]),
        //3 => app.debug.render(f, chunks[1]),
        _ => {}
    };
}

use tui::layout::Direction;
use tui::layout::Rect;

fn render_help(f: &mut Frame<MyType>, app: &mut App, area: Rect) -> Rect {
    let mut field = Field::new();
    field.replace_text(help_msg(app));
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
        .split(area);
    field.render(f, chunks[1], false);
    chunks[0]
}

fn help_msg(app: &mut App) -> String {
    let mut help = r#"
@@@@@@@@@@@@@@@@@@@@@@@@
@F1 TO TOGGLE HELP MENU@
@@@@@@@@@@@@@@@@@@@@@@@@

next tab: Tab,
previous tab: Shift+Tab,
move between widgets: Alt + arrow-keys (or vim-keys)
quit: Alt+q


        "#
    .to_string();

    let review = r#"
    
Skip card => Alt+s
Add old card as dependent: Alt+t
add new card as dependent: Alt+T
add old card as dependency: Alt+y
add new card as dependency: Alt+Y
suspend card: Alt+i
rate card: 1,2,3,4


        "#
    .to_string();

    let revinc = r#"
    
Mark text as done: Alt+d
skip text: Alt+s
make extract (visual mode): Alt+x 
make cloze (visual mode): Alt+z
add child card(in text widget): Alt+a

        "#
    .to_string();

    let revunf = r#"
    
Skip card: Alt+s
complete card: Alt+f
Add old card as dependent: Alt+t
add new card as dependent: Alt+T
add old card as dependency: Alt+y
add new card as dependency: Alt+Y
suspend card: Alt+i

        "#
    .to_string();

    let addcard = r#"
Topic of card is as selected in the topic widget.

Upper textbox is question, lower is answer.

add card as finished: Alt+f
Add card as unfinished: Alt+u    

        "#
    .to_string();

    let increading = r#"

Sources are the top level texts with the topic that is currently selected.
Extracts are the extracts taken from the currently focused text.
You can paste text into the textwidget.

Add wikipedia page: Alt+w
add new source: Alt+a
insert mode -> normal mode: Ctrl+c
normal mode -> insert mode: i
normal mode -> visual mode: v
visual mode -> normal mode: Ctrl+c
make extract (visual mode): Alt+x 
make cloze (visual mode): Alt+z

        "#
    .to_string();

    let  import = r#"

Here you can import any anki decks you want! audio included, but not yet images. Press enter to view description about the deck, and then enter again to download

When inspecting the deck, you can edit the templates for the deck. The front/back view are how the cards will look like after you import them! 

If you don't want to import the selected deck, press escape!


        "#.to_string();

    match app.tabs.index {
        0 => match app.review.mode {
            ReviewMode::Review(_) | ReviewMode::Pending(_) => help.push_str(&review),
            ReviewMode::Unfinished(_) => help.push_str(&revunf),
            ReviewMode::IncRead(_) => help.push_str(&revinc),
            _ => {}
        },
        1 => help.push_str(&addcard),
        2 => help.push_str(&increading),
        3 => help.push_str(&import),
        _ => {}
    }
    help
}

use crate::logic::review::ReviewMode;
