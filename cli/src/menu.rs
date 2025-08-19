use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum MenuChoice {
    RunSB,
    RunSDisk,
    UpdateBinaries,
    Settings,
    Exit,
}

pub struct Menu {
    state: ListState,
    items: Vec<(&'static str, MenuChoice)>,
    sb_path: Option<PathBuf>,
    sdisk_path: Option<PathBuf>,
}

impl Menu {
    pub fn new() -> Self {
        let items = vec![
            ("📚 Run Saorsa Browser (sb)", MenuChoice::RunSB),
            ("💾 Run Saorsa Disk (sdisk)", MenuChoice::RunSDisk),
            ("🔄 Update Binaries", MenuChoice::UpdateBinaries),
            ("⚙️  Settings", MenuChoice::Settings),
            ("🚪 Exit", MenuChoice::Exit),
        ];

        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            items,
            sb_path: None,
            sdisk_path: None,
        }
    }

    pub fn set_binary_paths(&mut self, sb_path: Option<PathBuf>, sdisk_path: Option<PathBuf>) {
        self.sb_path = sb_path;
        self.sdisk_path = sdisk_path;
    }

    pub async fn run(&mut self) -> Result<MenuChoice> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal).await;

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    async fn run_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<MenuChoice> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => self.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.next(),
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some(selected) = self.state.selected() {
                                return Ok(self.items[selected].1.clone());
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(MenuChoice::Exit);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(4),
            ])
            .split(f.area());

        self.draw_header(f, chunks[0]);
        self.draw_menu(f, chunks[1]);
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Saorsa CLI", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("Interactive menu for Saorsa tools"),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));

        f.render_widget(header, area);
    }

    fn draw_menu(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (label, choice))| {
                let mut style = Style::default();
                let mut suffix = String::new();

                // Add status indicators
                match choice {
                    MenuChoice::RunSB => {
                        if self.sb_path.is_none() {
                            style = style.fg(Color::DarkGray);
                            suffix = " (not installed)".to_string();
                        } else {
                            style = style.fg(Color::Green);
                        }
                    }
                    MenuChoice::RunSDisk => {
                        if self.sdisk_path.is_none() {
                            style = style.fg(Color::DarkGray);
                            suffix = " (not installed)".to_string();
                        } else {
                            style = style.fg(Color::Green);
                        }
                    }
                    _ => {}
                }

                // Highlight selected item
                if Some(i) == self.state.selected() {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(format!("{}{}", label, suffix))
                    .style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Menu ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default())
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().fg(Color::DarkGray)),
                Span::styled("↑↓/jk", Style::default().fg(Color::Cyan)),
                Span::styled(" | Select: ", Style::default().fg(Color::DarkGray)),
                Span::styled("Enter/Space", Style::default().fg(Color::Cyan)),
                Span::styled(" | Quit: ", Style::default().fg(Color::DarkGray)),
                Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));

        f.render_widget(footer, area);
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}