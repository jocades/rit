use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{fs, io, path::PathBuf};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};

struct App {
    items: Vec<PathBuf>,
    selected_item: usize,
    content: String,
}

impl App {
    fn new() -> io::Result<App> {
        let mut items = fs::read_dir(".")?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect::<Vec<_>>();

        items.sort();

        Ok(App {
            items,
            selected_item: 0,
            content: String::from("This is the content of the selected item."),
        })
    }

    fn next(&mut self) {
        self.selected_item = (self.selected_item + 1) % self.items.len();
        self.update_content();
    }

    fn previous(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        } else {
            self.selected_item = self.items.len() - 1;
        }
        self.update_content();
    }

    fn update_content(&mut self) {
        let path = &self.items[self.selected_item];
        if path.is_file() {
            self.content =
                fs::read_to_string(path).unwrap_or_else(|_| "Could not read file".into());
        } else {
            self.content = "Selected item is not a file".into()
        }
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = app
                .items
                .iter()
                .enumerate()
                .map(|(i, path)| {
                    let name = path.file_name().unwrap().to_string_lossy();
                    let style = if i == app.selected_item {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Spans::from(vec![Span::styled(name, style)]))
                })
                .collect();

            let items =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Files"));

            f.render_widget(items, chunks[0]);

            let content = Paragraph::new(app.content.as_ref())
                .block(Block::default().borders(Borders::ALL).title("Content"))
                .wrap(Wrap { trim: true });

            f.render_widget(content, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') => app.next(),
                KeyCode::Char('k') => app.previous(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
