use std::borrow::BorrowMut;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
    Frame,
};

use crate::elm::ApplicationStateModel;

pub fn render(frame: &mut Frame<'_>, chunk: Rect, state: &ApplicationStateModel) {}
