
use overview::Process;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table, Padding, Paragraph, Tabs, Widget},
    DefaultTerminal,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use color_eyre::Result;
mod overview;
pub use overview::print_process;
pub use overview::get_processes;
use tokio::time::{self, Duration};
mod cpuUsage;
pub use cpuUsage::calculate_cpu_usage;


fn main() {
    let terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> = ratatui::init();
    let app_result:std::result::Result<(), color_eyre::eyre::Error>= App::default().run(terminal);
    ratatui::restore();
    app_result.unwrap();
    
}


#[derive(Default)]
struct App {
    state: AppState,
    selected_tab: SelectedTab,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Processes")]
    Tab1,
    #[strum(to_string = "CPU")]
    Tab2,
    #[strum(to_string = "Memory")]
    Tab3,
    #[strum(to_string = "I/O")]
    Tab4,
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
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
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
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
        self.selected_tab.render(inner_area, buf);
        render_footer(footer_area, buf);
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Color::default(), self.selected_tab.palette().c700);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    "Insert project title here (i forgot)".bold().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::raw("◄ ► to change tab | Press q to quit")
        .centered()
        .render(area, buf);
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // in a real app these might be separate widgets
        match self {
            Self::Tab1 => self.render_tab0(area, buf),
            Self::Tab2 => self.render_tab1(area, buf),
            Self::Tab3 => self.render_tab2(area, buf),
            Self::Tab4 => self.render_tab3(area, buf),
        }
    }
}

impl SelectedTab {
    /// Return tab's name as a styled `Line`
    fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    fn render_tab0(self, area: Rect, buf: &mut Buffer) {
        let processes: Vec<Process> = get_processes();

        let rows: Vec<Row> = processes.iter().map(|process| {
            Row::new(vec![
                Cell::from(process.pid.to_string()),
                Cell::from(process.user.clone()),
                Cell::from(process.command.clone()),
                Cell::from(format!("{:.2} MB", process.v_memory)),
                Cell::from(format!("{:.2} MB", process.rss_memory)),
                Cell::from(format!("{:.2} MB", process.shared_memory)),
                Cell::from(format!("{:.2}%", process.memory_uasge)),
                Cell::from(format!("{:.2}%", process.cpu_usage)),
                Cell::from(process.time.clone()),
                Cell::from(process.priority.to_string()),
                Cell::from(process.nice.to_string()),
                Cell::from(process.ppid.to_string()),
                Cell::from(process.state.clone()),
                Cell::from(process.threads.to_string()),
            ])
        }).collect();

        let widths = [
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(35),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];


        let table = Table::new(rows, widths)
            .header(Row::new(vec![
                Cell::from("PID"),
                Cell::from("User"),
                Cell::from("Command"),
                Cell::from("Virtual Memory"),
                Cell::from("RSS Memory"),
                Cell::from("Shared Memory"),
                Cell::from("Memory Usage"),
                Cell::from("CPU Usage"),
                Cell::from("Time"),
                Cell::from("Priority"),
                Cell::from("Nice"),
                Cell::from("Parent PID"),
                Cell::from("State"),
                Cell::from("Threads"),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Processes"))
            .widths(&widths);

        table.render(area, buf);
    }

    fn render_tab1(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Welcome to the Ratatui tabs example!")
            .block(self.block())
            .render(area, buf);
    }

    fn render_tab2(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Look! I'm different than others!")
            .block(self.block())
            .render(area, buf);
    }

    fn render_tab3(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("I know, these are some basic changes. But I think you got the main idea.")
            .block(self.block())
            .render(area, buf);
    }

    /// A block surrounding the tab's content
    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700)
    }

    const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Tab1 => tailwind::BLUE,
            Self::Tab2 => tailwind::EMERALD,
            Self::Tab3 => tailwind::INDIGO,
            Self::Tab4 => tailwind::RED,
        }
    }
}