use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use anyhow::{Result, anyhow};
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::{
    core::{
        archive::{build_archive_plan, destination_preview},
        config::{ConfigStore, validate_date_format, validate_destination_root},
        copy::{CopyProgress, CopySummary, execute_copy, plan_copy},
        types::{ArchiveSettings, DeviceAvailability, DeviceInfo, ImportSession},
    },
    platform::discovery::{DeviceDiscovery, SystemDeviceDiscovery, validate_selected_device},
};

const HELP_TEXT: &str = "Ctrl+Q quit | Ctrl+R refresh | Ctrl+S settings | arrows move | Left/Right collapse-expand | Tab cycle focus | Enter confirm/save | Esc back";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Settings,
    Confirmation,
    Copying,
    CopyResults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusField {
    SourceTree,
    Theme,
    DestinationRoot,
    DateFormat,
}

#[derive(Debug, Clone)]
struct SourceEntry {
    device_id: String,
    path: PathBuf,
    label: String,
    depth: usize,
    is_expanded: bool,
    has_children: bool,
    is_loading: bool,
    is_device_root: bool,
}

#[derive(Debug, Clone)]
enum DirectoryLoadState {
    Loading,
    Loaded(Vec<PathBuf>),
}

pub fn run_app() -> Result<()> {
    let config_store = ConfigStore::new()?;
    let mut app = App::new(config_store)?;
    let mut terminal = setup_terminal()?;

    let outcome = run_loop(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    outcome
}

fn run_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        app.poll_background_updates();
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.handle_key(key)? {
                break;
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title =
        Paragraph::new("Sorted").block(Block::default().title("Sorted").borders(Borders::ALL));
    frame.render_widget(title, layout[0]);

    match app.screen {
        Screen::Main => draw_main(frame, app, layout[1]),
        Screen::Settings => draw_settings(frame, app, layout[1]),
        Screen::Confirmation => draw_confirmation(frame, app, layout[1]),
        Screen::Copying => draw_copying(frame, app, layout[1]),
        Screen::CopyResults => draw_results(frame, app, layout[1]),
    }

    let status = Paragraph::new(app.status_message.clone())
        .block(Block::default().title("Status").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(status, layout[2]);

    let keyboard = Paragraph::new(HELP_TEXT)
        .block(Block::default().title("Keyboard").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(keyboard, layout[3]);
}

fn draw_main(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_source_tree(frame, app, columns[0]);
    draw_session(frame, app, columns[1]);
}

fn draw_source_tree(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let items = if app.source_entries.is_empty() {
        vec![ListItem::new(if app.devices_loading {
            "Scanning devices..."
        } else {
            "No removable devices found"
        })]
    } else {
        app.source_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let prefix = if entry.has_children {
                    if entry.is_expanded { "▾" } else { "▸" }
                } else {
                    "•"
                };
                let loading = if entry.is_loading { " (loading)" } else { "" };
                let indent = "  ".repeat(entry.depth);
                let style = if index == app.source_index && app.focus == FocusField::SourceTree {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if entry.is_device_root {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{indent}{prefix} {}{loading}", entry.label)).style(style)
            })
            .collect()
    };

    let widget = List::new(items).block(
        Block::default()
            .title("Source")
            .borders(Borders::ALL)
            .border_style(if app.focus == FocusField::SourceTree {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );
    frame.render_widget(widget, area);
}

fn draw_session(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let preview = app
        .preview_path()
        .unwrap_or_else(|| "Archive path preview unavailable".to_string());
    let source = app
        .selected_source()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "No source folder selected".to_string());

    let content = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Theme: ", field_style(app, FocusField::Theme)),
            Span::raw(&app.import_session.theme),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Source Folder: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(source),
        ]),
        Line::from(vec![
            Span::styled(
                "Destination: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.settings.destination_root.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "Date Format: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.settings.date_format),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Preview: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(preview),
        ]),
    ])
    .block(
        Block::default()
            .title("Target")
            .borders(Borders::ALL)
            .border_style(if app.focus == FocusField::Theme {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(content, area);
}

fn draw_settings(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let preview = validate_date_format(&app.settings.date_format)
        .map(|date| date.preview)
        .unwrap_or_else(|error| format!("invalid: {error}"));

    let content = Paragraph::new(vec![
        Line::from("Edit settings in-place: typed input is applied to the focused field."),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Destination Root: ",
                field_style(app, FocusField::DestinationRoot),
            ),
            Span::raw(app.settings.destination_root.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Date Format: ", field_style(app, FocusField::DateFormat)),
            Span::raw(app.settings.date_format.clone()),
        ]),
        Line::from(vec![
            Span::styled("Preview: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(preview),
        ]),
        Line::from(""),
        Line::from("Press enter to save settings or esc to return."),
    ])
    .block(Block::default().title("Settings").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
    frame.render_widget(content, area);
}

fn draw_confirmation(frame: &mut Frame<'_>, app: &App, area: Rect) {
    frame.render_widget(Clear, area);
    let preview = app
        .preview_path()
        .unwrap_or_else(|| "Archive path preview unavailable".to_string());
    let source = app
        .selected_source()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "No source folder selected".to_string());
    let modal = Paragraph::new(vec![
        Line::from("Confirm archive import"),
        Line::from(""),
        Line::from(format!("Source: {source}")),
        Line::from(preview),
        Line::from(""),
        Line::from("Press enter to start copy or esc to cancel."),
    ])
    .block(Block::default().title("Confirmation").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
    frame.render_widget(modal, area);
}

fn draw_results(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let result_text = match &app.copy_result {
        Some(result) => {
            let mut lines = vec![
                Line::from(format!(
                    "Copied {} files into {}",
                    result.copied_files,
                    result.destination.display()
                )),
                Line::from(""),
            ];
            if result.failures.is_empty() {
                lines.push(Line::from("No copy failures reported."));
            } else {
                lines.push(Line::from("Failures:"));
                for failure in &result.failures {
                    lines.push(Line::from(format!(
                        "{}: {}",
                        failure.file.display(),
                        failure.error
                    )));
                }
            }
            lines
        }
        None => vec![Line::from("No copy has been run yet.")],
    };

    let widget = Paragraph::new(result_text)
        .block(Block::default().title("Copy Results").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_copying(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let progress_line = match &app.copy_progress {
        Some(progress) => format!(
            "Copied {}/{} files{}",
            progress.copied_files,
            progress.total_files,
            progress
                .current_file
                .as_ref()
                .map(|path| format!(" | {}", path.display()))
                .unwrap_or_default()
        ),
        None => "Preparing copy job...".to_string(),
    };

    let widget = Paragraph::new(vec![
        Line::from("Archive import in progress"),
        Line::from(""),
        Line::from(progress_line),
        Line::from(""),
        Line::from("Progress updates will stay visible here until the copy finishes."),
    ])
    .block(
        Block::default()
            .title("Copy Progress")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn field_style(app: &App, field: FocusField) -> Style {
    if app.focus == field {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn setup_terminal() -> Result<DefaultTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(ratatui::init())
}

fn restore_terminal(terminal: &mut DefaultTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    ratatui::restore();
    Ok(())
}

struct App {
    config_store: ConfigStore,
    settings: ArchiveSettings,
    devices: Vec<DeviceInfo>,
    devices_loading: bool,
    directory_state: HashMap<PathBuf, DirectoryLoadState>,
    expanded_sources: BTreeSet<PathBuf>,
    pending_directory_loads: HashSet<PathBuf>,
    source_entries: Vec<SourceEntry>,
    source_index: usize,
    import_session: ImportSession,
    status_message: String,
    screen: Screen,
    focus: FocusField,
    copy_progress: Option<CopyProgress>,
    copy_result: Option<CopySummary>,
    copy_updates: Option<Receiver<CopyUpdate>>,
    background_updates: Receiver<BackgroundUpdate>,
    background_sender: Sender<BackgroundUpdate>,
}

impl App {
    fn new(config_store: ConfigStore) -> Result<Self> {
        let settings = config_store.load()?;
        let (background_sender, background_updates) = mpsc::channel();
        let mut app = Self {
            config_store,
            settings,
            devices: Vec::new(),
            devices_loading: true,
            directory_state: HashMap::new(),
            expanded_sources: BTreeSet::new(),
            pending_directory_loads: HashSet::new(),
            source_entries: Vec::new(),
            source_index: 0,
            import_session: ImportSession::default(),
            status_message: "Starting up. Scanning devices in the background...".to_string(),
            screen: Screen::Main,
            focus: FocusField::SourceTree,
            copy_progress: None,
            copy_result: None,
            copy_updates: None,
            background_updates,
            background_sender,
        };
        app.request_device_refresh();
        Ok(app)
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
                KeyCode::Char('r') | KeyCode::Char('R') => self.request_device_refresh(),
                KeyCode::Char('s') | KeyCode::Char('S') => self.open_settings(),
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::F(5) => self.request_device_refresh(),
            KeyCode::F(2) => self.open_settings(),
            KeyCode::Enter => self.confirm_or_advance()?,
            KeyCode::Esc => self.go_back(),
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Left => self.handle_left(),
            KeyCode::Right => self.handle_right(),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Char(ch) => self.handle_char(ch),
            _ => {}
        }

        Ok(false)
    }

    fn open_settings(&mut self) {
        if self.screen == Screen::Copying {
            self.status_message =
                "Copy is running. Wait for it to finish before opening settings.".to_string();
        } else {
            self.screen = Screen::Settings;
            self.focus = FocusField::DestinationRoot;
            self.status_message = "Editing persisted settings.".to_string();
        }
    }

    fn request_device_refresh(&mut self) {
        self.devices_loading = true;
        self.status_message = "Refreshing devices in the background...".to_string();
        let sender = self.background_sender.clone();
        thread::spawn(move || {
            let discovery = SystemDeviceDiscovery;
            let result = discovery.discover();
            let _ = sender.send(BackgroundUpdate::DevicesLoaded(result));
        });
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.screen {
            Screen::Main | Screen::Confirmation | Screen::Copying | Screen::CopyResults => {
                match self.focus {
                    FocusField::SourceTree => FocusField::Theme,
                    _ => FocusField::SourceTree,
                }
            }
            Screen::Settings => match self.focus {
                FocusField::DestinationRoot => FocusField::DateFormat,
                _ => FocusField::DestinationRoot,
            },
        };
    }

    fn move_selection(&mut self, delta: isize) {
        if self.screen != Screen::Main || self.focus != FocusField::SourceTree {
            return;
        }
        if self.source_entries.is_empty() {
            return;
        }
        let max = self.source_entries.len() as isize - 1;
        let next = (self.source_index as isize + delta).clamp(0, max) as usize;
        self.source_index = next;
        self.sync_selection_from_index();
    }

    fn handle_left(&mut self) {
        if self.screen != Screen::Main || self.focus != FocusField::SourceTree {
            return;
        }
        let Some(entry) = self.source_entries.get(self.source_index) else {
            return;
        };
        if self.expanded_sources.remove(&entry.path) {
            self.rebuild_source_entries();
            self.source_index = self
                .source_index
                .min(self.source_entries.len().saturating_sub(1));
            self.sync_selection_from_index();
        }
    }

    fn handle_right(&mut self) {
        if self.screen != Screen::Main || self.focus != FocusField::SourceTree {
            return;
        }
        let Some(entry) = self.source_entries.get(self.source_index).cloned() else {
            return;
        };

        if entry.has_children {
            self.expanded_sources.insert(entry.path.clone());
            self.ensure_directory_loaded(&entry.path);
            self.rebuild_source_entries();
            self.sync_selection_from_index();
        }
    }

    fn confirm_or_advance(&mut self) -> Result<()> {
        match self.screen {
            Screen::Main => {
                let selected_device = self
                    .selected_device()
                    .ok_or_else(|| anyhow!("select a source before continuing"))?;
                let selected_device = validate_selected_device(&selected_device, &self.devices)?;
                if !selected_device.is_available() {
                    self.status_message = availability_message(&selected_device);
                    return Ok(());
                }
                let Some(source_root) = self.selected_source() else {
                    self.status_message = "Pick a source folder before continuing.".to_string();
                    return Ok(());
                };
                if matches!(
                    self.directory_state.get(&source_root),
                    Some(DirectoryLoadState::Loading)
                ) {
                    self.status_message =
                        "That folder is still loading. Try again in a moment.".to_string();
                    return Ok(());
                }
                self.import_session.selected_device = Some(selected_device);
                self.screen = Screen::Confirmation;
                self.status_message =
                    "Review the source folder and archive destination before copy starts."
                        .to_string();
            }
            Screen::Settings => {
                validate_destination_root(&self.settings.destination_root)?;
                validate_date_format(&self.settings.date_format)?;
                self.config_store.save(&self.settings)?;
                self.screen = Screen::Main;
                self.focus = FocusField::SourceTree;
                self.status_message = format!(
                    "Saved settings to {}",
                    self.config_store.config_path().display()
                );
            }
            Screen::Confirmation => self.start_copy()?,
            Screen::Copying => {}
            Screen::CopyResults => {
                self.screen = Screen::Main;
                self.status_message = "Returned to import screen.".to_string();
            }
        }
        Ok(())
    }

    fn go_back(&mut self) {
        match self.screen {
            Screen::Main => {}
            Screen::Settings | Screen::Confirmation | Screen::CopyResults => {
                self.screen = Screen::Main;
                self.focus = FocusField::SourceTree;
                self.status_message = "Returned to import screen.".to_string();
            }
            Screen::Copying => {
                self.status_message =
                    "Copy is running. Wait for it to finish before leaving this screen."
                        .to_string();
            }
        }
    }

    fn handle_backspace(&mut self) {
        match self.focus {
            FocusField::Theme => {
                self.import_session.theme.pop();
            }
            FocusField::DestinationRoot if self.screen == Screen::Settings => {
                let updated = trim_last_char(self.settings.destination_root.display().to_string());
                self.settings.destination_root = updated.into();
            }
            FocusField::DateFormat if self.screen == Screen::Settings => {
                self.settings.date_format.pop();
            }
            _ => {}
        }
    }

    fn handle_char(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }

        match self.focus {
            FocusField::Theme if self.screen == Screen::Main => self.import_session.theme.push(ch),
            FocusField::DestinationRoot if self.screen == Screen::Settings => {
                let mut path = self.settings.destination_root.display().to_string();
                path.push(ch);
                self.settings.destination_root = path.into();
            }
            FocusField::DateFormat if self.screen == Screen::Settings => {
                self.settings.date_format.push(ch)
            }
            _ => {}
        }
    }

    fn preview_path(&self) -> Option<String> {
        let device = self.selected_device()?;
        build_archive_plan(
            &self.settings,
            &self.import_session.theme,
            &device,
            Local::now(),
        )
        .ok()
        .map(|plan| destination_preview(&plan))
    }

    fn start_copy(&mut self) -> Result<()> {
        let selected = self
            .selected_device()
            .ok_or_else(|| anyhow!("select a source before importing"))?;
        let selected = validate_selected_device(&selected, &self.devices)?;
        if !selected.is_available() {
            self.status_message = availability_message(&selected);
            self.screen = Screen::Main;
            return Ok(());
        }
        if self.import_session.theme.trim().is_empty() {
            self.status_message = "Enter a theme before starting the import.".to_string();
            self.screen = Screen::Main;
            self.focus = FocusField::Theme;
            return Ok(());
        }
        let source_root = self
            .selected_source()
            .ok_or_else(|| anyhow!("pick a source folder before importing"))?;

        let plan = plan_copy(
            &self.settings,
            &self.import_session.theme,
            &selected,
            &source_root,
            Local::now(),
        )?;
        self.status_message = format!(
            "Copying {} media file(s) from {}",
            plan.files.len(),
            source_root.display()
        );
        self.copy_progress = Some(CopyProgress {
            copied_files: 0,
            total_files: plan.files.len(),
            current_file: None,
        });
        self.copy_result = None;
        self.screen = Screen::Copying;
        let (sender, receiver) = mpsc::channel();
        self.copy_updates = Some(receiver);

        thread::spawn(move || {
            let sender_for_progress = sender.clone();
            let result = execute_copy(&plan, move |progress| {
                let _ = sender_for_progress.send(CopyUpdate::Progress(progress));
            });
            let _ = sender.send(CopyUpdate::Finished(result));
        });
        Ok(())
    }

    fn poll_background_updates(&mut self) {
        while let Ok(update) = self.background_updates.try_recv() {
            match update {
                BackgroundUpdate::DevicesLoaded(result) => {
                    self.devices_loading = false;
                    match result {
                        Ok(devices) => {
                            self.apply_devices(devices);
                        }
                        Err(error) => {
                            self.devices.clear();
                            self.source_entries.clear();
                            self.import_session.selected_device = None;
                            self.import_session.selected_source = None;
                            self.status_message = format!("Device refresh failed: {error}");
                        }
                    }
                }
                BackgroundUpdate::DirectoryLoaded(path, children) => {
                    self.pending_directory_loads.remove(&path);
                    self.directory_state
                        .insert(path.clone(), DirectoryLoadState::Loaded(children));
                    self.rebuild_source_entries();
                    self.status_message = format!("Loaded {}", path.display());
                }
            }
        }

        let mut finished = None;
        if let Some(receiver) = &self.copy_updates {
            while let Ok(update) = receiver.try_recv() {
                match update {
                    CopyUpdate::Progress(progress) => {
                        self.copy_progress = Some(progress.clone());
                        self.status_message = format!(
                            "Copying media files: {}/{} complete",
                            progress.copied_files, progress.total_files
                        );
                    }
                    CopyUpdate::Finished(result) => finished = Some(result),
                }
            }
        }

        if let Some(result) = finished {
            self.copy_updates = None;
            match result {
                Ok(summary) => {
                    self.copy_result = Some(summary.clone());
                    self.screen = Screen::CopyResults;
                    self.status_message = if summary.failures.is_empty() {
                        format!("Copy finished: {} file(s) archived.", summary.copied_files)
                    } else {
                        format!("Copy completed with {} failure(s).", summary.failures.len())
                    };
                }
                Err(error) => {
                    self.copy_result = None;
                    self.screen = Screen::Main;
                    self.status_message = format!("Copy failed: {error}");
                }
            }
        }
    }

    fn apply_devices(&mut self, devices: Vec<DeviceInfo>) {
        let previous_source = self.import_session.selected_source.clone();
        self.devices = devices;
        self.directory_state.clear();
        self.pending_directory_loads.clear();
        self.expanded_sources.clear();
        self.source_entries.clear();
        self.source_index = 0;
        self.import_session.selected_device = None;
        self.import_session.selected_source = None;

        for device in &self.devices {
            self.directory_state
                .insert(device.mount_path.clone(), DirectoryLoadState::Loading);
            self.pending_directory_loads
                .insert(device.mount_path.clone());
            spawn_directory_load(self.background_sender.clone(), device.mount_path.clone());
        }

        self.rebuild_source_entries();

        if let Some(path) = previous_source {
            if let Some(index) = self
                .source_entries
                .iter()
                .position(|entry| entry.path == path)
            {
                self.source_index = index;
            }
        }
        self.sync_selection_from_index();

        self.status_message = if self.devices.is_empty() {
            "No removable devices found.".to_string()
        } else {
            format!("Found {} removable device(s).", self.devices.len())
        };
    }

    fn ensure_directory_loaded(&mut self, path: &Path) {
        if self.directory_state.contains_key(path) || self.pending_directory_loads.contains(path) {
            return;
        }
        self.directory_state
            .insert(path.to_path_buf(), DirectoryLoadState::Loading);
        self.pending_directory_loads.insert(path.to_path_buf());
        spawn_directory_load(self.background_sender.clone(), path.to_path_buf());
    }

    fn rebuild_source_entries(&mut self) {
        let mut entries = Vec::new();
        for device in &self.devices {
            flatten_source_tree(
                device,
                &self.directory_state,
                &self.expanded_sources,
                0,
                &mut entries,
            );
        }
        self.source_entries = entries;
        if self.source_index >= self.source_entries.len() {
            self.source_index = self.source_entries.len().saturating_sub(1);
        }
    }

    fn sync_selection_from_index(&mut self) {
        if let Some(entry) = self.source_entries.get(self.source_index) {
            self.import_session.selected_source = Some(entry.path.clone());
            self.import_session.selected_device = self
                .devices
                .iter()
                .find(|device| device.id == entry.device_id)
                .cloned();
        }
    }

    fn selected_source(&self) -> Option<PathBuf> {
        self.import_session.selected_source.clone()
    }

    fn selected_device(&self) -> Option<DeviceInfo> {
        self.import_session.selected_device.clone()
    }
}

enum BackgroundUpdate {
    DevicesLoaded(Result<Vec<DeviceInfo>>),
    DirectoryLoaded(PathBuf, Vec<PathBuf>),
}

enum CopyUpdate {
    Progress(CopyProgress),
    Finished(Result<CopySummary>),
}

fn flatten_source_tree(
    device: &DeviceInfo,
    directory_state: &HashMap<PathBuf, DirectoryLoadState>,
    expanded_sources: &BTreeSet<PathBuf>,
    depth: usize,
    entries: &mut Vec<SourceEntry>,
) {
    let path = &device.mount_path;
    let is_expanded = expanded_sources.contains(path);
    let (has_children, is_loading) = match directory_state.get(path) {
        Some(DirectoryLoadState::Loading) => (true, true),
        Some(DirectoryLoadState::Loaded(children)) => (!children.is_empty(), false),
        None => (true, false),
    };

    entries.push(SourceEntry {
        device_id: device.id.clone(),
        path: path.clone(),
        label: format!(
            "{} ({})",
            device.display_name,
            match &device.availability {
                DeviceAvailability::Available => "ready",
                DeviceAvailability::Unavailable(_) => "unavailable",
            }
        ),
        depth,
        is_expanded,
        has_children,
        is_loading,
        is_device_root: true,
    });

    if !is_expanded {
        return;
    }

    if let Some(DirectoryLoadState::Loaded(children)) = directory_state.get(path) {
        for child in children {
            flatten_directory_entry(
                &device.id,
                child,
                directory_state,
                expanded_sources,
                depth + 1,
                entries,
            );
        }
    }
}

fn flatten_directory_entry(
    device_id: &str,
    path: &Path,
    directory_state: &HashMap<PathBuf, DirectoryLoadState>,
    expanded_sources: &BTreeSet<PathBuf>,
    depth: usize,
    entries: &mut Vec<SourceEntry>,
) {
    let is_expanded = expanded_sources.contains(path);
    let (has_children, is_loading) = match directory_state.get(path) {
        Some(DirectoryLoadState::Loading) => (true, true),
        Some(DirectoryLoadState::Loaded(children)) => (!children.is_empty(), false),
        None => (true, false),
    };

    entries.push(SourceEntry {
        device_id: device_id.to_string(),
        path: path.to_path_buf(),
        label: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("folder")
            .to_string(),
        depth,
        is_expanded,
        has_children,
        is_loading,
        is_device_root: false,
    });

    if !is_expanded {
        return;
    }

    if let Some(DirectoryLoadState::Loaded(children)) = directory_state.get(path) {
        for child in children {
            flatten_directory_entry(
                device_id,
                child,
                directory_state,
                expanded_sources,
                depth + 1,
                entries,
            );
        }
    }
}

fn spawn_directory_load(sender: Sender<BackgroundUpdate>, path: PathBuf) {
    thread::spawn(move || {
        let children = read_directory_children(&path);
        let _ = sender.send(BackgroundUpdate::DirectoryLoaded(path, children));
    });
}

fn read_directory_children(root: &Path) -> Vec<PathBuf> {
    let mut children = fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(|entry| entry.ok()))
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| !name.starts_with('.'))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    children.sort();
    children
}

fn availability_message(device: &DeviceInfo) -> String {
    match &device.availability {
        DeviceAvailability::Available => format!("{} is ready to import.", device.display_name),
        DeviceAvailability::Unavailable(reason) => {
            format!("{} cannot be used: {}", device.display_name, reason)
        }
    }
}

fn trim_last_char(input: String) -> String {
    let mut chars = input.chars().collect::<Vec<_>>();
    chars.pop();
    chars.into_iter().collect()
}
