use overview::{start_background_update,Process};
mod ctrl;
use std::cmp::Reverse;
use std::sync::{Mutex,Arc};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub use ctrl::kill_process;
pub use ctrl::terminate_process;
pub use ctrl::suspend_process;
pub use ctrl::resume_process;
pub use ctrl::change_priority;
use libc::{setpriority, PRIO_PROCESS};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Direction},
    style::{palette::tailwind, Color, Style, Styled, Stylize, Modifier},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Tabs, Table, Widget, Gauge},    DefaultTerminal,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use color_eyre::Result;
mod overview;
pub use overview::print_process;
pub use overview::get_processes;

mod cpuUsage;
pub use cpuUsage::CpuUsage;
pub use cpuUsage::cpu_result;
mod Memory;
use Memory::MemoryUsage;
use Memory::Mem_Usage;
use Memory::start_background_update_mem;

mod IO;
use IO::DiskUsage;
use IO::Disk_Usage;
use IO::start_background_update_io;

use cpuUsage::start_background_update_cpu;


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
    let mut app = App::default();
    
    
    start_background_update_mem(Arc::clone(&app.memory_usage));
    start_background_update(Arc::clone(&app.process_data));
    start_background_update_io(Arc::clone(&app.disk_usage));
    start_background_update_cpu(Arc::clone(&app.cpu_usage));
    let app_result: std::result::Result<(), color_eyre::eyre::Error> = app.run(terminal);
    ratatui::restore();
    app_result.unwrap();
}

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    selected_row: usize,
    selected_setting: usize,
    is_cursed: bool,
    setting_cursed: bool,
    is_sortMode: bool,
    whichSort: Arc<Mutex<i8>>,
    sort: bool,
    pub vertical_scroll: usize,
    pub process_data: Arc<Mutex<Vec<Process>>>,
    pub memory_usage: Arc<Mutex<MemoryUsage>>,
    pub disk_usage: Arc<Mutex<DiskUsage>>,
    pub cpu_usage: Arc<Mutex<Vec<CpuUsage>>>,
    pub column_visibility: Arc<Mutex<Vec<bool>>>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
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
    #[strum(to_string = "Memory/IO")]
    Tab3,
    #[strum(to_string = "Settings")]
    Tab4,

}


impl Default for App {
    fn default() -> Self {
        let mut column_visibility = vec![true; 14]; // Initialize with the same length as column_names
        column_visibility.push(false);
        let whichSort = 28 as i8;
        App {
            state: AppState::Running,
            selected_tab: SelectedTab::Tab1,
            selected_row: 0,
            selected_setting: 0,
            is_cursed: false,
            setting_cursed: false,
            is_sortMode: false,
            whichSort: Arc::new(Mutex::new(whichSort)),
            sort: false,
            vertical_scroll: 0,
            process_data: Arc::new(Mutex::new(Vec::new())),
            memory_usage: Arc::new(Mutex::new(MemoryUsage::default())),
            disk_usage: Arc::new(Mutex::new(DiskUsage::default())),
            cpu_usage: Arc::new(Mutex::new(Vec::new())),
            column_visibility: Arc::new(Mutex::new(column_visibility)),
        }
    }
}

impl App {
    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(100); 
    
        while self.state == AppState::Running {
            // Force redraw periodically
            if last_update.elapsed() >= update_interval {
                terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;
                last_update = Instant::now();
            }
            
            // Check for input events
            if event::poll(Duration::from_millis(10))? {
                self.handle_events()?;
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Right => 
                    {
                        self.next_tab(); 
                        self.is_cursed = false; 
                        self.setting_cursed = false; 
                        self.sort = false;
                    },
                    KeyCode::Left => {self.previous_tab(); self.is_cursed = false; self.setting_cursed = false; self.sort = false;},
                    KeyCode::Char(' ') if self.selected_tab == SelectedTab::Tab4 && self.setting_cursed => 
                    {
                        self.toggle();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    KeyCode::Char('c') if self.selected_tab == SelectedTab::Tab1 || self.selected_tab == SelectedTab::Tab4 =>self.curse(),
                    KeyCode::Up =>
                    {
                        if self.is_cursed || self.setting_cursed {
                            self.move_cursor_up();
                        } 
                        else{ 
                            self.scroll_up();
                        } 
                    },  
                    KeyCode::Down =>{
                        if self.is_cursed || self.setting_cursed {
                            self.move_cursor_down();
                        }
                        else {
                            self.scroll_down()
                        }
                    }, 
                    KeyCode::Char('k') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.kill(),
                    KeyCode::Char('s') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.suspend(),
                    KeyCode::Char('r') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.resume(),
                    KeyCode::Char('t') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.terminate(),
                    KeyCode::Char('u') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.priority_up(),
                    KeyCode::Char('d') if self.is_cursed && self.selected_tab == SelectedTab::Tab1=> self.priority_down(),
                    KeyCode::Char('s') if self.selected_tab == SelectedTab::Tab1 => self.is_sortMode = !self.is_sortMode,
                    KeyCode::Char('w') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => {self.sort = !self.sort; self.sorting('o');},
                    KeyCode::Char('0') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('p'),
                    KeyCode::Char('1') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('u'),
                    KeyCode::Char('2') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('c'),
                    KeyCode::Char('3') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('v'),
                    KeyCode::Char('4') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('r'),
                    KeyCode::Char('5') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('s'),
                    KeyCode::Char('6') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('m'),
                    KeyCode::Char('7') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('C'),
                    KeyCode::Char('8') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('t'),
                    KeyCode::Char('9') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('l'),
                    KeyCode::Char('a') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('n'),
                    KeyCode::Char('b') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('P'),
                    KeyCode::Char('c') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('a'),
                    KeyCode::Char('d') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('S'),
                    KeyCode::Char('e') if self.is_sortMode && self.selected_tab == SelectedTab::Tab1 => self.sorting('T'),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn sorting(&mut self, s: char) {
        let mut data = {
            let data = self.process_data.lock().unwrap();
            data.clone()
        };
        let mut whichSort = {
            let data = self.whichSort.lock().unwrap();
            data.clone()
        };
    
        match s {
            'o' => if (whichSort % 2 == 0) {whichSort += 1;} else {whichSort -= 1;},
            'p' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 0;
                    Reverse(process.pid)
                } else {
                    whichSort = 1;
                    Reverse(-process.pid) // Reverse with a negative ensures descending order
                }
            }),
            'u' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 2;
                    Reverse(process.user.clone())
                } else {
                    whichSort = 3;
                    Reverse(process.user.clone())
                }
            }),
            'c' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 4;
                    Reverse(process.command.clone())
                } else {
                    whichSort = 5;
                    Reverse(process.command.clone())
                }
            }),
            'v' => data.sort_by(|a, b| {
                if self.sort {
                    whichSort = 6;
                    b.v_memory.partial_cmp(&a.v_memory).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    whichSort = 7;
                    a.v_memory.partial_cmp(&b.v_memory).unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            'r' => data.sort_by(|a, b| {
                if self.sort {
                    whichSort = 8;
                    b.rss_memory.partial_cmp(&a.rss_memory).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    whichSort = 9;
                    a.rss_memory.partial_cmp(&b.rss_memory).unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            's' => data.sort_by(|a, b| {
                if self.sort {
                    whichSort = 10;
                    b.shared_memory.partial_cmp(&a.shared_memory).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    whichSort = 11;
                    a.shared_memory.partial_cmp(&b.shared_memory).unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            'm' => data.sort_by(|a, b| {
                if self.sort {
                    whichSort = 12;
                    b.memory_uasge.partial_cmp(&a.memory_uasge).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    whichSort = 13;
                    a.memory_uasge.partial_cmp(&b.memory_uasge).unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            'C' => data.sort_by(|a, b| {
                if self.sort {
                    whichSort = 14;
                    b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    whichSort = 15;
                    a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            't' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 16;
                    Reverse(process.time.clone())
                } else {
                    whichSort = 17;
                    Reverse(process.time.clone())
                }
            }),
            'l' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 18;
                    Reverse(process.priority)
                } else {
                    whichSort = 19;
                    Reverse(-process.priority)
                }
            }),
            'n' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 20;
                    Reverse(process.nice)
                } else {
                    whichSort = 21;
                    Reverse(-process.nice)
                }
            }),
            'P' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 22;
                    Reverse(process.ppid)
                } else {
                    whichSort = 23;
                    Reverse(-process.ppid)
                }
            }),
            'S' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 24;
                    Reverse(process.state.clone())
                } else {
                    whichSort = 25;
                    Reverse(process.state.clone())
                }
            }),
            'T' => data.sort_by_key(|process| {
                if self.sort {
                    whichSort = 26;
                    Reverse(process.threads)
                } else {
                    whichSort = 27;
                    Reverse(-process.threads)
                }
            }),
            _ => {}
        }
    
        // Save the sorted data back if necessary
        *self.process_data.lock().unwrap() = data;
        *self.whichSort.lock().unwrap() = whichSort;
    }
    
    
    

    pub fn scroll_up(&mut self) {
        if self.vertical_scroll > 0 {
            self.vertical_scroll -= 1;
        }
    }
    pub fn scroll_down(&mut self) {
        let data = self.process_data.lock().unwrap();
        let process_count = data.len();
        if self.vertical_scroll < process_count.saturating_sub(1) {
            self.vertical_scroll += 1;
        }
    }

    pub fn priority_up(&mut self) {
        let data = self.process_data.lock().unwrap();
    
        let filtered_data: Vec<&Process> = data.iter()
            .filter(|process| process.user != "root")
            .collect();
    
        if let Some(process) = filtered_data.get(self.selected_row) {
            let pid = process.pid as libc::pid_t;
    
            // Get the current priority
            let current_priority = unsafe { libc::getpriority(PRIO_PROCESS, pid as u32) };
    
            // Convert to true priority range (-20 to 19)
            let true_priority = current_priority - 20;
    
            // Calculate the new priority and clamp it
            let new_priority = (true_priority - 1).clamp(-20, 19);
    
            // Set the new priority
            let result = unsafe { libc::setpriority(PRIO_PROCESS, pid as u32, new_priority + 20) };
        }
    }
    pub fn priority_down(&mut self) {
        let data = self.process_data.lock().unwrap();

        let filtered_data: Vec<&Process> = data.iter()
        .filter(|process| process.user != "root")
        .collect();
        if let Some(process) = filtered_data.get(self.selected_row) {
            let pid = process.pid as i16;
            let current_priority = unsafe { libc::getpriority(PRIO_PROCESS, pid as u32) };
            let new_priority = (current_priority + 1).clamp(-20, 19);
            let result = unsafe { setpriority(PRIO_PROCESS, pid as u32, new_priority)};
        }
    }

    pub fn kill(&mut self) {
        
        let data = self.process_data.lock().unwrap();

        let filtered_data: Vec<&Process> = data.iter()
        .filter(|process| process.user != "root")
        .collect();

        if let Some(process) = filtered_data.get(self.selected_row) {
            let pid = process.pid as i16; 
            let p_id = Pid::from_raw(pid as i32);
            if let Err(err) = kill(p_id, Signal::SIGKILL) {
                eprintln!("Failed to send signal to process {}: {}", pid, err);
            }
        }
    }
    pub fn toggle(&mut self) {
        let mut data = self.column_visibility.lock().unwrap();
        data[self.selected_setting] = !data[self.selected_setting];


    }
    pub fn terminate(&mut self) {
        
        let data = self.process_data.lock().unwrap();

        let filtered_data: Vec<&Process> = data.iter()
        .filter(|process| process.user != "root")
        .collect();

        if let Some(process) = filtered_data.get(self.selected_row) {
            let pid = process.pid as i16; 
            let p_id = Pid::from_raw(pid as i32);
            if let Err(err) = kill(p_id, Signal::SIGTERM) {
                eprintln!("Failed to send signal to process {}: {}", pid, err);
            }
        }
    }
    pub fn resume(&mut self) {
        
        let data = self.process_data.lock().unwrap();

        let filtered_data: Vec<&Process> = data.iter()
        .filter(|process| process.user != "root")
        .collect();

        if let Some(process) = filtered_data.get(self.selected_row) {
            let pid = process.pid; 
            let p_id = Pid::from_raw(pid as i32);
            if let Err(err) = kill(p_id, Signal::SIGCONT) {
                eprintln!("Failed to send signal to process {}: {}", pid, err);
            }
        }
    }
    pub fn suspend(&mut self) {
        
        let data = self.process_data.lock().unwrap();

        let filtered_data: Vec<&Process> = data.iter()
        .filter(|process| process.user != "root")
        .collect();

        if let Some(process) = filtered_data.get(self.selected_row) {
            let pid = process.pid; 
            let p_id = Pid::from_raw(pid as i32);
            if let Err(err) = kill(p_id, Signal::SIGSTOP) {
                eprintln!("Failed to send signal to process {}: {}", pid, err);
            }
        }
    }

    pub fn curse(&mut self) {
        if (self.selected_tab == SelectedTab::Tab1){
            self.is_cursed = !self.is_cursed;
        }
        else if (self.selected_tab == SelectedTab::Tab4){
            self.setting_cursed = !self.setting_cursed;
        }
        
        self.selected_row = self.vertical_scroll;
    
        let data = self.process_data.lock().unwrap();
        let process_count = data.len();
        if self.selected_row >= process_count {
            self.selected_row = process_count.saturating_sub(1);
        }
        self.sort = false;
    }
    
    pub fn move_cursor_up(&mut self) {
        if self.selected_tab == SelectedTab::Tab1 && self.selected_row > 0 {
            self.selected_row -= 1;  
        }
        else if self.selected_tab == SelectedTab::Tab4 && self.selected_setting > 0 {
            self.selected_setting -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.selected_tab == SelectedTab::Tab1 {
            let data = self.process_data.lock().unwrap();
            let process_count = data.len();
            if self.selected_row < process_count.saturating_sub(1) {
                self.selected_row += 1;  
            }
        }
        else if self.selected_tab == SelectedTab::Tab4 {
            if self.selected_setting < 15 {
                self.selected_setting += 1;
            }
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
        render_footer(footer_area, buf, self.selected_tab, self.is_cursed, self.is_sortMode);
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

fn render_footer(area: Rect, buf: &mut Buffer, selected_tab: SelectedTab, cursor:bool, sorting: bool) {
    if cursor && selected_tab == SelectedTab::Tab1 {
        Line::raw("← → to change tab | Press q to quit | Press c to cursor | ↑ ↓ to move | k to kill | t to terminate | s to suspend | r to resume | u to increase priority | d to decrease priority")
        .centered()
        .render(area, buf);
    }
    else if selected_tab == SelectedTab::Tab1 && sorting {
        Line::raw("← → to change tab | Press q to quit | Press c to cursor | Press w to switch sort | Press 0-9 & a-e to sort by column")
        .centered()
        .render(area, buf);
    }
    else if selected_tab == SelectedTab::Tab1 {
        
        Line::raw("← → to change tab | Press q to quit | Press s to sort | Press c to cursor")
        .centered()
        .render(area, buf);
    }
    else if (selected_tab == SelectedTab::Tab4) {
        Line::raw("← → to change tab | Press q to quit | Press c to cursor | Press space to toggle setting")
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
            Self::Tab1 => render_processes(area, buf, app.selected_row, app.is_cursed, app, app.vertical_scroll),
            Self::Tab2 => render_cpu(area, buf, app.cpu_usage.clone()),
            Self::Tab3 => render_memory(area, buf, app.memory_usage.clone(), app.disk_usage.clone()),
            Self::Tab4 => render_settings(area, buf, app.column_visibility.clone(), app.setting_cursed, app.selected_setting),
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
            Self::Tab4 => tailwind::AMBER,
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

fn render_processes(
    area: Rect,
    buf: &mut Buffer,
    selected_row: usize,
    is_cursed: bool,
    app: &App,
    vertical_scroll: usize,
) {
    let mut filtered_data: Vec<Process> = {
        let data = app.process_data.lock().unwrap();
        data.iter()
            .filter(|process| process.user != "root")
            .cloned()
            .collect()
    };
    let s = {
        let data = app.whichSort.lock().unwrap();
        data.clone()
    };

    match s {
        0 => filtered_data.sort_by_key(|process| {
                Reverse(process.pid)
            }),
        1 => filtered_data.sort_by_key(|process| {
                Reverse(-process.pid)
            }),
        2 => filtered_data.sort_by_key(|process| {
                Reverse(process.user.clone())
            }),
        3 => filtered_data.sort_by_key(|process| {
                Reverse(process.user.clone())
            }),
        4 => filtered_data.sort_by_key(|process| {
                Reverse(process.command.clone())
            }),
        5 => filtered_data.sort_by_key(|process| {
                Reverse(process.command.clone())
            }),
        6 => filtered_data.sort_by(|a, b| {
                b.v_memory.partial_cmp(&a.v_memory).unwrap_or(std::cmp::Ordering::Equal)
            }),
        7 => filtered_data.sort_by(|a, b| {
                a.v_memory.partial_cmp(&b.v_memory).unwrap_or(std::cmp::Ordering::Equal)
            }),
        8 => filtered_data.sort_by(|a, b| {
                b.rss_memory.partial_cmp(&a.rss_memory).unwrap_or(std::cmp::Ordering::Equal)
            }),
        9 => filtered_data.sort_by(|a, b| {
                a.rss_memory.partial_cmp(&b.rss_memory).unwrap_or(std::cmp::Ordering::Equal)
            }),
        10 => filtered_data.sort_by(|a, b| {
                b.shared_memory.partial_cmp(&a.shared_memory).unwrap_or(std::cmp::Ordering::Equal)
            }),
        11 => filtered_data.sort_by(|a, b| {
                a.shared_memory.partial_cmp(&b.shared_memory).unwrap_or(std::cmp::Ordering::Equal)
            }),
        12 => filtered_data.sort_by(|a, b| {
                b.memory_uasge.partial_cmp(&a.memory_uasge).unwrap_or(std::cmp::Ordering::Equal)
            }),
        13 => filtered_data.sort_by(|a, b| {
                a.memory_uasge.partial_cmp(&b.memory_uasge).unwrap_or(std::cmp::Ordering::Equal)
            }),
        14 => filtered_data.sort_by(|a, b| {
                b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            }),
        15 => filtered_data.sort_by(|a, b| {
                a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            }),
        16 => filtered_data.sort_by_key(|process| {
                Reverse(process.time.clone())
            }),
        17 => filtered_data.sort_by_key(|process| {
                Reverse(process.time.clone())
            }),
        18 => filtered_data.sort_by_key(|process| {
                Reverse(process.priority)
            }),
        19 => filtered_data.sort_by_key(|process| {
                Reverse(-process.priority)
            }),
        20 => filtered_data.sort_by_key(|process| {
                Reverse(process.nice)
            }),
        21 => filtered_data.sort_by_key(|process| {
                Reverse(-process.nice)
            }),
        22 => filtered_data.sort_by_key(|process| {
                Reverse(process.ppid)
            }),
        23 => filtered_data.sort_by_key(|process| {
                Reverse(-process.ppid)
            }),
        24 => filtered_data.sort_by_key(|process| {
                Reverse(process.state.clone())
            }),
        25 => filtered_data.sort_by_key(|process| {
                Reverse(process.state.clone())
            }),
        26 => filtered_data.sort_by_key(|process| {
                Reverse(process.threads)
            }),
        27 => filtered_data.sort_by_key(|process| {
                Reverse(-process.threads)
            }),
        _ => {}
    }

    let test = 
    {
        *app.process_data.lock().unwrap() = filtered_data.clone();
    };
    

    let is_tree_view: bool = {
        let data = app.column_visibility.lock().unwrap();
        data.clone()[14]
    };

    // Group processes by `ppid`
    let mut process_tree: HashMap<u32, Vec<Process>> = HashMap::new();
    for process in &filtered_data {
        process_tree
            .entry(process.ppid as u32)
            .or_insert_with(Vec::new)
            .push(process.clone());
    }

    // Detect root processes dynamically
    let root_processes: Vec<&Process> = filtered_data
        .iter()
        .filter(|process| !filtered_data.iter().any(|p| p.pid == process.ppid))
        .collect();

    // Recursive function to render the tree view
    fn render_tree(
        processes: &[&Process],
        process_tree: &HashMap<u32, Vec<Process>>,
        depth: usize,
        current_row: &mut usize,
        selected_row: usize,
        vertical_scroll: usize,
        is_cursed: bool,
        buf: &mut Buffer,
        area: Rect,
    ) {
        for process in processes {
            if *current_row < vertical_scroll {
                *current_row += 1; // Skip rows outside the visible area
                continue;
            }

            let y = area.y + (*current_row - vertical_scroll) as u16;
            if y >= area.bottom() {
                break; // Stop rendering if we exceed the visible area
            }

            let is_selected = (*current_row - vertical_scroll) == selected_row;
            let style = if is_selected && is_cursed {
                Style::default().fg(Color::Blue).bg(Color::LightGreen).bold()
            } else if is_selected {
                Style::default().fg(Color::Blue).bold()
            } else {
                Style::default()
            };

            let indent = "  ".repeat(depth);
            let process_line = format!(
                "{}PID: {} | User: {} | Cmd: {} | CPU: {:.2}% | Mem: {:.2} MB",
                indent, process.pid, process.user, process.command, process.cpu_usage, process.v_memory
            );

            buf.set_string(area.x, y, process_line, style);
            *current_row += 1;

            if let Some(children) = process_tree.get(&(process.pid as u32)) {
                let children_refs: Vec<&Process> = children.iter().collect();
                render_tree(
                    &children_refs,
                    process_tree,
                    depth + 1,
                    current_row,
                    selected_row,
                    vertical_scroll,
                    is_cursed,
                    buf,
                    area,
                );
            }
        }
    }

    // Start rendering
    let mut current_row = 0;

    if is_tree_view {
        // Tree view rendering
        render_tree(
            &root_processes,
            &process_tree,
            0,
            &mut current_row,
            selected_row,
            vertical_scroll,
            is_cursed,
            buf,
            area,
        );
    } else {
    
        let column_data: Vec<bool> = {
            let data = app.column_visibility.lock().unwrap();
            data.clone()
        };
        
        let max_visible_rows = (area.height as usize) - 2;
        let start_index = vertical_scroll;
        let end_index = std::cmp::min(start_index + max_visible_rows, filtered_data.len()); 
        let rows: Vec<Row> = filtered_data[start_index..end_index].iter().enumerate().map(|(index, process)|
        {   
            let global_index = start_index + index;
            let is_selected = global_index == selected_row;
            let style = if is_selected && is_cursed {
                Style::default()
                    .fg(Color::Blue).bold()  
                    .bg(Color::LightGreen)  
                     
            } else {
                Style::default() 
            };
            let cells = vec![
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
            ].into_iter()
            .enumerate()
            .filter_map(|(i, data)| {
                if column_data[i] {
                    Some(Cell::from(data).style(style))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
            Row::new(cells)
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
    
        let headerCells = vec![
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
        ].into_iter()
        .enumerate()
        .filter_map(|(i, data)| {
            if column_data[i] {
                Some(Cell::from(data).style(Style::default()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
        let table = Table::new(rows, widths)
            .header(Row::new(headerCells))
            .block(Block::default().borders(Borders::ALL).title("Processes"))
            .widths(&widths);
    
        table.render(area, buf);
    }
}


fn render_cpu(area: Rect, buf: &mut Buffer, cpu: Arc<Mutex<Vec<CpuUsage>>>) {
    let cpu_usages: Vec<CpuUsage> = {
        let data = cpu.lock().unwrap();
        data.clone()
    };
        
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

fn render_memory(area: Rect, buf: &mut Buffer, memory_usage: Arc<Mutex<MemoryUsage>>, disk: Arc<Mutex<DiskUsage>>) {
    let memory = memory_usage.lock().unwrap();
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

    let disk_usage = disk.lock().unwrap();
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


fn render_settings(area: Rect, buf: &mut Buffer, column_visibility: Arc<Mutex<Vec<bool>>>, is_cursed: bool, selected_row: usize) {
    let column_names = vec![
        "PID", "User", "Command", "Virtual Memory", "RSS Memory", "Shared Memory", "Memory Usage",
        "CPU Usage", "Time", "Priority", "Nice", "Parent PID", "State", "Threads", "Tree"
    ];
    let data = column_visibility.lock().unwrap();

    let rows: Vec<Row> = column_names.iter().enumerate().map(|(i, name)| {
        let is_selected = is_cursed && i == selected_row;
        let visibility_marker = if data[i] { "[x]" } else { "[ ]" };

        Row::new(vec![
            Cell::from(visibility_marker),
            Cell::from(name.to_string()),
        ])
        .style(if is_selected {
            Style::default().fg(Color::Blue).bg(Color::LightGreen).bold()
        } else {
            Style::default()
        })
    }).collect();

    let table = Table::new(rows, [Constraint::Length(5), Constraint::Min(20)])
        .block(Block::default().borders(Borders::ALL).title("Settings - Toggle Columns"))
        .widths(&[Constraint::Length(5), Constraint::Min(20)]);

    table.render(area, buf);
}

