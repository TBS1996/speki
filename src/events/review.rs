use crate::utils::misc::PopUpStatus;
use crate::utils::widgets::textinput::CursorPos;
use crate::{Direction, MyKey};

use crate::logic::review::{CardReview, UnfCard, UnfSelection, PopUp};
use crate::logic::review::{IncMode, IncSelection, ReviewSelection};
use crate::logic::review::{ReviewList, ReviewMode};
use crate::utils::card::{Card, CardType, RecallGrade};
use crate::utils::sql::fetch::get_cardtype;
use crate::utils::sql::update::{
    double_skip_duration, set_suspended, update_card_answer, update_card_question, update_inc_text,
};
use crate::utils::widgets::find_card::{CardPurpose, FindCardWidget};

use crate::utils::aliases::*;
use crate::utils::widgets::newchild::AddChildWidget;
use rusqlite::Connection;

use crate::utils::widgets::newchild::Purpose;

use std::sync::{Arc, Mutex};
