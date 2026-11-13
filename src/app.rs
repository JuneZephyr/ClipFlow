use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Text,
    Link,
    Code,
    Snippet,
}

pub fn detect_kind(text: &str) -> Kind {
    let t = text.trim();
    if t.starts_with("http://") || t.starts_with("https://") || t.starts_with("www.") {
        return Kind::Link;
    }
    let code_markers = [
        "fn ", "let ", "const ", "=>", "();", "};", "</", "{\n", "#include", "def ", "class ",
        "SELECT ", "#!/", "$ ", "npm ", "cargo ", "git ",
    ];
    if code_markers.iter().any(|m| t.contains(m)) {
        return Kind::Code;
    }
    Kind::Text
}

/// How many unpinned clips the history keeps. Pinned clips are always kept.
pub const MAX_HISTORY: usize = 300;

#[derive(Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: u64,
    pub text: String,
    pub source: String,
    pub created_ms: i64,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: u64,
    pub name: String,
    pub text: String,
    #[serde(default)]
    pub created_ms: i64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Pinned,
    Links,
    Code,
    Snippets,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ThemeChoice {
    Dark,
    Light,
    System,
}

impl Default for ThemeChoice {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeChoice {
    pub fn pref(self) -> egui::ThemePreference {
        match self {
            Self::Dark => egui::ThemePreference::Dark,
            Self::Light => egui::ThemePreference::Light,
            Self::System => egui::ThemePreference::System,
        }
    }
}

/// The words the interface is written in. English unless the person chooses Chinese.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    English,
    Chinese,
}

impl Language {
    pub fn is_chinese(self) -> bool {
        self == Self::Chinese
    }

    pub fn t(self, en: &'static str, zh: &'static str) -> &'static str {
        if self.is_chinese() {
            zh
        } else {
            en
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: ThemeChoice,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub ai_url: String,
    #[serde(default)]
    pub ai_model: String,
    #[serde(default)]
    pub ai_key: String,
    #[serde(default)]
    pub bg_image: String,
    #[serde(default = "default_bg_dim")]
    pub bg_dim: f32,
}

fn default_bg_dim() -> f32 {
    0.55
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Dark,
            language: Language::English,
            ai_url: String::new(),
            ai_model: String::new(),
            ai_key: String::new(),
            bg_image: String::new(),
            bg_dim: default_bg_dim(),
        }
    }
}

pub enum AiStatus {
    Idle,
    Running,
    Done(String),
    Failed(String),
}

impl Default for AiStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// One analysis at a time; the result lives only in memory until the app closes.
pub struct AiPanel {
    pub clip_id: Option<u64>,
    pub instruction: String,
    pub status: AiStatus,
    /// Changes whenever the dialog opens or a new request starts, so the scroll
    /// areas begin at the top instead of remembering an old position.
    pub epoch: u64,
    rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
}

impl Default for AiPanel {
    fn default() -> Self {
        Self {
            clip_id: None,
            instruction: String::new(),
            status: AiStatus::Idle,
            epoch: 0,
            rx: None,
        }
    }
}

#[derive(Default)]
pub struct TestState {
    pub running: bool,
    pub message: Option<(bool, String)>,
    rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
}

pub struct Shot {
    pub path: PathBuf,
    pub frames: u32,
    pub requested: bool,
    pub image: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
}

#[derive(Serialize, Deserialize, Default)]
struct Saved {
    #[serde(default)]
    version: u32,
    clips: Vec<Clip>,
    snippets: Vec<Snippet>,
    next_id: u64,
    #[serde(default)]
    settings: Settings,
}

#[derive(Clone, Copy)]
pub enum Action {
    CopyClip(u64),
    TogglePin(u64),
    DeleteClip(u64),
    CopySnippet(u64),
    DeleteSnippet(u64),
    OpenAi(u64),
}

pub struct ClipFlow {
    pub clips: Vec<Clip>,
    pub snippets: Vec<Snippet>,
    pub filter: Filter,
    pub search: String,
    pub selected: Option<u64>,
    pub toast: Option<(String, f64)>,
    pub paused: bool,
    pub shot: Option<Shot>,
    pub snippet_name: String,
    pub snippet_text: String,
    pub clear_armed_until: f64,
    pub now: f64,
    pub settings: Settings,
    pub show_key: bool,
    pub ai: AiPanel,
    pub test: TestState,
    pub background: Option<egui::TextureHandle>,
    pub background_size: egui::Vec2,
    pub bg_error: Option<String>,

    clipboard: Option<arboard::Clipboard>,
    last_seen: String,
    last_poll: f64,
    next_id: u64,
    state_path: Option<PathBuf>,
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn rel_time(created_ms: i64, lang: Language) -> String {
    let delta = (now_ms() - created_ms).max(0) / 1000;
    let zh = lang.is_chinese();
    if delta < 45 {
        if zh {
            "刚刚".to_string()
        } else {
            "just now".to_string()
        }
    } else if delta < 3600 {
        let n = (delta / 60).max(1);
        if zh {
            format!("{n} 分钟前")
        } else {
            format!("{n} min ago")
        }
    } else if delta < 86_400 {
        let n = delta / 3600;
        if zh {
            format!("{n} 小时前")
        } else {
            format!("{n} h ago")
        }
    } else if delta < 7 * 86_400 {
        let n = delta / 86_400;
        if zh {
            format!("{n} 天前")
        } else {
            format!("{n} d ago")
        }
    } else {
        chrono::DateTime::from_timestamp_millis(created_ms)
            .map(|d| {
                if zh {
                    format!("{}月{}日", d.format("%-m"), d.format("%-d"))
                } else {
                    d.format("%b %-d").to_string()
                }
            })
            .unwrap_or_else(|| "a while ago".to_string())
    }
}

fn state_path() -> Option<PathBuf> {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .map(|d| d.join("ClipFlow").join("state.json"))
}

fn seeded_clips() -> Vec<(String, &'static str, i64, bool)> {
    let m = 60_000;
    vec![
        (
            "https://www.are.na/block/redesign-notes".to_string(),
            "Safari",
            4 * m,
            false,
        ),
        (
            "cargo run --release -- --screenshot shots/preview.png".to_string(),
            "Terminal",
            11 * m,
            false,
        ),
        (
            "Standup notes: finish the search field today, then polish the empty state."
                .to_string(),
            "Notes",
            27 * m,
            false,
        ),
        (
            "hello@clipflow.app".to_string(),
            "Mail",
            74 * m,
            false,
        ),
        (
            "{\n  \"theme\": \"graphite\",\n  \"accent\": \"indigo\"\n}".to_string(),
            "VS Code",
            3 * 60 * m,
            true,
        ),
        (
            "Oat flat white, no sugar.".to_string(),
            "Messages",
            5 * 60 * m,
            false,
        ),
    ]
}

impl ClipFlow {
    pub fn new(cc: &eframe::CreationContext<'_>, shot: Option<PathBuf>) -> Self {
        ui::install_fonts(&cc.egui_ctx);
        // macOS puts ClipFlow in the bar at the top of the screen, so the window
        // does not have to stay open below everything else.
        #[cfg(target_os = "macos")]
        crate::menu_bar::install();

        // A preview run (--screenshot) shows example clips and never touches the
        // person's real history; a normal run starts from what they actually copied.
        let preview = shot.is_some();
        let path = if preview { None } else { state_path() };
        let mut next_id = 1;
        let (mut clips, mut snippets) = (Vec::new(), Vec::new());
        let mut settings = Settings::default();
        let mut loaded = false;

        if let Some(p) = &path {
            if let Ok(text) = std::fs::read_to_string(p) {
                if let Ok(saved) = serde_json::from_str::<Saved>(&text) {
                    if saved.version >= 1 {
                        clips = saved.clips;
                        snippets = saved.snippets;
                        next_id = saved.next_id.max(1);
                        settings = saved.settings;
                        loaded = true;
                    }
                }
            }
        }

        ui::apply_theme(&cc.egui_ctx, settings.theme);

        if preview {
            // Preview runs also accept a theme override, for looking at both palettes.
            if let Ok(theme) = std::env::var("CLIPFLOW_THEME") {
                settings.theme = match theme.as_str() {
                    "light" => ThemeChoice::Light,
                    "system" => ThemeChoice::System,
                    _ => ThemeChoice::Dark,
                };
                ui::apply_theme(&cc.egui_ctx, settings.theme);
            }
            if let Ok(lang) = std::env::var("CLIPFLOW_LANG") {
                settings.language = match lang.as_str() {
                    "zh" | "cn" | "chinese" => Language::Chinese,
                    _ => Language::English,
                };
            }
        }

        if loaded {
            // Older saves may predate the current limit.
            let mut seen = 0usize;
            clips.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
            clips.retain(|c| {
                if c.pinned {
                    true
                } else {
                    seen += 1;
                    seen <= MAX_HISTORY
                }
            });
        }

        if !loaded && preview {
            let base = now_ms();
            for (text, source, ago, pinned) in seeded_clips() {
                clips.push(Clip {
                    id: next_id,
                    text,
                    source: source.to_string(),
                    created_ms: base - ago,
                    pinned,
                });
                next_id += 1;
            }
            snippets.push(Snippet {
                id: next_id,
                name: "Signature".to_string(),
                text: "Best,\nAlex".to_string(),
                created_ms: base,
            });
            next_id += 1;
            snippets.push(Snippet {
                id: next_id,
                name: "Meeting link".to_string(),
                text: "https://meet.example.com/team-sync".to_string(),
                created_ms: base,
            });
            next_id += 1;

            if let Ok(text) = std::env::var("CLIPFLOW_CLIP_TEXT") {
                if let Some(first) = clips.first_mut() {
                    first.text = text;
                }
            }
        }

        let mut app = Self {
            clips,
            snippets,
            filter: Filter::All,
            search: String::new(),
            selected: None,
            toast: None,
            paused: false,
            shot: shot.map(|path| Shot {
                path,
                frames: 0,
                requested: false,
                image: Arc::new(Mutex::new(None)),
            }),
            snippet_name: String::new(),
            snippet_text: String::new(),
            clear_armed_until: 0.0,
            now: 0.0,
            settings,
            show_key: false,
            ai: AiPanel::default(),
            test: TestState::default(),
            background: None,
            background_size: egui::Vec2::ZERO,
            bg_error: None,
            clipboard: None,
            last_seen: String::new(),
            last_poll: 0.0,
            next_id,
            state_path: path,
        };

        if !preview {
            if app.clipboard.is_none() {
                app.clipboard = arboard::Clipboard::new().ok();
            }
            if let Some(cb) = &mut app.clipboard {
                if let Ok(t) = cb.get_text() {
                    app.last_seen = t;
                }
            }
        } else {
            // Lets a preview run open the settings page or the AI dialog for a look.
            match std::env::var("CLIPFLOW_PAGE").as_deref() {
                Ok("settings") => app.filter = Filter::Settings,
                Ok("snippets") => app.filter = Filter::Snippets,
                Ok("ai") => {
                    if let Some(first) = app.clips.first().map(|c| c.id) {
                        app.open_ai(first);
                        // Lets a preview run show the expanded source too.
                        if std::env::var("CLIPFLOW_AI_EXPAND").is_ok() {
                            cc.egui_ctx.memory_mut(|m| {
                                m.data.insert_temp(ui::source_expand_id(first), true);
                            });
                        }
                    }
                    if let Ok(text) = std::env::var("CLIPFLOW_AI_TEXT") {
                        app.ai.instruction = text;
                    }
                    if let Ok(answer) = std::env::var("CLIPFLOW_AI_RESULT") {
                        app.ai.status = AiStatus::Done(answer);
                    }
                }
                _ => {}
            }
            if let Ok(path) = std::env::var("CLIPFLOW_BG") {
                app.settings.bg_image = path;
            }
            if let Ok(dim) = std::env::var("CLIPFLOW_DIM") {
                if let Ok(value) = dim.parse::<f32>() {
                    app.settings.bg_dim = value;
                }
            }
        }
        app.load_background(&cc.egui_ctx);
        app.save();
        app
    }

    /// Keeps the newest unpinned clips up to [`MAX_HISTORY`]; pinned clips never expire.
    fn trim_history(&mut self) {
        self.clips.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
        let mut kept = 0usize;
        self.clips.retain(|c| {
            if c.pinned {
                true
            } else {
                kept += 1;
                kept <= MAX_HISTORY
            }
        });
    }

    pub fn save(&self) {
        let Some(path) = &self.state_path else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let saved = Saved {
            version: 1,
            clips: self.clips.clone(),
            snippets: self.snippets.clone(),
            next_id: self.next_id,
            settings: self.settings.clone(),
        };
        if let Ok(text) = serde_json::to_string_pretty(&saved) {
            let _ = std::fs::write(path, text);
        }
    }

    pub fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), self.now));
    }

    pub fn copy_text(&mut self, text: &str) {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }
        if let Some(cb) = &mut self.clipboard {
            if cb.set_text(text.to_string()).is_ok() {
                self.last_seen = text.to_string();
            }
        }
        let lang = self.settings.language;
        self.toast(lang.t("Copied to clipboard", "已复制到剪贴板"));
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::CopyClip(id) => {
                self.selected = Some(id);
                if let Some(c) = self.clips.iter().find(|c| c.id == id) {
                    let text = c.text.clone();
                    self.copy_text(&text);
                }
            }
            Action::TogglePin(id) => {
                if let Some(c) = self.clips.iter_mut().find(|c| c.id == id) {
                    c.pinned = !c.pinned;
                    let lang = self.settings.language;
                    let msg = if c.pinned {
                        lang.t("Pinned", "已固定")
                    } else {
                        lang.t("Unpinned", "已取消固定")
                    };
                    self.toast(msg);
                    self.trim_history();
                    self.save();
                }
            }
            Action::DeleteClip(id) => {
                self.clips.retain(|c| c.id != id);
                if self.selected == Some(id) {
                    self.selected = None;
                }
                if self.ai.clip_id == Some(id) {
                    self.close_ai();
                }
                let lang = self.settings.language;
                self.toast(lang.t("Clip removed", "已删除剪贴"));
                self.save();
            }
            Action::CopySnippet(id) => {
                if let Some(s) = self.snippets.iter().find(|s| s.id == id) {
                    let text = s.text.clone();
                    self.copy_text(&text);
                }
            }
            Action::DeleteSnippet(id) => {
                self.snippets.retain(|s| s.id != id);
                let lang = self.settings.language;
                self.toast(lang.t("Snippet removed", "已删除片段"));
                self.save();
            }
            Action::OpenAi(id) => self.open_ai(id),
        }
    }

    pub fn clear_history(&mut self) {
        self.clips.clear();
        self.selected = None;
        let lang = self.settings.language;
        self.toast(lang.t("History cleared", "历史已清空"));
        self.save();
    }

    pub fn pause(&mut self, paused: bool) {
        self.paused = paused;
        let lang = self.settings.language;
        self.toast(if paused {
            lang.t("History paused", "已暂停记录")
        } else {
            lang.t("Watching the clipboard", "正在监听剪贴板")
        });
    }

    pub fn ai_configured(&self) -> bool {
        !self.settings.ai_url.trim().is_empty() && !self.settings.ai_key.trim().is_empty()
    }

    pub fn open_ai(&mut self, id: u64) {
        if self.clips.iter().all(|c| c.id != id) {
            return;
        }
        self.ai.clip_id = Some(id);
        self.ai.instruction.clear();
        self.ai.status = AiStatus::Idle;
        self.ai.rx = None;
        self.ai.epoch = self.ai.epoch.wrapping_add(1);
    }

    pub fn close_ai(&mut self) {
        self.ai.clip_id = None;
        self.ai.instruction.clear();
        self.ai.status = AiStatus::Idle;
        self.ai.rx = None;
    }

    /// Sends the clip plus the person's instruction to their endpoint on a worker thread.
    pub fn run_ai(&mut self, ctx: &egui::Context) {
        if matches!(self.ai.status, AiStatus::Running) {
            return;
        }
        let Some(id) = self.ai.clip_id else { return };
        let Some(text) = self
            .clips
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.text.clone())
        else {
            self.close_ai();
            return;
        };
        if !self.ai_configured() {
            self.ai.status = AiStatus::Failed(
                "Add your AI request URL and API key in Settings first.".to_string(),
            );
            return;
        }

        let instruction = if self.ai.instruction.trim().is_empty() {
            "Explain what this text is and what it is useful for.".to_string()
        } else {
            self.ai.instruction.trim().to_string()
        };
        let prompt = format!("{instruction}\n\n---\n{text}");
        let url = self.settings.ai_url.clone();
        let key = self.settings.ai_key.clone();
        let model = self.settings.ai_model.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = crate::ai::chat(&url, &key, &model, crate::ai::ANALYSE_SYSTEM, &prompt);
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.ai.rx = Some(rx);
        self.ai.status = AiStatus::Running;
        // A fresh answer should be read from the first line.
        self.ai.epoch = self.ai.epoch.wrapping_add(1);
    }

    pub fn test_connection(&mut self, ctx: &egui::Context) {
        if self.test.running {
            return;
        }
        let url = self.settings.ai_url.clone();
        let key = self.settings.ai_key.clone();
        let model = self.settings.ai_model.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = crate::ai::test(&url, &key, &model);
            let _ = tx.send(result);
            ctx.request_repaint();
        });

        self.test.rx = Some(rx);
        self.test.running = true;
        self.test.message = None;
    }

    pub fn set_theme(&mut self, ctx: &egui::Context, choice: ThemeChoice) {
        if self.settings.theme == choice {
            return;
        }
        self.settings.theme = choice;
        ui::apply_theme(ctx, choice);
        self.save();
    }

    pub fn set_language(&mut self, language: Language) {
        if self.settings.language == language {
            return;
        }
        self.settings.language = language;
        self.save();
    }

    pub fn save_settings(&mut self) {
        self.save();
    }

    /// Loads the picture chosen for the background, if there is one.
    pub fn load_background(&mut self, ctx: &egui::Context) {
        self.background = None;
        self.background_size = egui::Vec2::ZERO;
        self.bg_error = None;
        let path = self.settings.bg_image.trim().to_string();
        if path.is_empty() {
            return;
        }
        // Keep the picture inside what the renderer can upload (often 2048px a side).
        let limit = ctx
            .input(|input| input.max_texture_side)
            .clamp(256, 4096) as u32;
        match load_background_image(&path, limit) {
            Ok((image, size)) => {
                self.background = Some(ctx.load_texture(
                    "clipflow-background",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
                self.background_size = size;
            }
            Err(message) => self.bg_error = Some(message),
        }
    }

    pub fn clear_background(&mut self) {
        self.settings.bg_image.clear();
        self.background = None;
        self.background_size = egui::Vec2::ZERO;
        self.bg_error = None;
        self.save();
    }

    pub fn choose_background(&mut self, ctx: &egui::Context) {
        let picked = rfd::FileDialog::new()
            .set_title("Choose a background picture")
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"],
            )
            .pick_file();
        if let Some(path) = picked {
            self.settings.bg_image = path.to_string_lossy().to_string();
            self.load_background(ctx);
            self.save();
        }
    }

    fn poll_ai(&mut self) {
        use std::sync::mpsc::TryRecvError;
        let Some(rx) = &self.ai.rx else { return };
        match rx.try_recv() {
            Ok(result) => {
                self.ai.status = match result {
                    Ok(text) => AiStatus::Done(text),
                    Err(err) => AiStatus::Failed(err),
                };
                self.ai.rx = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.ai.status = AiStatus::Failed("The AI request stopped unexpectedly.".to_string());
                self.ai.rx = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    fn poll_test(&mut self) {
        use std::sync::mpsc::TryRecvError;
        let Some(rx) = &self.test.rx else { return };
        match rx.try_recv() {
            Ok(result) => {
                self.test.running = false;
                self.test.message = Some(match result {
                    Ok(text) => (
                        true,
                        format!("Connected — the model replied “{}”", crate::ai::shorten(&text, 60)),
                    ),
                    Err(err) => (false, err),
                });
                self.test.rx = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.test.running = false;
                self.test.message = Some((
                    false,
                    "The connection test stopped unexpectedly.".to_string(),
                ));
                self.test.rx = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let pinned = self.clips.iter().filter(|c| c.pinned).count();
        let links = self
            .clips
            .iter()
            .filter(|c| detect_kind(&c.text) == Kind::Link)
            .count();
        let code = self
            .clips
            .iter()
            .filter(|c| detect_kind(&c.text) == Kind::Code)
            .count();
        (self.clips.len(), pinned, links, code)
    }

    pub fn visible_clips(&self) -> Vec<Clip> {
        let query = self.search.trim().to_lowercase();
        let mut items: Vec<Clip> = self
            .clips
            .iter()
            .filter(|c| match self.filter {
                Filter::All => true,
                Filter::Pinned => c.pinned,
                Filter::Links => detect_kind(&c.text) == Kind::Link,
                Filter::Code => detect_kind(&c.text) == Kind::Code,
                Filter::Snippets | Filter::Settings => false,
            })
            .filter(|c| {
                query.is_empty()
                    || c.text.to_lowercase().contains(&query)
                    || c.source.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();
        items.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then(b.created_ms.cmp(&a.created_ms))
        });
        items
    }

    pub fn visible_snippets(&self) -> Vec<Snippet> {
        let query = self.search.trim().to_lowercase();
        self.snippets
            .iter()
            .filter(|s| {
                query.is_empty()
                    || s.name.to_lowercase().contains(&query)
                    || s.text.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    pub fn add_snippet(&mut self) {
        let name = self.snippet_name.trim();
        let text = self.snippet_text.trim();
        if text.is_empty() {
            let lang = self.settings.language;
            self.toast(lang.t("Write the text first", "请先填写文字"));
            return;
        }
        let name = if name.is_empty() {
            text.lines().next().unwrap_or("Snippet").chars().take(28).collect()
        } else {
            name.to_string()
        };
        self.snippets.insert(
            0,
            Snippet {
                id: self.next_id,
                name,
                text: text.to_string(),
                created_ms: now_ms(),
            },
        );
        self.next_id += 1;
        self.snippet_name.clear();
        self.snippet_text.clear();
        self.toast(self.settings.language.t("Snippet saved", "片段已保存"));
        self.save();
    }

    pub fn poll_clipboard(&mut self) {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }
        let Some(cb) = &mut self.clipboard else { return };
        let Ok(raw) = cb.get_text() else { return };
        let text = raw.trim_end().to_string();
        if text.trim().is_empty() || text == self.last_seen {
            return;
        }
        self.last_seen = text.clone();

        if let Some(existing) = self.clips.iter_mut().find(|c| c.text == text) {
            existing.created_ms = now_ms();
            let pinned = existing.pinned;
            let id = existing.id;
            let source = existing.source.clone();
            let mut clip = Clip {
                id,
                text: text.clone(),
                source,
                created_ms: now_ms(),
                pinned,
            };
            clip.source = "Copied".to_string();
            self.clips.retain(|c| c.id != id);
            self.clips.insert(0, clip);
        } else {
            self.clips.insert(
                0,
                Clip {
                    id: self.next_id,
                    text,
                    source: "Copied".to_string(),
                    created_ms: now_ms(),
                    pinned: false,
                },
            );
            self.next_id += 1;
        }

        self.trim_history();
        self.save();
    }

    fn tick(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        self.now = now;
        if self.shot.is_none() && !self.paused && now - self.last_poll > 0.4 {
            self.last_poll = now;
            self.poll_clipboard();
        }
        if let Some((_, at)) = self.toast {
            if now - at > 2.2 {
                self.toast = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(120));
            }
        }
        self.poll_ai();
        self.poll_test();
        if self.ai.rx.is_some() || self.test.rx.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
    }

    fn keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let cmd = i.modifiers.command || i.modifiers.ctrl;
            if cmd && i.key_pressed(egui::Key::F) {
                ctx.memory_mut(|m| m.request_focus(egui::Id::new("search_field")));
            }
        });

        let typing = ctx.memory(|m| m.focused().is_some());

        let (up, down, enter, esc) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
            )
        });

        if esc {
            if self.ai.clip_id.is_some() {
                self.close_ai();
            } else if !self.search.is_empty() {
                self.search.clear();
                self.selected = None;
            } else {
                self.selected = None;
            }
        }

        if self.filter == Filter::Snippets || self.filter == Filter::Settings {
            return;
        }

        let ids: Vec<u64> = self.visible_clips().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return;
        }

        if up || down {
            let current = self
                .selected
                .and_then(|id| ids.iter().position(|x| *x == id));
            let next = match (current, up, down) {
                (None, true, _) => ids.len() - 1,
                (None, _, _) => 0,
                (Some(i), true, _) => i.saturating_sub(1),
                (Some(i), _, _) => (i + 1).min(ids.len() - 1),
            };
            self.selected = Some(ids[next]);
        }

        if enter && !typing {
            if let Some(id) = self.selected {
                if let Some(text) = self
                    .clips
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.text.clone())
                {
                    self.copy_text(&text);
                }
            }
        }
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.shot.as_mut() else {
            return;
        };
        shot.frames += 1;
        if shot.frames >= 100 && !shot.requested {
            shot.requested = true;
            let slot = shot.image.clone();
            ctx.request_screenshot(move |image| {
                if let Ok(mut guard) = slot.lock() {
                    *guard = Some(image);
                }
            });
        }
        let captured = shot.image.lock().ok().and_then(|mut guard| guard.take());
        if let Some(image) = captured {
            if let Some(parent) = shot.path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = save_color_image(&shot.path, &image);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn load_background_image(
    path: &str,
    limit: u32,
) -> Result<(egui::ColorImage, egui::Vec2), String> {
    let file = image::open(path).map_err(|err| format!("Could not open this picture: {err}"))?;
    let mut rgba = file.to_rgba8();
    let (mut width, mut height) = rgba.dimensions();

    // Very large photos are shrunk so the app stays quick and the GPU can upload them.
    if width > limit || height > limit {
        let scale = limit as f32 / width.max(height) as f32;
        let new_width = ((width as f32 * scale).round() as u32).max(1);
        let new_height = ((height as f32 * scale).round() as u32).max(1);
        rgba = image::imageops::thumbnail(&rgba, new_width, new_height);
        width = new_width;
        height = new_height;
    }

    let image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        rgba.as_raw(),
    );
    Ok((image, egui::vec2(width as f32, height as f32)))
}

fn save_color_image(path: &Path, image: &egui::ColorImage) -> Result<(), String> {    let [width, height] = image.size;
    let mut rgba = Vec::with_capacity(width * height * 4);
    for pixel in &image.pixels {
        rgba.extend_from_slice(&[pixel.r(), pixel.g(), pixel.b(), pixel.a()]);
    }
    image::save_buffer(
        path,
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}

impl eframe::App for ClipFlow {    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        let c = if visuals.dark_mode {
            ui::pal().bg
        } else {
            ui::Palette::light().bg
        };
        [
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            1.0,
        ]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // macOS keeps the window handle for the menu bar's show/hide item.
        #[cfg(target_os = "macos")]
        crate::menu_bar::remember_window(frame);
        self.tick(&ctx);
        self.keyboard(&ctx);
        ui::draw(self, ui);
        self.handle_screenshot(&ctx);
        if self.shot.is_some() {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_without_settings_still_load() {
        let text = r#"{"version":1,"clips":[],"snippets":[],"next_id":3}"#;
        let saved: Saved = serde_json::from_str(text).unwrap();
        assert_eq!(saved.next_id, 3);
        assert_eq!(saved.settings.theme, ThemeChoice::Dark);
        assert!(saved.settings.ai_key.is_empty());
    }

    #[test]
    fn settings_survive_a_save() {
        let saved = Saved {
            version: 1,
            clips: Vec::new(),
            snippets: Vec::new(),
            next_id: 1,
            settings: Settings {
                theme: ThemeChoice::Light,
                language: Language::English,
                ai_url: "https://example.test/v1".into(),
                ai_model: "some-model".into(),
                ai_key: "secret".into(),
                bg_image: "/tmp/wall.png".into(),
                bg_dim: 0.7,
            },
        };
        let text = serde_json::to_string(&saved).unwrap();
        let back: Saved = serde_json::from_str(&text).unwrap();
        assert_eq!(back.settings.theme, ThemeChoice::Light);
        assert_eq!(back.settings.language, Language::English);
        assert_eq!(back.settings.ai_url, "https://example.test/v1");
        assert_eq!(back.settings.ai_key, "secret");
        assert_eq!(back.settings.bg_image, "/tmp/wall.png");
        assert_eq!(back.settings.bg_dim, 0.7);
    }
}
