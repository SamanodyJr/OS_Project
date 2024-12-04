use overview::Process;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Stylize, Style},
    symbols,
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table, Padding, Paragraph, Tabs, Widget, ListItem, List},
    DefaultTerminal,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use color_eyre::Result;
mod overview;
pub use overview::print_process;
pub use overview::get_processes;
use tokio::time::{self, Duration};

fn main() {
    let terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> = ratatui::init();
    let app_result: std::result::Result<(), color_eyre::eyre::Error> = App::default().run(terminal);
    ratatui::restore();
    app_result.unwrap();
}

#[derive(Default)]
struct App {
    state: AppState,
    selected_tab: usize,
    is_cursed: bool,
    processes: Vec<Process>,
    selected_process_index: Option<usize>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.processes = get_processes();
        while self.state == AppState::Running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
                    KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    KeyCode::Char('c') if self.selected_tab == 0 => self.curse(),
                    KeyCode::Up if self.is_cursed => self.curseup(),
                    KeyCode::Down if self.is_cursed => self.cursedown(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 4;
    }

    pub fn previous_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = 3;
        } else {
            self.selected_tab -= 1;
        }
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quitting;
    }

    pub fn curse(&mut self) {
        self.is_cursed = !self.is_cursed;
    }

    pub fn curseup(&mut self) {
        if let Some(index) = self.selected_process_index {
            self.selected_process_index = Some(index.saturating_sub(1));
        }
    }

    pub fn cursedown(&mut self) {
        if let Some(index) = self.selected_process_index {
            self.selected_process_index = Some(index.saturating_add(1));
        }
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = vec!["Processes", "CPU", "Memory", "I/O"]
            .into_iter()
            .map(|title| Line::raw(format!("  {}  ", title)).fg(tailwind::SLATE.c200));
        let highlight_style = (Color::default(), tailwind::SLATE.c900);
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(self.selected_tab)
            .render(area, buf);
    }

    fn render_processes_tab(&self, area: Rect, buf: &mut Buffer) {
        let list_items: Vec<ListItem> = self.processes.iter().enumerate().map(|(i, process)| {
            let mut style = Style::default();
            if Some(i) == self.selected_process_index && self.is_cursed {
                style = style.bg(Color::Red).fg(Color::Yellow);
            }

            let process_info = format!(
                "PID: {:<8} User: {:<10} Command: {:<35} CPU: {:.2}% Memory: {:.2} MB",
                process.pid,
                process.user,
                process.command,
                process.cpu_usage,
                process.rss_memory
            );
            ListItem::new(process_info).style(style)
        }).collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("Processes"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("-> ");
        
        list.render(area, buf);
    }

    fn render_cpu_tab(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("CPU Tab Content").block(self.tab_block()).render(area, buf);
    }

    fn render_memory_tab(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Memory Tab Content").block(self.tab_block()).render(area, buf);
    }

    fn render_io_tab(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("I/O Tab Content").block(self.tab_block()).render(area, buf);
    }

    fn tab_block(&self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(tailwind::SLATE.c700)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);

        match self.selected_tab {
            0 => self.render_processes_tab(inner_area, buf),
            1 => self.render_cpu_tab(inner_area, buf),
            2 => self.render_memory_tab(inner_area, buf),
            3 => self.render_io_tab(inner_area, buf),
            _ => {}
        }

        render_footer(footer_area, buf, self.selected_tab);
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    "Insert project title here (i forgot)".bold().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer, selected_tab: usize) {
    if selected_tab == 0 {
        Line::raw("◄ ► to change tab | Press q to quit | Press c to curse")
            .centered()
            .render(area, buf);
    } else {
        Line::raw("◄ ► to change tab | Press q to quit")
            .centered()
            .render(area, buf);
    }
}
