use crate::utils::aliases::*;
use crate::utils::card::{Card, CardType};
use crate::utils::misc::get_current_unix;
//, Topic, Review}
use crate::widgets::topics::Topic;
use rusqlite::{Connection, Result, Row};
use std::sync::{Arc, Mutex};
use std::time::{ SystemTime, UNIX_EPOCH};

use color_eyre::eyre::Result as PrettyResult;

pub mod cards;

enum CardFilter {
    Suspended(bool),
    Resolved(bool),
    Cardtype(Vec<CardType>),
    StrengthRange((f32, f32)),
    Minstability(u32),
    Maxstability(u32),
    Minstrength(f32),
    Maxstrength(f32),
    Contains(String),
    Topics(Vec<TopicID>),
    MinPosition(u32),
    MaxPosition(u32),
    MinSkipDaysPassed(f32),
    MaxSkipDaysPassed(f32),
    Source(u32),
    DueUnfinished,
}

use std::fmt::{self, Display};
impl fmt::Display for CardFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CardFilter::*;
        let text = match self {
            Suspended(val) => format!("suspended = {}", val),
            Resolved(val) => format!("resolved = {}", val),
            Cardtype(val) => {
                let mut topicstr = String::from("(");
                for var in val {
                    topicstr.push_str(match var {
                        CardType::Pending => "cardtype = 0 OR ",
                        CardType::Unfinished => "cardtype = 1 OR ",
                        CardType::Finished => "cardtype = 2 OR ",
                    });
                }
                // removes the final " OR "
                for _ in 0..4 {
                    topicstr.pop();
                }
                topicstr.push(')');
                if val.is_empty() {
                    topicstr = String::from("cardtype = 3"); //quickfix
                }
                topicstr
            }
            DueUnfinished => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as f32;
                format!("{} - skiptime > (skipduration * 84600)", now)
            }
            MaxSkipDaysPassed(val) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as f32;
                let since = now - (val * 86400.);
                format!("skiptime < {}", since)
            }
            MinSkipDaysPassed(val) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as f32;
                let since = now - (val * 86400.);
                format!("skiptime > {}", since)
            }
            Source(val) => format!("source = {}", val),
            MaxPosition(val) => format!("position < {}", val),
            MinPosition(val) => format!("position > {}", val),
            Minstability(val) => format!("stability > {}", val),
            Maxstability(val) => format!("stability < {}", val),
            Minstrength(val) => format!("strength > {}", val),
            Maxstrength(val) => format!("strength < {}", val),
            StrengthRange(val) => format!("strength BETWEEN {} and {}", val.0, val.1),
            Contains(val) => format!("(question LIKE '%{}%' or answer LIKE '%{}%')", val, val),
            Topics(vec) => {
                let mut topicstr = String::from("(");
                for id in vec {
                    topicstr.push_str(&format!("topic = {} OR ", id));
                }
                // removes the final " OR "
                for _ in 0..4 {
                    topicstr.pop();
                }
                topicstr.push(')');
                topicstr
            }
        };
        write!(f, "{}", text)
    }
}

#[derive(Default)]
pub struct CardQuery {
    filters: Vec<CardFilter>,
    order_by: Option<String>,
    limit: Option<u32>,
}

impl CardQuery {
    fn make_query(&self) -> String {
        let mut query = r#"SELECT * 
            FROM cards
            LEFT OUTER JOIN finished_cards
            ON cards.id = finished_cards.id 
            LEFT OUTER JOIN unfinished_cards
            ON cards.id = unfinished_cards.id 
            LEFT OUTER JOIN pending_cards
            ON cards.id = pending_cards.id

            "#
        .to_string();
        if !self.filters.is_empty() {
            query.push_str("WHERE ");
            for filter in &self.filters {
                query.push_str(&format!("{} and ", filter));
            }

            // removes the final " and "
            for _ in 0..4 {
                query.pop();
            }
        }
        if let Some(orderby) = &self.order_by {
            query.push_str(&format!("{} ", orderby));
        }
        if let Some(limit) = self.limit {
            query.push_str(&format!("LIMIT {}", limit));
        }
        query
    }

    pub fn suspended(mut self, val: bool) -> Self {
        self.filters.push(CardFilter::Suspended(val));
        self
    }
    pub fn resolved(mut self, val: bool) -> Self {
        self.filters.push(CardFilter::Resolved(val));
        self
    }
    pub fn strength(mut self, val: (f32, f32)) -> Self {
        self.filters.push(CardFilter::StrengthRange(val));
        self
    }
    pub fn minimum_stability(mut self, val: u32) -> Self {
        self.filters.push(CardFilter::Minstability(val));
        self
    }
    pub fn max_stability(mut self, val: u32) -> Self {
        self.filters.push(CardFilter::Maxstability(val));
        self
    }
    pub fn contains(mut self, val: String) -> Self {
        self.filters.push(CardFilter::Contains(val));
        self
    }
    pub fn minimum_strength(mut self, val: f32) -> Self {
        self.filters.push(CardFilter::Minstrength(val));
        self
    }
    pub fn max_strength(mut self, val: f32) -> Self {
        self.filters.push(CardFilter::Maxstrength(val));
        self
    }
    pub fn topics(mut self, val: Vec<TopicID>) -> Self {
        self.filters.push(CardFilter::Topics(val));
        self
    }
    pub fn minimum_position(mut self, val: u32) -> Self {
        self.filters.push(CardFilter::MinPosition(val));
        self
    }
    pub fn max_position(mut self, val: u32) -> Self {
        self.filters.push(CardFilter::MaxPosition(val));
        self
    }
    pub fn minimum_days_since_skip(mut self, val: f32) -> Self {
        self.filters.push(CardFilter::MinSkipDaysPassed(val));
        self
    }
    pub fn max_days_since_skip(mut self, val: f32) -> Self {
        self.filters.push(CardFilter::MaxSkipDaysPassed(val));
        self
    }
    pub fn source(mut self, val: u32) -> Self {
        self.filters.push(CardFilter::Source(val));
        self
    }
    pub fn unfinished_due(mut self) -> Self {
        self.filters.push(CardFilter::DueUnfinished);
        self
    }
    pub fn cardtype(mut self, val: Vec<CardType>) -> Self {
        self.filters.push(CardFilter::Cardtype(val));
        self
    }
    pub fn order_by(mut self, val: String) -> Self {
        self.order_by = Some(val);
        self
    }
    pub fn limit(mut self, val: u32) -> Self {
        self.limit = Some(val);
        self
    }

    pub fn fetch_carditems(self, conn: &Arc<Mutex<Connection>>) -> Vec<CardItem> {
        let f = |row: &Row| CardItem {
            question: row.get(1).unwrap(),
            id: row.get(0).unwrap(),
        };
        self.fetch_generic(conn, f)
    }

    pub fn fetch_card_ids(self, conn: &Arc<Mutex<Connection>>) -> Vec<CardID> {
        let f = |row: &Row| row.get(0).unwrap();
        self.fetch_generic(conn, f)
    }
    pub fn fetch_card(self, conn: &Arc<Mutex<Connection>>) -> Vec<Card> {
        //let f = |row: &Row| { row2card(row)};
        let mut cards = self.fetch_generic(conn, row2card);
        fill_card_vec(&mut cards, conn);
        cards
    }

    pub fn fetch_generic<F, T>(self, conn: &Arc<Mutex<Connection>>, mut genfun: F) -> Vec<T>
    where
        F: FnMut(&Row) -> T,
    {
        let query = self.make_query();
        let mut cardvec = Vec::<T>::new();
        conn.lock()
            .unwrap()
            .prepare(&query)
            .unwrap()
            .query_map([], |row| {
                cardvec.push(genfun(row));
                Ok(())
            })
            .unwrap()
            .for_each(|_| {}); // this is just to make the iterator execute, feels clumsy so let me
                               // know if there's a better way
        cardvec
    }
}

pub fn is_table_empty(conn: Conn, table_name: String) -> bool {
    fetch_item(
        conn,
        format!("SELECT EXISTS (SELECT 1 FROM {})", table_name),
        |row| row.get::<usize, usize>(0),
    )
    .unwrap()
        == 0
}

pub fn get_topics(conn: &Arc<Mutex<Connection>>) -> PrettyResult<Vec<Topic>> {
    let mut vecoftops = Vec::<Topic>::new();
    conn.lock()
        .unwrap()
        .prepare("SELECT * FROM topics")?
        .query_map([], |row| {
            Ok(Topic {
                id: row.get(0)?,
                name: row.get(1)?,
                parent: row.get(2)?,
                children: Vec::<CardID>::new(),
                ancestors: 0,
                relpos: row.get(3)?,
            })
        })?
        .for_each(|topic| {
            vecoftops.push(topic.unwrap());
        });
    Ok(vecoftops)
}

use crate::utils::incread::{IncRead, IncStatus};
use crate::widgets::textinput::Field;

struct IncTemp {
    parent: u32,
    topic: u32,
    source: String,
    isactive: bool,
}

// -------------------------------------------------------------- //

/*

(   */
pub fn get_incread(conn: &Arc<Mutex<Connection>>, id: u32) -> IncRead {
    let extracts = load_extracts(conn, id);
    let clozes = CardQuery::default().source(id).fetch_carditems(conn);
    conn.lock()
        .unwrap()
        .query_row("SELECT * FROM incread WHERE id = ?", [id], |row| {
            Ok(IncRead {
                id,
                parent: row.get(1).unwrap(),
                topic: row.get(2).unwrap(),
                source: Field::new_with_text(
                    row.get(3).unwrap(),
                    row.get(7).unwrap(),
                    row.get(8).unwrap(),
                ),
                status: IncStatus::new_from_row(row),
                extracts,
                clozes,
            })
        })
        .unwrap()
}

use crate::utils::incread::IncListItem;

pub fn load_inc_items(
    conn: &Arc<Mutex<Connection>>,
    topic: TopicID,
) -> PrettyResult<Vec<IncListItem>> {
    let mut incvec = Vec::<IncListItem>::new();
    conn.lock()
        .unwrap()
        .prepare("SELECT * FROM incread where parent = 0 and topic = ?")
        .unwrap()
        .query_map([topic], |row| {
            incvec.push(IncListItem {
                text: row.get(3)?,
                id: row.get(0)?,
            });
            Ok(())
        })?
        .for_each(|_| {});
    Ok(incvec)
}

pub fn load_extracts(conn: &Arc<Mutex<Connection>>, parent: IncID) -> Vec<IncListItem> {
    fetch_items(
        conn,
        format!("SELECT * FROM incread WHERE parent={}", parent),
        |row| IncListItem {
            text: row.get(3).unwrap(),
            id: row.get(0).unwrap(),
        },
    )
    .unwrap()
}

pub fn load_active_inc(conn: &Arc<Mutex<Connection>>) -> Vec<IncID> {
    let current_time = get_current_unix();

    fetch_items(
        conn, 
        format!("SELECT id FROM incread where active = 1 and (({} - skiptime) > (skipduration * 86400))", current_time.as_secs()), 
        |row| row.get::<usize, IncID>(0).unwrap()
        ).unwrap()
}

use crate::utils::card::CardItem;

use self::cards::{fill_card_vec, row2card};
pub fn load_inc_text(conn: &Arc<Mutex<Connection>>, id: IncID) -> Result<String> {
    conn.lock()
        .unwrap()
        .prepare("SELECT source FROM incread where id = ?")
        .unwrap()
        .query_row([id], |row| Ok(row.get(0).unwrap()))
}

pub fn load_inc_title(
    conn: &Arc<Mutex<Connection>>,
    incid: IncID,
    titlelen: u16,
) -> Result<String> {
    let mut source = load_inc_text(conn, incid).unwrap();
    source.truncate(titlelen.into());
    if source.len() < 5 {
        source = "Empty Source".to_string();
    }
    Ok(source)
}

pub fn get_topic_of_inc(conn: &Arc<Mutex<Connection>>, id: IncID) -> Result<TopicID> {
    conn.lock()
        .unwrap()
        .prepare("SELECT topic FROM incread where id = ?")
        .unwrap()
        .query_row([id], |row| Ok(row.get(0).unwrap()))
}

enum Table {
    Cards,
    Dependencies,
    FinishedCards,
    IncRead,
    PendingCards,
    Revlog,
    Topics,
    UnfinishedCards,
}

impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Cards => "cards",
            Self::Dependencies => "dependencies",
            Self::FinishedCards => "finished_cards",
            Self::IncRead => "incread",
            Self::PendingCards => "pending_cards",
            Self::Revlog => "revlog",
            Self::Topics => "topics",
            Self::UnfinishedCards => "unfinished_cards",
        };
        write!(f, "{}", s)
    }
}

enum CardsCols {
    Id,
    Question,
    Answer,
    FrontAudio,
    BackAudio,
    FrontImg,
    BackImg,
    CardType,
    Suspended,
    Resolved,
    Topic,
    Source,
}

impl Display for CardsCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Question => "question",
            Self::Answer => "answer",
            Self::FrontAudio => "frontaudio",
            Self::BackAudio => "backaudio",
            Self::FrontImg => "frontimg",
            Self::BackImg => "backimg",
            Self::CardType => "cardtype",
            Self::Suspended => "suspended",
            Self::Resolved => "resolved",
            Self::Topic => "topic",
            Self::Source => "source",
        };
        write!(f, "{}", s)
    }
}

enum DependenciesCols {
    Dependent,
    Dependency,
}

impl Display for DependenciesCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Dependent => "dependent",
            Self::Dependency => "dependency",
        };
        write!(f, "{}", s)
    }
}

enum FinishedCardsCols {
    Id,
    Strength,
    Stability,
}

impl Display for FinishedCardsCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Strength => "strength",
            Self::Stability => "stability",
        };
        write!(f, "{}", s)
    }
}

enum IncReadCols {
    Id,
    Parent,
    Topic,
    Source,
    Active,
    SkipTime,
    SkipDuration,
    Row,
    Column,
}

impl Display for IncReadCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Parent => "parent",
            Self::Topic => "topic",
            Self::Source => "source",
            Self::Active => "active",
            Self::SkipTime => "skiptime",
            Self::SkipDuration => "skipduration",
            Self::Row => "row",
            Self::Column => "column",
        };
        write!(f, "{}", s)
    }
}

enum PendingCardsCols {
    Id,
    Position,
}

impl Display for PendingCardsCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Position => "position",
        };
        write!(f, "{}", s)
    }
}

enum RevlogCols {
    Unix,
    Cid,
    Grade,
    Qtime,
    Atime,
}

impl Display for RevlogCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Unix => "unix",
            Self::Cid => "cid",
            Self::Grade => "grade",
            Self::Qtime => "qtime",
            Self::Atime => "atime",
        };
        write!(f, "{}", s)
    }
}

enum TopicCols {
    Id,
    Name,
    Parent,
    Relpos,
}

impl Display for TopicCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Parent => "parent",
            Self::Relpos => "relpos",
        };
        write!(f, "{}", s)
    }
}

enum UnfinishedCardsCols {
    Id,
    Skiptime,
    Skipduration,
}

impl Display for UnfinishedCardsCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Id => "id",
            Self::Skiptime => "skiptime",
            Self::Skipduration => "skipduration",
        };
        write!(f, "{}", s)
    }
}

enum DBCol {
    Cards(CardsCols),
    Dependencies(DependenciesCols),
    FinishedCards(FinishedCardsCols),
    Incread(IncReadCols),
    PendingCards(PendingCardsCols),
    Revlog(RevlogCols),
    Topic(TopicCols),
    UnfinishedCards(UnfinishedCardsCols),
}

impl Display for DBCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Cards(val) => val.to_string(),
            Self::Dependencies(val) => val.to_string(),
            Self::FinishedCards(val) => val.to_string(),
            Self::Incread(val) => val.to_string(),
            Self::PendingCards(val) => val.to_string(),
            Self::Revlog(val) => val.to_string(),
            Self::Topic(val) => val.to_string(),
            Self::UnfinishedCards(val) => val.to_string(),
        };
        write!(f, "{}", s)
    }
}

enum SqlOperator {
    Equal,
    GreaterThan,
    LessThan,
}

impl Display for SqlOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Equal => "=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
        };
        write!(f, "{}", s)
    }
}

struct Matcher<T: Display> {
    column: DBCol,
    operator: SqlOperator,
    value: T,
}

impl<T: Display> Display for Matcher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.column, self.operator, self.value)
    }
}

/*
fn fetch_rows<T, U, V>(conn: Conn, table: Table, columns: T, matcher: U) -> Vec<Rows>
where
    T: Into<Vec<DBCol>>,
    U: Into<Vec<Matcher<V>>>,
    V: Display,
{
    let columns = columns.into();
    let matcher = matcher.into();

    let cols = if columns.is_empty() {
        "*".to_string()
    } else {
        let mut foo = String::new();
        for col in columns.iter().peekable() {
            foo.push_str(&format!("{}, ", col));
        }
        foo
    };

    conn.lock()
        .unwrap()
        .prepare("Select * from pending_cards ORDER BY position ASC")
        .unwrap()
        .query_map([], |row| {
            cardvec.push(PendingItem {
                id: row.get(0).unwrap(),
                pos: row.get(1).unwrap(),
            });
            Ok(())
        })
        .unwrap()
        .for_each(|_| {}); // this is just to make the iterator execute, feels clumsy so let me
}

*/

pub fn fetch_items<F, T>(conn: Conn, statement: String, mut genfun: F) -> PrettyResult<Vec<T>>
where
    F: FnMut(&Row) -> T,
{
    let mut vec = Vec::<T>::new();
    conn.lock()
        .unwrap()
        .prepare(&statement)?
        .query_map([], |row| {
            vec.push(genfun(row));
            Ok(())
        })?
        .for_each(|_| {});

    Ok(vec)
}

pub fn fetch_item<F, T>(conn: Conn, statement: String, genfun: F) -> PrettyResult<T>
where
    F: FnMut(&Row) -> Result<T>,
{
    let res = conn.lock().unwrap().query_row(&statement, [], genfun)?;
    Ok(res)
}
