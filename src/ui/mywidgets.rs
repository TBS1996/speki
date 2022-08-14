use crate::logic::review;
use crate::utils::sql::fetch::fetch_card;
use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Row, Wrap, Table, Cell, ListItem, List},
    Frame,
};
use crate::utils::structs::Topic;



pub fn draw_field<B>(f: &mut Frame<B>, area: Rect, text: Vec<Span>, title: &str, alignment: Alignment, selected: bool)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    
    let block = Block::default().borders(Borders::ALL).border_style(style).title(Span::styled(
        title,
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(Spans::from(text)).block(block).alignment(alignment).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

pub fn cursorsplit(text: &str, index: usize) -> Vec<Span> {
        
    if text.len() == 0{
    return vec![Span::from(text)];
    }




    let (beforecursor, cursor) = text.split_at(index);
    let (cursor, aftercursor) = cursor.split_at(1);

    vec![
        Span::from(beforecursor),
        Span::styled(cursor, Style::default().add_modifier(Modifier::REVERSED)),
        Span::from(aftercursor)]

}





pub fn card_status<B>(f: &mut Frame<B>, _app: &mut App, area: Rect, selected: bool)
where
    B: Backend,
{
    let opt =  _app.review.card;
    if let None = opt{
        return;
    }
    let card = fetch_card(&_app.conn, opt.unwrap());

    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    
    let rows = vec![
        Row::new(vec![Cell::from(Span::raw(format!("initiated: {:?}", card.status.initiated)))]),
        Row::new(vec![Cell::from(Span::raw(format!("complete:  {:?}",  card.status.complete)))]),
        Row::new(vec![Cell::from(Span::raw(format!("resolved:  {:?}",  card.status.resolved)))]),
        Row::new(vec![Cell::from(Span::raw(format!("suspended: {:?}", card.status.suspended)))]),
        Row::new(vec![Cell::from(Span::raw(format!("strength:  {}",    card.strength)))]),
        Row::new(vec![Cell::from(Span::raw(format!("stability: {}",   card.stability)))]),
    ];

    
    let table = Table::new(rows).block(Block::default().title("stats").borders(Borders::ALL).border_style(style)).widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);

}



pub fn view_dependencies<B>(f: &mut Frame<B>, id: u32, app: & App, area: Rect, selected: bool)
where
    B: Backend,
{
    let thecard = fetch_card(&app.conn, id);
    let dep_ids = &thecard.dependencies;
    
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

   
    let mut deps = Vec::<Row>::new();
    for dep_id in dep_ids{
        let dep = fetch_card(&app.conn, *dep_id);
        let ques = dep.question.clone();
        let foo = Row::new(vec![Cell::from(Span::raw(ques))]);
        deps.push(foo.clone());
    }
    
    
    let table = Table::new(deps).block(Block::default().title("dependencies").borders(Borders::ALL).border_style(style)).widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);
}


pub fn view_dependents<B>(f: &mut Frame<B>, id: u32, app: & App, area: Rect, selected: bool)
where
    B: Backend,
{
    let thecard = fetch_card(&app.conn, id);
    let dep_ids = &thecard.dependents;
    

   
    let mut deps = Vec::<Row>::new();
    for dep_id in dep_ids{
        let dep = fetch_card(&app.conn, *dep_id);
        let foo = Row::new(vec![Cell::from(Span::raw(dep.question.clone()))]);
        deps.push(foo);
    }
    
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);
    
    let table = Table::new(deps)
        .block(
            Block::default()
                .title("dependents")
                .borders(Borders::ALL)
                .border_style(style))
        .widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);
}








/*
fn filter_status<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{

    let items: Vec<ListItem> = _app.browse.selected.items.iter().map(|id| {
        let lines = vec![Spans::from(_fetch_card(id).question.clone())];
        ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
    }).collect();
    


    
    let mut items = List::new(items).block(Block::default().borders(Borders::ALL).title("Selected"));


    if let BrowseCursor::Selected = _app.browse.cursor{
        items = items
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>> ");
        
    }

    f.render_stateful_widget(items, area, &mut _app.browse.selected.state);

}

*/

fn topic2string(topic: &Topic) -> String {
    let mut mystring: String = String::new();
    if topic.ancestors > 0{
        for x in 0..topic.ancestors - 1{
            mystring.push_str("  ");
        }
        mystring.push_str("└─")
    }



    mystring.push_str(&topic.name);
    mystring
}


// TODO pass in &Topics as an argument so that this widget can be used for several
pub fn topiclist<B>(f: &mut Frame<B>, _app: &mut App, area: Rect, selected: bool)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);


    let items: Vec<ListItem> = _app.add_card.topics.items.iter().map(|topic| {
        let lines = vec![Spans::from(topic2string(topic))];
        ListItem::new(lines).style(Style::default().fg(Color::Red).bg(Color::Black))
    }).collect();
    
    let items = List::new(items).block(Block::default().borders(Borders::TOP).border_style(style).title("Topics"));

    let  items = items
        .highlight_style(
            Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
//    .highlight_symbol(">>> ");
    
    
    f.render_stateful_widget(items, area, &mut _app.add_card.topics.state);



}
