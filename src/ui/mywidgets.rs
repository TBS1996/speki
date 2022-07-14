use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Row, Wrap, Table, Cell},
    Frame,
};



pub fn draw_field<B>(f: &mut Frame<B>, area: Rect, text: Vec<Span>, title: &str, alignment: Alignment)
where
    B: Backend,
{
    
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
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





pub fn card_status<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    let opt =  _app.review.card;
    if let None = opt{
        return;
    }
    let card = &_app.cardmap[&opt.unwrap()];
    
    let rows = vec![
        Row::new(vec![Cell::from(Span::raw(format!("initiated: {:?}", card.status.initiated)))]),
        Row::new(vec![Cell::from(Span::raw(format!("complete:  {:?}",  card.status.complete)))]),
        Row::new(vec![Cell::from(Span::raw(format!("resolved:  {:?}",  card.status.resolved)))]),
        Row::new(vec![Cell::from(Span::raw(format!("suspended: {:?}", card.status.suspended)))]),
        Row::new(vec![Cell::from(Span::raw(format!("strength:  {}",    card.strength)))]),
        Row::new(vec![Cell::from(Span::raw(format!("stability: {}",   card.stability)))]),
    ];

    
    let table = Table::new(rows).block(Block::default().title("stats").borders(Borders::ALL)).widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);

}



pub fn view_dependencies<B>(f: &mut Frame<B>, id: u32, app: & App, area: Rect)
where
    B: Backend,
{
    let thecard = &app.cardmap.get(&id).clone().unwrap();
    let dep_ids = &thecard.dependencies;
    

   
    let mut deps = Vec::<Row>::new();
    for dep_id in dep_ids{
        let dep = &app.cardmap.get(&dep_id).clone().unwrap();
        let foo = Row::new(vec![Cell::from(Span::raw(&dep.question))]);
        deps.push(foo);
    }
    
    
    let table = Table::new(deps).block(Block::default().title("dependencies").borders(Borders::ALL)).widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);
}


pub fn view_dependents<B>(f: &mut Frame<B>, id: u32, app: & App, area: Rect)
where
    B: Backend,
{
    let thecard = &app.cardmap.get(&id).clone().unwrap();
    let dep_ids = &thecard.dependents;
    

   
    let mut deps = Vec::<Row>::new();
    for dep_id in dep_ids{
        let dep = &app.cardmap.get(&dep_id).clone().unwrap();
        let foo = Row::new(vec![Cell::from(Span::raw(&dep.question))]);
        deps.push(foo);
    }
    
    
    let table = Table::new(deps).block(Block::default().title("dependents").borders(Borders::ALL)).widths(&[
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
        let lines = vec![Spans::from(_app.cardmap[id].question.clone())];
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
