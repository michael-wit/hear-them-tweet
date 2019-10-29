use std::io;

use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::Terminal;

use crate::tracking::Tracking;

/// Terminal based dashboard
pub struct Dashboard {
    terminal: Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
}

impl Dashboard {
    pub fn new() -> Self {
        let stdout = io::stdout().into_raw_mode().expect("Stdout available");
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Terminal created");
        terminal.hide_cursor().expect("Cursor hidden");
        Dashboard { terminal: terminal }
    }

    pub fn update(&mut self, data: &Tracking) {
        self.terminal
            .draw(|mut f| {
                let header = data.get_lables(true);
                let avrg_data = data.get_hourly_moving_avg();
                let trend_data = data.get_hourly_trend();
                let sum_data = data.get_hourly_total();
                let rows = vec![
                    Row::Data(avrg_data.iter()),
                    Row::Data(trend_data.iter()),
                    Row::Data(sum_data.iter()),
                ];
                let widths: Vec<u16> = header.iter().map(|_| 12).collect();
                let rects = Layout::default()
                    .constraints([Constraint::Percentage(30)].as_ref())
                    .margin(5)
                    .split(f.size());
                Table::new(header.iter(), rows.into_iter())
                    .block(Block::default().borders(Borders::ALL).title("Tweets"))
                    .widths(&widths[..])
                    .render(&mut f, rects[0]);
            })
            .expect("Dashboard update");
    }
}
