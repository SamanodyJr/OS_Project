use overview::Process;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{palette::tailwind, Color, Style, Styled, Stylize, Modifier},
    symbols,
    text::Line,
    widgets::{block::Title, Block, Borders, Cell, Gauge, Padding, Paragraph, Row, Table, Tabs, Widget},
    DefaultTerminal,
};
use std::{sync::{Arc, Mutex}, time::Duration};
use std::thread::sleep;

use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use color_eyre::Result;
mod overview;
pub use overview::print_process;
pub use overview::get_processes;
use tokio::time;

mod cpuUsage;
pub use cpuUsage::CpuUsage;
pub use cpuUsage::cpu_result;
mod Memory;
use Memory::MemoryUsage;
use Memory::Mem_Usage;

mod IO;
use IO::DiskUsage;
use IO::Disk_Usage;

const gaugeBarColor: Color = tailwind::RED.c800;
const gaugeTextColor: Color = tailwind::GREEN.c600;

fn calculate_gauge_color(percent: u16) -> Color {
    match percent {
        0..=20 => tailwind::GREEN.c300,
        21..=40 => tailwind::ORANGE.c500,
        41..=60 => tailwind::ORANGE.c800,
        61..=80 => tailwind::RED.c800,
        _ => tailwind::RED.c900,
    }
}


fn main() {
    let terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> = ratatui::init();
    let app_result: std::result::Result<(), color_eyre::eyre::Error> = App::default().run(terminal);
    ratatui::restore();
    app_result.unwrap();
}

#[derive(Default)]
struct App {
    state: AppState,
    selected_tab: SelectedTab,
    selected_row: usize,
    is_cursed: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Processes")]
    Tab1,
    #[strum(to_string = "CPU")]
    Tab2,
    #[strum(to_string = "Memory")]
    Tab3,

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
                    KeyCode::Char('c') if self.selected_tab == SelectedTab::Tab1 =>self.curse(),
                    KeyCode::Up if self.is_cursed => self.move_cursor_up(),  
                    KeyCode::Down if self.is_cursed => self.move_cursor_down(), 
                    KeyCode::Char('k') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.kill(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn kill(&mut self) {
        
        // let selected_process = processes.get(self.selected_row);
        // if let Some(process) = selected_process {
        //     let pid = process.pid;
        //     let _ = print_process(pid);
        // }
    }

    pub fn curse(&mut self) {
        self.is_cursed = !self.is_cursed;
        self.selected_row = 0;
        
    }
    pub fn move_cursor_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;  
        }
    }

    pub fn move_cursor_down(&mut self) {
        let process_count = get_processes().len();
        if self.selected_row < process_count.saturating_sub(1) {
            self.selected_row += 1;  
        }
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


impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);
        self.selected_tab.render(inner_area, buf, self); 
        render_footer(footer_area, buf, self.selected_tab, self.is_cursed);
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
    "ProcMaster".bold().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer, selected_tab: SelectedTab, cursor:bool) {
    if cursor && selected_tab == SelectedTab::Tab1 {
        Line::raw("← → to change tab | Press q to quit | Press c to cursor | ↑ ↓ to move | k to kill | t to terminate | s to suspend | r to resume | p to set priority")
        .centered()
        .render(area, buf);
    }
    else if selected_tab == SelectedTab::Tab1 {
        
        Line::raw("← → to change tab | Press q to quit | Press c to cursor")
        .centered()
        .render(area, buf);
    }
    else {
        Line::raw("← → to change tab | Press q to quit ")
        .centered()
        .render(area, buf);
    }
}

impl SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer, app: &App) {

        match self {
            Self::Tab1 => render_processes(area, buf, app.selected_row, app.is_cursed),
            Self::Tab2 => render_cpu(area, buf),
            Self::Tab3 => render_memory(area, buf),
        }
    }

    fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Tab1 => tailwind::BLUE,
            Self::Tab2 => tailwind::EMERALD,
            Self::Tab3 => tailwind::INDIGO,
        }
    }

    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

fn render_processes(area: Rect, buf: &mut Buffer, selected_row: usize, is_cursed: bool) {
    let processes: Vec<Process> = get_processes();
    
    let rows: Vec<Row> = processes.iter().enumerate().map(|(index, process)| {
        let is_selected = index == selected_row;
        let style = if is_selected && is_cursed {
            Style::default()
                .fg(Color::Blue).bold()  
                .bg(Color::LightGreen)  
                 
        } else {
            Style::default() 
        };
        
        Row::new(vec![
            Cell::from(process.pid.to_string()).style(style),        
            Cell::from(process.user.clone()).style(style),
            Cell::from(process.command.clone()).style(style),
            Cell::from(format!("{:.2} MB", process.v_memory)).style(style),
            Cell::from(format!("{:.2} MB", process.rss_memory)).style(style),
            Cell::from(format!("{:.2} MB", process.shared_memory)).style(style),
            Cell::from(format!("{:.2}%", process.memory_uasge)).style(style),
            Cell::from(format!("{:.2}%", process.cpu_usage)).style(style),
            Cell::from(process.time.clone()).style(style),
            Cell::from(process.priority.to_string()).style(style),
            Cell::from(process.nice.to_string()).style(style),
            Cell::from(process.ppid.to_string()).style(style),
            Cell::from(process.state.clone()).style(style),
            Cell::from(process.threads.to_string()).style(style),
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


fn render_cpu(area: Rect, buf: &mut Buffer) {
    let cpu_usages: Vec<CpuUsage> = cpu_result();
        
        let gauges: Vec<Gauge> = cpu_usages.iter().map(|cpu_usage| {
            let percent_value = cpu_usage.cpu_usage as u16;
            let label = format!("{:.1}%", cpu_usage.cpu_usage);
            let gauge_color = calculate_gauge_color(percent_value);

            Gauge::default()
                .block(Block::default().title(format!("CPU {} Usage", cpu_usage.core_number)).borders(Borders::ALL))
                .gauge_style(gauge_color)
                .percent(percent_value as u16)
                .label(label)
                .set_style(Style::default().fg(gaugeTextColor))
        }).collect();

        // Split the area into two columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // Split each column into rows for the gauges
        let left_column_constraints: Vec<Constraint> = vec![Constraint::Length((gauges.len() / 2) as u16); gauges.len() / 2];
        let right_column_constraints: Vec<Constraint> = vec![Constraint::Length((gauges.len() / 2) as u16); gauges.len() / 2];

        let left_column_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(left_column_constraints)
            .split(columns[0]);

        let right_column_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(right_column_constraints)
            .split(columns[1]);

        // Render the gauges in the left column
        for (i, gauge) in gauges.iter().take(gauges.len() / 2).enumerate() {
            gauge.render(left_column_chunks[i], buf);
        }

        // Render the gauges in the right column
        for (i, gauge) in gauges.iter().skip(gauges.len() / 2).enumerate() {
            gauge.render(right_column_chunks[i], buf);
        }
    }


fn render_memory(area: Rect, buf: &mut Buffer) {
    let memory = Mem_Usage();
    let gauge_color = calculate_gauge_color(((memory.used / memory.total) * 100.0) as u16);
    let gauge_color_swap = calculate_gauge_color(((memory.used_swap / memory.total_swap) * 100.0) as u16);
    let gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(gauge_color)
        .percent(((memory.used / memory.total) * 100.0) as u16)
        .label(format!("{:.1}%", (memory.used / memory.total) * 100.0))
        .set_style(Style::default().fg(gaugeTextColor));
    let swap_gauge = Gauge::default()
        .block(Block::default().title("Swap Usage").borders(Borders::ALL))
        .gauge_style(gauge_color_swap)
        .percent(((memory.used_swap / memory.total_swap) * 100.0) as u16)
        .label(format!("{:.1}%", (memory.used_swap / memory.total_swap) * 100.0))
        .set_style(Style::default().fg(gaugeTextColor));
    let rows = vec![
        Row::new(vec![
            Cell::from("Total Memory"),
            Cell::from(format!("{:.2} GB", memory.total)),
        ]),
        Row::new(vec![
            Cell::from("Used Memory"),
            Cell::from(format!("{:.2} GB", memory.used)),
        ]),
        Row::new(vec![
            Cell::from("Free Memory").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(format!("{:.2} GB", memory.free)),
        ])];
        let row_swap = vec![
        Row::new(vec![
            Cell::from("Total Swap"),
            Cell::from(format!("{:.2} MB", memory.total_swap)),
        ]),
        Row::new(vec![
            Cell::from("Used Swap"),
            Cell::from(format!("{:.2} MB", memory.used_swap)),
        ]),
        Row::new(vec![
            Cell::from("Free Swap"),
            Cell::from(format!("{:.2} MB", memory.free_swap)),
        ]),
    ];
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    let table = Table::new(rows, [Constraint::Length(20), Constraint::Length(20)])
        .block(Block::default().borders(Borders::ALL).title("Memory"))
        .widths(&[Constraint::Length(20), Constraint::Length(20)]);
    let table_swap = Table::new(row_swap, [Constraint::Length(20), Constraint::Length(20)])
        .block(Block::default().borders(Borders::ALL).title("Swap"))
        .widths(&[Constraint::Length(20), Constraint::Length(20)]);
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(10), Constraint::Percentage(10)].as_ref())
        .split(columns[0]);
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(10), Constraint::Percentage(10)].as_ref())
        .split(columns[1]);
    gauge.render(left_chunks[0], buf);
    swap_gauge.render(right_chunks[0], buf);
    table.render(left_chunks[1], buf);
    table_swap.render(right_chunks[1], buf);

    let disk_usage = Disk_Usage();
    let disk_rows = vec![
        Row::new(vec![
            Cell::from("Device Name"),
            Cell::from(disk_usage.device_name.clone()),
        ]),
        Row::new(vec![
            Cell::from("Reads Completed"),
            Cell::from(disk_usage.reads_completed.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Time Reading"),
            Cell::from(disk_usage.time_reading.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Writes Completed"),
            Cell::from(disk_usage.writes_completed.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Time Writing"),
            Cell::from(disk_usage.time_writing.to_string()),
        ]),
        Row::new(vec![
            Cell::from("I/O in Progress"),
            Cell::from(disk_usage.io_in_progress.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Time I/O"),
            Cell::from(disk_usage.time_io.to_string()),
        ]),
    ];
    let disk_table1 = Table::new(
        disk_rows.iter().take(disk_rows.len() / 2).cloned().collect::<Vec<_>>(),
        [Constraint::Length(20), Constraint::Length(20)],
    )
    .block(Block::default().borders(Borders::ALL).title("Disk Usage Part 1"))
    .widths(&[Constraint::Length(20), Constraint::Length(20)]);

    let disk_table2 = Table::new(
        disk_rows.iter().skip(disk_rows.len() / 2).cloned().collect::<Vec<_>>(),
        [Constraint::Length(20), Constraint::Length(20)],
    )
    .block(Block::default().borders(Borders::ALL).title("Disk Usage Part 2"))
    .widths(&[Constraint::Length(20), Constraint::Length(20)]);

    disk_table1.render(left_chunks[2], buf);
    disk_table2.render(right_chunks[2], buf);
    

}


