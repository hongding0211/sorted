use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
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
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
};
use sysinfo::Disks;

use crate::{
    core::{
        archive::{build_archive_plan, destination_preview},
        config::{ConfigStore, resolve_destination_root, validate_date_format, validate_settings},
        copy::{
            CopyProgress, CopySummary, archive_destination_exists, discover_media_files,
            execute_copy, plan_copy,
        },
        types::{ArchiveSettings, DeviceAvailability, DeviceInfo, ImportSession},
    },
    platform::discovery::{DeviceDiscovery, SystemDeviceDiscovery, validate_selected_device},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Settings,
    Confirmation,
    CopyResults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusField {
    SourceTree,
    Theme,
    DestinationRoot,
    DateFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusKind {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusKind {
    fn icon(self) -> &'static str {
        match self {
            StatusKind::Info => "◎",
            StatusKind::Success => "●",
            StatusKind::Warning => "▲",
            StatusKind::Error => "✕",
        }
    }
}

#[derive(Debug, Clone)]
struct StatusMessage {
    kind: StatusKind,
    text: String,
}

impl StatusMessage {
    fn info(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Info,
            text: text.into(),
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Success,
            text: text.into(),
        }
    }

    fn warning(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Warning,
            text: text.into(),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Error,
            text: text.into(),
        }
    }
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
    is_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceSummary {
    file_count: usize,
    total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AsyncValue<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
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

        app.advance_animation();
    }

    Ok(())
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    let status_height = if app.is_copy_active() { 6 } else { 4 };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(status_height),
            Constraint::Length(4),
        ])
        .split(frame.area());

    let title = Paragraph::new(vec![Line::from(vec![Span::styled(
        "Archive anything, stay sorted",
        helper_style(),
    )])])
    .block(panel_block("Sorted", false))
    .wrap(Wrap { trim: true });
    frame.render_widget(title, layout[0]);

    match app.screen {
        Screen::Main => draw_main(frame, app, layout[1]),
        Screen::Settings => draw_settings(frame, app, layout[1]),
        Screen::Confirmation => draw_confirmation(frame, app, layout[1]),
        Screen::CopyResults => draw_results(frame, app, layout[1]),
    }

    draw_status(frame, app, layout[2]);

    let keyboard = Paragraph::new(app.keyboard_help_lines())
        .block(panel_block("Keyboard", false))
        .wrap(Wrap { trim: true });
    frame.render_widget(keyboard, layout[3]);
}

fn draw_main(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    draw_source_tree(frame, app, columns[0]);
    draw_session(frame, app, columns[1]);
}

fn draw_source_tree(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let items = if app.source_entries.is_empty() {
        if app.devices_loading {
            vec![
                ListItem::new(Line::from(vec![Span::styled(
                    format!("{} Scanning removable devices...", app.loading_glyph()),
                    helper_style(),
                )])),
                ListItem::new(Line::from(vec![Span::styled(
                    "Browse will update automatically when discovery finishes.",
                    helper_style(),
                )])),
            ]
        } else {
            vec![
                ListItem::new(Line::from(vec![Span::styled(
                    "No removable devices found.",
                    semantic_style(StatusKind::Warning),
                )])),
                ListItem::new(Line::from(vec![Span::styled(
                    "Connect media, then refresh to scan again.",
                    helper_style(),
                )])),
            ]
        }
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
                let loading = if entry.is_loading {
                    format!("  {}", app.loading_glyph())
                } else {
                    String::new()
                };
                let indent = "  ".repeat(entry.depth);
                let style = if index == app.source_index && app.focus == FocusField::SourceTree {
                    focus_style()
                } else if entry.is_device_root && !entry.is_available {
                    semantic_style(StatusKind::Warning).add_modifier(Modifier::BOLD)
                } else if entry.is_device_root {
                    label_style()
                } else {
                    Style::default()
                };
                ListItem::new(format!("{indent}{prefix} {}{loading}", entry.label)).style(style)
            })
            .collect()
    };

    let widget = List::new(items).block(panel_block(
        "Source Browser",
        app.focus == FocusField::SourceTree,
    ));
    frame.render_widget(widget, area);
}

fn draw_session(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let preview = app.preview_path();
    let theme = if app.import_session.theme.trim().is_empty() {
        None
    } else {
        Some(app.import_session.theme.clone())
    };

    let content = Paragraph::new(vec![
        Line::from(vec![Span::styled("THEME", section_style())]),
        Line::from(vec![
            Span::styled("  ", helper_style()),
            match theme {
                Some(ref value) => Span::styled(value.clone(), field_style(app, FocusField::Theme)),
                None => Span::raw(""),
            },
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("TARGET", section_style())]),
        Line::from(vec![
            Span::styled("  ", helper_style()),
            match preview {
                Some(value) => Span::styled(value, highlight_style()),
                None => Span::raw("N/A"),
            },
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("CAPACITY", section_style())]),
        Line::from(vec![
            Span::styled("  Selected", label_style()),
            Span::styled("  ", helper_style()),
            source_summary_span(app),
        ]),
        Line::from(vec![
            Span::styled("  Free Space", label_style()),
            Span::styled("  ", helper_style()),
            destination_space_span(app),
        ]),
    ])
    .block(panel_block(
        "Archive Target",
        app.focus == FocusField::Theme,
    ))
    .wrap(Wrap { trim: true });
    frame.render_widget(content, area);
}

fn draw_settings(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);
    let feedback = app.settings_feedback();
    let preview = validate_date_format(&app.settings.date_format)
        .map(|date| date.preview)
        .unwrap_or_else(|_| "Preview unavailable until the format is valid.".to_string());

    let fields = Paragraph::new(vec![
        Line::from(vec![Span::styled("Settings", title_style())]),
        Line::from(Span::styled(
            "Edit values in-place. The active field is highlighted so you always know where typing goes.",
            helper_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Destination Root: ",
                field_style(app, FocusField::DestinationRoot),
            ),
            Span::raw(app.settings.destination_root.display().to_string()),
        ]),
        Line::from(Span::styled(
            "Used as the writable base directory for each archive session.",
            helper_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Date Format: ", field_style(app, FocusField::DateFormat)),
            Span::raw(app.settings.date_format.clone()),
        ]),
        Line::from(Span::styled(
            "Controls the date segment rendered inside destination folder names.",
            helper_style(),
        )),
    ])
    .block(panel_block("Archive Preferences", true))
    .wrap(Wrap { trim: true });
    frame.render_widget(fields, panels[0]);

    let feedback_panel = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Preview: ", label_style()),
            Span::raw(preview),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{} ", feedback.kind.icon()),
                semantic_style(feedback.kind).add_modifier(Modifier::BOLD),
            ),
            Span::raw(feedback.text),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter saves these values. Esc returns without saving the current edits.",
            helper_style(),
        )),
    ])
    .block(panel_block("Validation", false).border_style(semantic_style(feedback.kind)))
    .wrap(Wrap { trim: true });
    frame.render_widget(feedback_panel, panels[1]);
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
        Line::from(vec![Span::styled("Ready to archive", title_style())]),
        Line::from(Span::styled(
            "Check the source and resolved destination one last time before the copy starts.",
            helper_style(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Source Folder: ", label_style()),
            Span::raw(source),
        ]),
        Line::from(vec![
            Span::styled("Destination Preview: ", label_style()),
            Span::raw(preview),
        ]),
        Line::from(vec![
            Span::styled("Theme: ", label_style()),
            Span::raw(if app.import_session.theme.trim().is_empty() {
                "No theme entered"
            } else {
                &app.import_session.theme
            }),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter starts the copy. Esc returns to the archive screen.",
            helper_style(),
        )),
    ])
    .block(panel_block("Confirmation", true))
    .wrap(Wrap { trim: true });
    frame.render_widget(modal, area);
}

fn draw_results(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let result_text = match &app.copy_result {
        Some(result) => {
            let mut lines = vec![
                Line::from(vec![Span::styled(
                    if result.failures.is_empty() {
                        "Archive complete"
                    } else {
                        "Archive completed with issues"
                    },
                    title_style(),
                )]),
                Line::from(vec![
                    Span::styled("Destination: ", label_style()),
                    Span::raw(result.destination.display().to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Files copied: ", label_style()),
                    Span::raw(result.copied_files.to_string()),
                ]),
                Line::from(""),
            ];
            if result.failures.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No copy failures were reported.",
                    semantic_style(StatusKind::Success),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Failures",
                    semantic_style(StatusKind::Warning).add_modifier(Modifier::BOLD),
                )));
                for failure in &result.failures {
                    lines.push(Line::from(format!(
                        "- {}: {}",
                        failure.file.display(),
                        failure.error
                    )));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press Enter or Esc to return to the main archive screen.",
                helper_style(),
            )));
            lines
        }
        None => vec![
            Line::from(vec![Span::styled(
                "No copy has been run yet.",
                helper_style(),
            )]),
            Line::from(Span::styled(
                "Start an import from the main screen to populate results here.",
                helper_style(),
            )),
        ],
    };

    let widget = Paragraph::new(result_text)
        .block(panel_block("Copy Results", true))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_status(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = panel_block("Status", false).border_style(semantic_style(app.status_message.kind));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.is_copy_active() {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let message = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", app.status_message.kind.icon()),
                semantic_style(app.status_message.kind).add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.status_message.text.clone()),
        ]))
        .wrap(Wrap { trim: true });
        frame.render_widget(message, rows[0]);

        let gauge = Gauge::default()
            .gauge_style(semantic_style(StatusKind::Info).add_modifier(Modifier::BOLD))
            .ratio(copy_progress_ratio(app.copy_progress.as_ref()))
            .label(copy_progress_summary(app.copy_progress.as_ref()));
        frame.render_widget(gauge, rows[1]);

        let current = Paragraph::new(Line::from(vec![
            Span::styled("Current: ", helper_style()),
            Span::raw(copy_progress_current_file(app.copy_progress.as_ref())),
        ]))
        .wrap(Wrap { trim: true });
        frame.render_widget(current, rows[2]);
    } else {
        let status = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("{} ", app.status_message.kind.icon()),
                    semantic_style(app.status_message.kind).add_modifier(Modifier::BOLD),
                ),
                Span::raw(app.status_message.text.clone()),
            ]),
            Line::from(Span::styled(
                "Transient app feedback appears here while richer guidance stays inside each screen.",
                helper_style(),
            )),
        ])
        .wrap(Wrap { trim: true });
        frame.render_widget(status, inner);
    }
}

fn field_style(app: &App, field: FocusField) -> Style {
    if app.focus == field {
        focus_style()
    } else {
        label_style()
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
    status_message: StatusMessage,
    screen: Screen,
    focus: FocusField,
    copy_progress: Option<CopyProgress>,
    copy_result: Option<CopySummary>,
    copy_updates: Option<Receiver<CopyUpdate>>,
    copy_cancel: Option<Arc<AtomicBool>>,
    background_updates: Receiver<BackgroundUpdate>,
    background_sender: Sender<BackgroundUpdate>,
    animation_started_at: Instant,
    source_summary: AsyncValue<SourceSummary>,
    source_summary_path: Option<PathBuf>,
    destination_free_space: AsyncValue<u64>,
    destination_free_path: Option<PathBuf>,
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
            status_message: StatusMessage::info(
                "Starting up. Scanning devices in the background...",
            ),
            screen: Screen::Main,
            focus: FocusField::SourceTree,
            copy_progress: None,
            copy_result: None,
            copy_updates: None,
            copy_cancel: None,
            background_updates,
            background_sender,
            animation_started_at: Instant::now(),
            source_summary: AsyncValue::Idle,
            source_summary_path: None,
            destination_free_space: AsyncValue::Idle,
            destination_free_path: None,
        };
        app.request_device_refresh();
        app.request_destination_free_space();
        Ok(app)
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    if self.is_copy_active() {
                        if let Some(cancel) = &self.copy_cancel {
                            cancel.store(true, Ordering::SeqCst);
                        }
                        self.status_message =
                            StatusMessage::warning("Stopping copy after the current file...");
                        return Ok(false);
                    }
                    return Ok(true);
                }
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
        if self.is_copy_active() {
            self.status_message = StatusMessage::warning(
                "Copy is running. Wait for it to finish before opening settings.",
            );
        } else {
            self.screen = Screen::Settings;
            self.focus = FocusField::DestinationRoot;
            self.status_message = StatusMessage::info("Editing persisted settings.");
        }
    }

    fn request_device_refresh(&mut self) {
        if self.is_copy_active() {
            self.status_message = StatusMessage::warning(
                "Copy is running. Wait for it to finish before refreshing devices.",
            );
            return;
        }
        self.devices_loading = true;
        self.status_message = StatusMessage::info("Refreshing devices in the background...");
        let sender = self.background_sender.clone();
        thread::spawn(move || {
            let discovery = SystemDeviceDiscovery;
            let result = discovery.discover();
            let _ = sender.send(BackgroundUpdate::DevicesLoaded(result));
        });
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.screen {
            Screen::Main | Screen::Confirmation | Screen::CopyResults => match self.focus {
                FocusField::SourceTree => FocusField::Theme,
                _ => FocusField::SourceTree,
            },
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
        if self.is_copy_active() && self.screen != Screen::CopyResults {
            self.status_message =
                StatusMessage::warning("Copy already in progress. Wait for it to stop first.");
            return Ok(());
        }

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
                    self.status_message =
                        StatusMessage::warning("Pick a source folder before continuing.");
                    return Ok(());
                };
                if matches!(
                    self.directory_state.get(&source_root),
                    Some(DirectoryLoadState::Loading)
                ) {
                    self.status_message = StatusMessage::warning(
                        "That folder is still loading. Try again in a moment.",
                    );
                    return Ok(());
                }
                if self.import_session.theme.trim().is_empty() {
                    self.status_message =
                        StatusMessage::warning("Enter a theme before starting the import.");
                    self.focus = FocusField::Theme;
                    return Ok(());
                }
                let archive_plan = build_archive_plan(
                    &self.settings,
                    &self.import_session.theme,
                    &selected_device,
                    Local::now(),
                )?;
                if archive_destination_exists(&archive_plan.archive_root) {
                    self.status_message = StatusMessage::error(format!(
                        "Archive destination {} already exists.",
                        archive_plan.archive_root.display()
                    ));
                    self.focus = FocusField::SourceTree;
                    return Ok(());
                }
                self.import_session.selected_device = Some(selected_device);
                self.screen = Screen::Confirmation;
                self.status_message = StatusMessage::info(
                    "Review the source folder and archive destination before copy starts.",
                );
            }
            Screen::Settings => {
                match validate_settings(&self.settings).and_then(|(settings, _)| {
                    self.config_store.save(&settings)?;
                    Ok(settings)
                }) {
                    Ok(settings) => {
                        self.settings = settings;
                        self.screen = Screen::Main;
                        self.focus = FocusField::SourceTree;
                        self.status_message = StatusMessage::success(format!(
                            "Saved settings to {}",
                            self.config_store.config_path().display()
                        ));
                    }
                    Err(error) => {
                        self.status_message =
                            StatusMessage::error(format!("Settings could not be saved: {error}"));
                    }
                }
            }
            Screen::Confirmation => self.start_copy()?,
            Screen::CopyResults => {
                self.screen = Screen::Main;
                self.status_message = StatusMessage::info("Returned to import screen.");
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
                self.status_message = StatusMessage::info("Returned to import screen.");
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
                self.request_destination_free_space();
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
                self.request_destination_free_space();
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
            self.status_message =
                StatusMessage::warning("Enter a theme before starting the import.");
            self.screen = Screen::Main;
            self.focus = FocusField::Theme;
            return Ok(());
        }
        let source_root = self
            .selected_source()
            .ok_or_else(|| anyhow!("pick a source folder before importing"))?;

        let plan = match plan_copy(
            &self.settings,
            &self.import_session.theme,
            &selected,
            &source_root,
            Local::now(),
        ) {
            Ok(plan) => plan,
            Err(error) => {
                self.screen = Screen::Main;
                self.focus = FocusField::SourceTree;
                self.status_message =
                    StatusMessage::error(format!("Import could not start: {error}"));
                return Ok(());
            }
        };
        self.status_message = StatusMessage::info(format!(
            "Copying {} media file(s) from {}",
            plan.files.len(),
            source_root.display()
        ));
        self.copy_progress = Some(CopyProgress {
            copied_files: 0,
            total_files: plan.files.len(),
            current_file: None,
        });
        self.copy_result = None;
        self.screen = Screen::Main;
        self.focus = FocusField::SourceTree;
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        self.copy_updates = Some(receiver);
        self.copy_cancel = Some(cancel.clone());

        thread::spawn(move || {
            let sender_for_progress = sender.clone();
            let result = execute_copy(
                &plan,
                move |progress| {
                    let _ = sender_for_progress.send(CopyUpdate::Progress(progress));
                },
                move || cancel.load(Ordering::SeqCst),
            );
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
                            self.status_message =
                                StatusMessage::error(format!("Device refresh failed: {error}"));
                        }
                    }
                }
                BackgroundUpdate::DirectoryLoaded(path, children) => {
                    self.pending_directory_loads.remove(&path);
                    self.directory_state
                        .insert(path.clone(), DirectoryLoadState::Loaded(children));
                    self.rebuild_source_entries();
                    self.status_message = StatusMessage::info(format!("Loaded {}", path.display()));
                }
                BackgroundUpdate::SourceSummaryLoaded(path, result) => {
                    if self.source_summary_path.as_ref() == Some(&path) {
                        self.source_summary = match result {
                            Ok(summary) => AsyncValue::Ready(summary),
                            Err(error) => AsyncValue::Error(error),
                        };
                    }
                }
                BackgroundUpdate::DestinationFreeSpaceLoaded(path, result) => {
                    if self.destination_free_path.as_ref() == Some(&path) {
                        self.destination_free_space = match result {
                            Ok(bytes) => AsyncValue::Ready(bytes),
                            Err(error) => AsyncValue::Error(error),
                        };
                    }
                }
            }
        }

        let mut finished = None;
        if let Some(receiver) = &self.copy_updates {
            while let Ok(update) = receiver.try_recv() {
                match update {
                    CopyUpdate::Progress(progress) => {
                        self.copy_progress = Some(progress.clone());
                        self.status_message = if self.is_cancel_requested() {
                            StatusMessage::warning("Stopping copy after the current file...")
                        } else {
                            StatusMessage::info(format!(
                                "Copying media files: {}/{} complete",
                                progress.copied_files, progress.total_files
                            ))
                        };
                    }
                    CopyUpdate::Finished(result) => finished = Some(result),
                }
            }
        }

        if let Some(result) = finished {
            self.copy_updates = None;
            self.copy_cancel = None;
            match result {
                Ok(summary) => {
                    self.copy_result = Some(summary.clone());
                    self.screen = Screen::Main;
                    self.copy_progress = None;
                    self.status_message = if summary.was_cancelled {
                        StatusMessage::warning(format!(
                            "Copy interrupted after {} file(s).",
                            summary.copied_files
                        ))
                    } else if summary.failures.is_empty() {
                        StatusMessage::success(format!(
                            "Copy finished: {} file(s) archived.",
                            summary.copied_files
                        ))
                    } else {
                        StatusMessage::warning(format!(
                            "Copy completed with {} failure(s).",
                            summary.failures.len()
                        ))
                    };
                }
                Err(error) => {
                    self.copy_result = None;
                    self.copy_progress = None;
                    self.screen = Screen::Main;
                    self.status_message = StatusMessage::error(format!("Copy failed: {error}"));
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
        self.source_summary = AsyncValue::Idle;
        self.source_summary_path = None;

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
        if self.source_entries.is_empty() {
            self.source_summary = AsyncValue::Idle;
            self.source_summary_path = None;
        }

        self.status_message = if self.devices.is_empty() {
            StatusMessage::warning("No removable devices found.")
        } else {
            StatusMessage::success(format!("Found {} removable device(s).", self.devices.len()))
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
            self.request_source_summary(entry.path.clone());
        }
    }

    fn selected_source(&self) -> Option<PathBuf> {
        self.import_session.selected_source.clone()
    }

    fn selected_device(&self) -> Option<DeviceInfo> {
        self.import_session.selected_device.clone()
    }

    fn is_copy_active(&self) -> bool {
        self.copy_updates.is_some()
    }

    fn is_cancel_requested(&self) -> bool {
        self.copy_cancel
            .as_ref()
            .is_some_and(|cancel| cancel.load(Ordering::SeqCst))
    }

    fn keyboard_help_lines(&self) -> Vec<Line<'static>> {
        let context = contextual_help(self.screen, self.focus);
        vec![
            Line::from(vec![
                Span::styled("Global: ", helper_style()),
                Span::raw(global_help_text(self.screen)),
            ]),
            Line::from(vec![
                Span::styled("Here: ", helper_style()),
                Span::raw(context),
            ]),
        ]
    }

    fn settings_feedback(&self) -> StatusMessage {
        match validate_settings(&self.settings) {
            Ok((resolved, preview)) => StatusMessage::success(format!(
                "Ready to save. Destination resolves to {} and renders dates like {}.",
                resolved.destination_root.display(),
                preview.preview
            )),
            Err(error) => StatusMessage::error(format!("Fix this before saving: {error}")),
        }
    }

    fn advance_animation(&mut self) {
        let _ = self.animation_started_at.elapsed();
    }

    fn loading_glyph(&self) -> &'static str {
        const FRAMES: &[&str] = &["◜", "◠", "◝", "◞", "◡", "◟"];
        let frame = (self.animation_started_at.elapsed().as_millis() / 120) as usize;
        FRAMES[frame % FRAMES.len()]
    }

    fn request_source_summary(&mut self, path: PathBuf) {
        if self.source_summary_path.as_ref() == Some(&path)
            && matches!(
                self.source_summary,
                AsyncValue::Loading | AsyncValue::Ready(_)
            )
        {
            return;
        }

        self.source_summary_path = Some(path.clone());
        self.source_summary = AsyncValue::Loading;
        let sender = self.background_sender.clone();
        thread::spawn(move || {
            let result = summarize_source_root(&path).map_err(|error| error.to_string());
            let _ = sender.send(BackgroundUpdate::SourceSummaryLoaded(path, result));
        });
    }

    fn request_destination_free_space(&mut self) {
        let Ok(path) = resolve_destination_root(&self.settings.destination_root) else {
            self.destination_free_path = None;
            self.destination_free_space = AsyncValue::Error(
                "Free space appears after the destination path is valid.".to_string(),
            );
            return;
        };

        if self.destination_free_path.as_ref() == Some(&path)
            && matches!(
                self.destination_free_space,
                AsyncValue::Loading | AsyncValue::Ready(_)
            )
        {
            return;
        }

        self.destination_free_path = Some(path.clone());
        self.destination_free_space = AsyncValue::Loading;
        let sender = self.background_sender.clone();
        thread::spawn(move || {
            let result = available_space_for_destination(&path).map_err(|error| error.to_string());
            let _ = sender.send(BackgroundUpdate::DestinationFreeSpaceLoaded(path, result));
        });
    }
}

enum BackgroundUpdate {
    DevicesLoaded(Result<Vec<DeviceInfo>>),
    DirectoryLoaded(PathBuf, Vec<PathBuf>),
    SourceSummaryLoaded(PathBuf, Result<SourceSummary, String>),
    DestinationFreeSpaceLoaded(PathBuf, Result<u64, String>),
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
        label: device.display_name.clone(),
        depth,
        is_expanded,
        has_children,
        is_loading,
        is_device_root: true,
        is_available: device.is_available(),
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
        is_available: true,
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

fn availability_message(device: &DeviceInfo) -> StatusMessage {
    match &device.availability {
        DeviceAvailability::Available => {
            StatusMessage::success(format!("{} is ready to import.", device.display_name))
        }
        DeviceAvailability::Unavailable(reason) => StatusMessage::warning(format!(
            "{} cannot be used: {}",
            device.display_name, reason
        )),
    }
}

fn trim_last_char(input: String) -> String {
    let mut chars = input.chars().collect::<Vec<_>>();
    chars.pop();
    chars.into_iter().collect()
}

fn panel_block<'a>(title: &'a str, active: bool) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if active {
            focus_style()
        } else {
            Style::default()
        })
}

fn focus_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn title_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

fn highlight_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn label_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

fn section_style() -> Style {
    Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD)
}

fn helper_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn semantic_style(kind: StatusKind) -> Style {
    match kind {
        StatusKind::Info => Style::default().fg(Color::Blue),
        StatusKind::Success => Style::default().fg(Color::Green),
        StatusKind::Warning => Style::default().fg(Color::Yellow),
        StatusKind::Error => Style::default().fg(Color::Red),
    }
}

fn capacity_ok_style() -> Style {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
}

fn capacity_warn_style() -> Style {
    Style::default()
        .fg(Color::LightRed)
        .add_modifier(Modifier::BOLD)
}

fn contextual_help(screen: Screen, focus: FocusField) -> &'static str {
    match screen {
        Screen::Main => match focus {
            FocusField::SourceTree => {
                "Arrows move | Left/Right collapse or expand folders | Tab switches to theme | Enter reviews the import"
            }
            FocusField::Theme => {
                "Type to edit the theme | Tab switches back to source browsing | Enter reviews the import"
            }
            _ => "Tab cycles focus between source browsing and theme entry.",
        },
        Screen::Settings => match focus {
            FocusField::DestinationRoot | FocusField::DateFormat => {
                "Type to edit the active field | Tab switches fields | Enter saves | Esc returns"
            }
            _ => "Edit archive preferences in-place, then press Enter to save.",
        },
        Screen::Confirmation => {
            "Enter starts the copy | Esc cancels and returns to the archive screen"
        }
        Screen::CopyResults => "Enter or Esc returns to the archive screen",
    }
}

fn global_help_text(_screen: Screen) -> &'static str {
    "Ctrl+Q quit | Ctrl+R refresh | Ctrl+S settings"
}

fn copy_progress_ratio(progress: Option<&CopyProgress>) -> f64 {
    let Some(progress) = progress else {
        return 0.0;
    };

    if progress.total_files == 0 {
        0.0
    } else {
        progress.copied_files as f64 / progress.total_files as f64
    }
}

fn copy_progress_summary(progress: Option<&CopyProgress>) -> String {
    match progress {
        Some(progress) if progress.total_files > 0 => format!(
            "{} of {} files copied",
            progress.copied_files, progress.total_files
        ),
        Some(_) => "Preparing copy job...".to_string(),
        None => "Preparing copy job...".to_string(),
    }
}

fn copy_progress_current_file(progress: Option<&CopyProgress>) -> String {
    match progress.and_then(|progress| progress.current_file.as_ref()) {
        Some(path) => path.display().to_string(),
        None => "Preparing copy job...".to_string(),
    }
}

fn source_summary_text(app: &App) -> String {
    match &app.source_summary {
        AsyncValue::Idle => "Pick a source folder".to_string(),
        AsyncValue::Loading => format!("{} ", app.loading_glyph()),
        AsyncValue::Ready(summary) => format!(
            "{} in {}",
            format_bytes(summary.total_bytes),
            pluralize(summary.file_count, "item", "items")
        ),
        AsyncValue::Error(_) => "N/A".to_string(),
    }
}

fn source_summary_span(app: &App) -> Span<'static> {
    let text = source_summary_text(app);
    match capacity_state(app) {
        Some(true) => Span::styled(text, capacity_ok_style()),
        Some(false) => Span::styled(text, capacity_warn_style()),
        None => Span::raw(text),
    }
}

fn destination_space_text(app: &App) -> String {
    match &app.destination_free_space {
        AsyncValue::Idle => "Waiting for destination".to_string(),
        AsyncValue::Loading => format!("{} ", app.loading_glyph()),
        AsyncValue::Ready(bytes) => format_bytes(*bytes),
        AsyncValue::Error(_) => "N/A".to_string(),
    }
}

fn destination_space_span(app: &App) -> Span<'static> {
    let text = destination_space_text(app);
    match capacity_state(app) {
        Some(true) => Span::styled(text, capacity_ok_style()),
        Some(false) => Span::styled(text, capacity_warn_style()),
        None => Span::raw(text),
    }
}

fn capacity_state(app: &App) -> Option<bool> {
    match (&app.source_summary, &app.destination_free_space) {
        (AsyncValue::Ready(source), AsyncValue::Ready(free)) => Some(*free >= source.total_bytes),
        _ => None,
    }
}

fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn summarize_source_root(root: &Path) -> Result<SourceSummary> {
    let files = discover_media_files(root)?;
    let total_bytes = files.iter().map(|file| file.size_bytes).sum();
    Ok(SourceSummary {
        file_count: files.len(),
        total_bytes,
    })
}

fn available_space_for_destination(path: &Path) -> Result<u64> {
    let probe = nearest_existing_ancestor(path)
        .ok_or_else(|| anyhow!("no existing parent directory could be resolved"))?;
    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .iter()
        .filter(|disk| probe.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .ok_or_else(|| anyhow!("no mounted filesystem could be resolved"))?;
    Ok(disk.available_space())
}

fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    while !current.exists() {
        if !current.pop() {
            return None;
        }
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_app() -> App {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let (background_sender, background_updates) = mpsc::channel();
        App {
            config_store: ConfigStore::from_path(config_path),
            settings: ArchiveSettings {
                destination_root: dir.path().to_path_buf(),
                date_format: "%Y-%m-%d".to_string(),
            },
            devices: Vec::new(),
            devices_loading: false,
            directory_state: HashMap::new(),
            expanded_sources: BTreeSet::new(),
            pending_directory_loads: HashSet::new(),
            source_entries: Vec::new(),
            source_index: 0,
            import_session: ImportSession::default(),
            status_message: StatusMessage::info("Ready"),
            screen: Screen::Main,
            focus: FocusField::SourceTree,
            copy_progress: None,
            copy_result: None,
            copy_updates: None,
            copy_cancel: None,
            background_updates,
            background_sender,
            animation_started_at: Instant::now(),
            source_summary: AsyncValue::Idle,
            source_summary_path: None,
            destination_free_space: AsyncValue::Idle,
            destination_free_path: None,
        }
    }

    #[test]
    fn contextual_help_is_screen_aware() {
        assert!(contextual_help(Screen::Settings, FocusField::DateFormat).contains("Enter saves"));
        assert!(
            contextual_help(Screen::CopyResults, FocusField::SourceTree).contains("Enter or Esc")
        );
        assert!(
            contextual_help(Screen::Confirmation, FocusField::SourceTree).contains("Enter starts")
        );
    }

    #[test]
    fn copy_progress_helpers_report_ratio_and_text() {
        let progress = CopyProgress {
            copied_files: 3,
            total_files: 5,
            current_file: Some(PathBuf::from("/tmp/media/frame.cr3")),
        };

        assert_eq!(copy_progress_ratio(Some(&progress)), 0.6);
        assert_eq!(
            copy_progress_summary(Some(&progress)),
            "3 of 5 files copied"
        );
        assert_eq!(
            copy_progress_current_file(Some(&progress)),
            "/tmp/media/frame.cr3"
        );
    }

    #[test]
    fn ctrl_q_quits_when_copy_is_not_active() {
        let mut app = test_app();

        let should_quit = app
            .handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL))
            .unwrap();

        assert!(should_quit);
    }

    #[test]
    fn ctrl_q_requests_copy_stop_while_copy_is_active() {
        let mut app = test_app();
        let (_sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        app.copy_updates = Some(receiver);
        app.copy_cancel = Some(cancel.clone());

        let should_quit = app
            .handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL))
            .unwrap();

        assert!(!should_quit);
        assert_eq!(app.status_message.kind, StatusKind::Warning);
        assert!(cancel.load(Ordering::SeqCst));
        assert!(app.status_message.text.contains("Stopping copy"));
    }

    #[test]
    fn settings_feedback_reports_invalid_date_format() {
        let mut app = test_app();
        app.settings.date_format = "%Q".to_string();

        let feedback = app.settings_feedback();

        assert_eq!(feedback.kind, StatusKind::Error);
        assert!(feedback.text.contains("unsupported date format specifier"));
    }

    #[test]
    fn invalid_settings_save_stays_in_settings_screen() {
        let mut app = test_app();
        app.screen = Screen::Settings;
        app.focus = FocusField::DateFormat;
        app.settings.date_format = "%Q".to_string();

        app.confirm_or_advance().unwrap();

        assert_eq!(app.screen, Screen::Settings);
        assert_eq!(app.status_message.kind, StatusKind::Error);
        assert!(
            app.status_message
                .text
                .contains("Settings could not be saved")
        );
    }

    #[test]
    fn missing_theme_blocks_confirmation_entry() {
        let mut app = test_app();
        let root = tempdir().unwrap();
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };
        app.devices = vec![device.clone()];
        app.import_session.selected_device = Some(device);
        app.import_session.selected_source = Some(root.path().to_path_buf());

        app.confirm_or_advance().unwrap();

        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.focus, FocusField::Theme);
        assert_eq!(app.status_message.kind, StatusKind::Warning);
        assert!(app.status_message.text.contains("Enter a theme"));
    }

    #[test]
    fn existing_archive_destination_blocks_confirmation_entry() {
        let mut app = test_app();
        let source_parent = tempdir().unwrap();
        let source_root = source_parent.path().join("DCIM");
        fs::create_dir_all(&source_root).unwrap();
        let destination_root = tempdir().unwrap();
        let archive_root = destination_root.path().join("shoot_2026-03-27").join("EOS_R6");
        fs::create_dir_all(&archive_root).unwrap();

        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: source_parent.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };
        app.devices = vec![device.clone()];
        app.settings.destination_root = destination_root.path().to_path_buf();
        app.import_session.selected_device = Some(device);
        app.import_session.selected_source = Some(source_root.clone());
        app.import_session.theme = "shoot".to_string();

        app.confirm_or_advance().unwrap();

        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.focus, FocusField::SourceTree);
        assert_eq!(app.status_message.kind, StatusKind::Error);
        assert!(app.status_message.text.contains("already exists"));
    }

    #[test]
    fn formats_source_summary_sizes() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("frame.jpg"), vec![0u8; 2048]).unwrap();

        let summary = summarize_source_root(root.path()).unwrap();

        assert_eq!(summary.file_count, 1);
        assert_eq!(summary.total_bytes, 2048);
        assert_eq!(format_bytes(summary.total_bytes), "2.0 KB");
    }

    #[test]
    fn reports_available_space_for_existing_destination() {
        let root = tempdir().unwrap();

        let free_space = available_space_for_destination(root.path()).unwrap();

        assert!(free_space > 0);
    }
}
