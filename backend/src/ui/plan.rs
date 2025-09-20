use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::acp::{Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus};

#[derive(Clone, Debug)]
pub struct PlanView {
    pub plan: Option<Plan>,
    state: ListState,
}

impl PlanView {
    pub fn new() -> Self {
        Self {
            plan: None,
            state: ListState::default(),
        }
    }

    pub fn has_plan(&self) -> bool {
        self.plan.is_some()
    }

    pub fn set_plan(&mut self, plan: Plan) {
        self.plan = Some(plan);
        self.state.select(Some(0));
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select_next(&mut self) {
        if let Some(count) = self.plan.as_ref().map(|p| p.entries.len()) {
            let i = match self.state.selected() {
                None => 0,
                Some(i) => {
                    if i >= count - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
            };
            self.state.select(Some(i));
        }
    }

    pub fn select_prev(&mut self) {
        if let Some(count) = self.plan.as_ref().map(|p| p.entries.len()) {
            let i = match self.state.selected() {
                None => 0,
                Some(i) => {
                    if i == 0 {
                        count.saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
            };
            self.state.select(Some(i));
        }
    }

    pub fn render(&self, area: Rect, f: &mut Frame) {
        if let Some(plan) = &self.plan {
            let items: Vec<ListItem> = plan.entries.iter().enumerate().map(|(i, entry)| {
                let priority_color = match entry.priority {
                    PlanEntryPriority::High => Color::Red,
                    PlanEntryPriority::Medium => Color::Yellow,
                    PlanEntryPriority::Low => Color::Green,
                };

                let status_style = match entry.status {
                    PlanEntryStatus::Pending => Style::default().fg(Color::Gray),
                    PlanEntryStatus::InProgress => Style::default().fg(Color::Yellow),
                    PlanEntryStatus::Completed => Style::default().fg(Color::Green),
                };

                let status_icon = match entry.status {
                    PlanEntryStatus::Pending => "⏳",
                    PlanEntryStatus::InProgress => "⚡",
                    PlanEntryStatus::Completed => "✅",
                };

                let line = Line::from(vec![
                    Span::styled(status_icon, status_style),
                    Span::raw(" "),
                    Span::styled(entry.content.as_str(), Style::default().fg(priority_color)),
                ]);

                ListItem::new(line)
            }).collect();

            let mut state = self.state.clone();
            if state.selected().is_none() {
                state.select(Some(0));
            }

            let list = List::new(items)
                .block(Block::default().title("Agent Plan").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Blue))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, area, &mut state);
        } else {
            let block = Block::default()
                .title("Agent Plan")
                .borders(Borders::ALL);
            f.render_widget(block, area);
        }
    }
}