use crossterm::{
    self,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    self,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

struct StatefulList<T> {
    list_state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            list_state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i))
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i))
    }

    fn unselect(&mut self) {
        self.list_state.select(None);
    }
}

#[derive(PartialEq)]
enum Mode {
    Add,
    Delete,
    Update,
    Normal,
}

struct AppState<'a> {
    list: StatefulList<&'a mut String>,
    mode: Mode,
}

impl<'a> AppState<'a> {
    fn new(list_vector: Vec<&'a mut String>) -> AppState<'a> {
        AppState {
            list: StatefulList::with_items(list_vector),
            mode: Mode::Normal,
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: AppState,
    tick_rate: Duration,
) -> Result<(), io::Error> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    Mode::Update => match key.code {
                        KeyCode::Char(c) => {
                            app.list.items[app.list.list_state.selected().unwrap()].push(c);
                        }
                        KeyCode::Backspace => {}
                        KeyCode::Enter => app.mode = Mode::Normal,
                        _ => {}
                    },
                    Mode::Add => {}
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Left => app.list.unselect(),
                        KeyCode::Down => app.list.next(),
                        KeyCode::Up => app.list.previous(),
                        KeyCode::Char('a') => app.mode = Mode::Add,
                        KeyCode::Char('e') => app.mode = Mode::Update,
                        _ => {}
                    },
                    Mode::Delete => {}
                }
                if app.mode == Mode::Normal {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(3),
        ])
        .split(f.size());

    let options_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(2)])
        .split(chunks[2]);

    let block = Block::default().borders(Borders::ALL);
    let list_items: Vec<ListItem> = app
        .list
        .items
        .iter()
        .map(|i| {
            ListItem::new(Spans::from(Span::styled(
                String::from(&*i.as_str()),
                Style::default(),
            )))
        })
        .collect();

    let list = List::new(list_items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>");

    let app_title = Paragraph::new(vec![Spans::from(Span::styled(
        "Todo List",
        Style::default(),
    ))])
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default())
    .alignment(Alignment::Center);

    let options = vec!["Add -> A", "Delete -> X", "Edit -> E"];

    let options: Vec<Spans> = options
        .iter()
        .map(|i| Spans::from(Span::styled(*i, Style::default())))
        .collect();

    let options_list = Tabs::new(options)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    match app.mode {
        Mode::Add => {
            f.set_cursor(chunks[0].x, chunks[0].y);
        }
        Mode::Update => {}
        Mode::Delete => {}
        Mode::Normal => {}
    }

    f.render_widget(app_title, chunks[0]);
    f.render_widget(options_list, options_chunks[0]);
    f.render_stateful_widget(list, chunks[1], &mut app.list.list_state);
}

fn get_item_position(list: Vec<ListItem>, index: usize, chunk: Rect) {
    // TODO
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let s1: &mut String = &mut String::from("Be a gangster");
    let s2: &mut String = &mut String::from("Finish a project");
    let s3: &mut String = &mut String::from("Be a coder");
    let mut list_vector: Vec<&mut String> = vec![s1, s2, s3];
    let app = AppState::new(list_vector);
    let tick_rate = Duration::from_millis(250);

    let result = run_app(&mut terminal, app, tick_rate);
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
