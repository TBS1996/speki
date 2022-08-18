use crate::app::App;
use crate::utils::sql::update::update_topic_name;
use crossterm::event::KeyCode;
use crate::logic::add_card::{TextSelect, DepState, NewTopic} ;
use crate::utils::sql::fetch::highest_id;
use crate::utils::sql::insert::{new_topic};

pub fn add_card_event(app: &mut App, key: KeyCode){
    if app.add_card.istextselected(){
        if let TextSelect::Question(_) = app.add_card.selection{
            match key{
                KeyCode::Right  => app.add_card.question.next(),
                KeyCode::Delete => app.add_card.question.delete(),
                KeyCode::Char(c)   => app.add_card.question.addchar(c),
                KeyCode::Backspace => app.add_card.question.backspace(),
                KeyCode::Left  => app.add_card.question.prev(),
                KeyCode::Enter => app.add_card.selection = TextSelect::Answer(true),
                 _ => {},
            }
        }
        else if let TextSelect::Answer(_) = app.add_card.selection{
            match key{
                KeyCode::Left  => app.add_card.answer.prev(),
                KeyCode::Delete => app.add_card.answer.delete(),
                KeyCode::Char(c)   => app.add_card.answer.addchar(c),
                KeyCode::Backspace => app.add_card.answer.backspace(),
                KeyCode::Right  => app.add_card.answer.next(),
                KeyCode::Enter => app.add_card.selection = TextSelect::SubmitFinished,
                 _ => {},
            }
        }
        match key{
            KeyCode::Up   => app.add_card.uprow(),
            KeyCode::End  => app.add_card.end(),
            KeyCode::Tab  => app.add_card.tab(),
            KeyCode::Esc  => app.add_card.deselect(),
            KeyCode::Home => app.add_card.home(),
            KeyCode::Down   => app.add_card.downrow(),
            KeyCode::PageUp => app.add_card.pageup(),
            KeyCode::BackTab   => app.add_card.backtab(),
            KeyCode::PageDown  => app.add_card.pagedown(),
            _ => {},
        }
        
    }else{
        if let TextSelect::Topic(None) = app.add_card.selection{
            match key{
                KeyCode::Left => app.add_card.navigate(key),
                KeyCode::Char('k') | KeyCode::Up => app.add_card.topics.previous(),
                KeyCode::Char('d') => {
                    let mut index = app.add_card.topics.state.selected().unwrap() as u32;
                    if index == 0 {return}
                    app.add_card.topics.delete_topic(&app.conn, index);
                    if index == app.add_card.topics.items.len() as u32 - 1{ index -= 1}
                    app.add_card.reload_topics(&app.conn);
                    app.add_card.topics.state.select(Some((index) as usize));

                }
                KeyCode::Char('h') => {
                    let index = app.add_card.topics.state.selected().unwrap() as u32;
                    let topic = app.add_card.topics.items[index as usize].clone();
                    if topic.parent == 1 {return}
                    if index == 0 {return}
                    let parent_index = app.add_card.topics.index_from_id(topic.parent);
                    app.add_card.topics.shift_left(&app.conn, index);
                    app.add_card.reload_topics(&app.conn);
                    app.add_card.topics.state.select(Some((parent_index) as usize));

                    

                }
                KeyCode::Char('l') => {
                    let index = app.add_card.topics.state.selected().unwrap() as u32;
                    if index == (app.add_card.topics.items.len() as u32) - 1 {return}
                    if index == 0 {return}
                    let topic = app.add_card.topics.topic_from_index(index);
                    if app.add_card.topics.is_last_sibling(topic.id) {return}
                    if app.add_card.topics.items[index as usize].children.len() > 0 {return}

                    app.add_card.topics.shift_right(&app.conn, index as u32);
                    app.add_card.reload_topics(&app.conn);
                    app.add_card.topics.state.select(Some((index + 1) as usize));
                
                }
                KeyCode::Char('J') => {
                    let index = app.add_card.topics.state.selected().unwrap() as u32;
                    let topic = app.add_card.topics.items[index as usize].clone();

                    if app.add_card.topics.is_last_sibling(topic.id) {return}

                    app.add_card.topics.shift_down(&app.conn, index as u32);
                    app.add_card.reload_topics(&app.conn);
                    let new_index = app.add_card.topics.index_from_id(topic.id);
                    app.add_card.topics.state.select(Some((new_index) as usize));
                }
                KeyCode::Char('K') => {
                    let index = app.add_card.topics.state.selected().unwrap();
                    //let distance = app.add_card.topics.distance_sibling_above(index as u32);
                    if index > 1 {
                    let index_above = app.add_card.topics.index_sibling_above(index as u32);
                    app.add_card.topics.shift_up(&app.conn, index as u32);
                    app.add_card.reload_topics(&app.conn);
                    app.add_card.topics.state.select(Some(index_above as usize));
                    }

                    
                },
                KeyCode::Char('j') | KeyCode::Down => app.add_card.topics.next(),
                KeyCode::Char('a') => {
                    let parent = app.add_card.topics.get_selected_id().unwrap();
                    let parent_index = app.add_card.topics.state.selected().unwrap();

                    let name = String::new();

                    let children = app.add_card.topics.items[parent_index].children.clone();
                    let sibling_qty = (&children).len();
                    
                  //  panic!("sibling index is {} and qty is {} and the children are {:?}", parent_index, sibling_qty, children);
                    new_topic(&app.conn, name, parent, sibling_qty as u32).unwrap();
                    let id = *(&app.conn.last_insert_rowid()) as u32;
                    app.add_card.selection = TextSelect::Topic(Some(NewTopic::new(id)));
                    app.add_card.reload_topics(&app.conn);
                    
                },
                _ => {},
        }}
        else if let TextSelect::Topic(Some(inner)) = &mut app.add_card.selection{
            match key{
                KeyCode::Char(c) => {
                    match &mut app.add_card.selection{
                        TextSelect::Topic(foo) => {
                            foo.as_mut().unwrap().name.addchar(c);
                            let id = foo.as_ref().unwrap().id;
                            let name = foo.as_ref().unwrap().name.text.clone();
                            update_topic_name(&app.conn, id, name).unwrap();
                            app.add_card.reload_topics(&app.conn);


                        },
                        _ => panic!("ohno"),
                    }
                },
                KeyCode::Backspace => {
                    inner.name.backspace();
                    let id = inner.id;
                    let name = inner.name.text.clone();
                    update_topic_name(&app.conn, id, name).unwrap();
                    app.add_card.reload_topics(&app.conn);
                },
                KeyCode::Enter => {
                    let id = inner.id;
                    let index = app.add_card.topics.index_from_id(id);
                    let parent_id = app.add_card.topics.items[index as usize].parent;
                    let parent_index = app.add_card.topics.index_from_id(parent_id);
                    app.add_card.topics.state.select(Some(parent_index as usize));
                    app.add_card.selection = TextSelect::Topic(None);

                }
                _ => {},
            }
        }
        else {
            match key{
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Esc  => app.add_card.topics.state.select(None),
                KeyCode::Char('z')   => app.on_left(),
                KeyCode::Char('x')  => app.on_right(),
                KeyCode::Enter  => app.add_card.enterkey(&app.conn),
                KeyCode::Down   => app.add_card.downkey(),
                KeyCode::Up     => app.add_card.upkey(),
                KeyCode::Right => app.add_card.rightkey(),
                KeyCode::Left => app.add_card.leftkey(),
                KeyCode::Char('y') => {
                    let id = highest_id(&app.conn).unwrap();
                    app.add_card.reset(DepState::HasDependent(id), &app.conn);

                },
                KeyCode::Char('t') => {
                    let id = highest_id(&app.conn).unwrap();
                    app.add_card.reset(DepState::HasDependency(id), &app.conn);
                },
                _=> {},
        }

    }
    }
}
