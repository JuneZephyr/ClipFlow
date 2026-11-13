use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Rect,
    RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};

use crate::app::{
    detect_kind, rel_time, Action, AiStatus, Clip, ClipFlow, Filter, Kind, Language, Snippet,
    ThemeChoice, MAX_HISTORY,
};

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub card_selected: Color32,
    pub input: Color32,
    pub border: Color32,
    pub border_hover: Color32,
    pub accent_border: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub accent_text: Color32,
    pub success: Color32,
    pub warn: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub danger_border: Color32,
    pub dark_mode: bool,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(0x12, 0x14, 0x18),
            panel: Color32::from_rgb(0x17, 0x1A, 0x20),
            card: Color32::from_rgb(0x1D, 0x21, 0x29),
            card_hover: Color32::from_rgb(0x23, 0x28, 0x32),
            card_selected: Color32::from_rgb(0x21, 0x2A, 0x3F),
            input: Color32::from_rgb(0x1B, 0x1F, 0x26),
            border: Color32::from_rgb(0x27, 0x2C, 0x36),
            border_hover: Color32::from_rgb(0x36, 0x3D, 0x4A),
            accent_border: Color32::from_rgb(0x39, 0x47, 0x72),
            text: Color32::from_rgb(0xE8, 0xEB, 0xF1),
            text2: Color32::from_rgb(0xA2, 0xAB, 0xBB),
            muted: Color32::from_rgb(0x6B, 0x74, 0x84),
            accent: Color32::from_rgb(0x6E, 0x8B, 0xFF),
            accent_soft: Color32::from_rgb(0x24, 0x2C, 0x45),
            accent_text: Color32::from_rgb(0xC9, 0xD5, 0xFF),
            success: Color32::from_rgb(0x5B, 0xD3, 0x9B),
            warn: Color32::from_rgb(0xE8, 0xB3, 0x6A),
            danger: Color32::from_rgb(0xF2, 0x8B, 0x9B),
            danger_soft: Color32::from_rgb(0x3A, 0x22, 0x26),
            danger_border: Color32::from_rgb(0x7A, 0x3B, 0x44),
            dark_mode: true,
        }
    }

    pub fn light() -> Self {
        Self {
            bg: Color32::from_rgb(0xF5, 0xF6, 0xF9),
            panel: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            card: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            card_hover: Color32::from_rgb(0xF3, 0xF5, 0xFA),
            card_selected: Color32::from_rgb(0xEA, 0xF0, 0xFF),
            input: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            border: Color32::from_rgb(0xE3, 0xE7, 0xEF),
            border_hover: Color32::from_rgb(0xCE, 0xD5, 0xE2),
            accent_border: Color32::from_rgb(0xB9, 0xC8, 0xF7),
            text: Color32::from_rgb(0x1A, 0x1E, 0x26),
            text2: Color32::from_rgb(0x4E, 0x57, 0x68),
            muted: Color32::from_rgb(0x8A, 0x93, 0xA3),
            accent: Color32::from_rgb(0x4C, 0x6B, 0xFF),
            accent_soft: Color32::from_rgb(0xE7, 0xEC, 0xFF),
            accent_text: Color32::from_rgb(0x2B, 0x49, 0xC4),
            success: Color32::from_rgb(0x14, 0x90, 0x6A),
            warn: Color32::from_rgb(0xB8, 0x7A, 0x1F),
            danger: Color32::from_rgb(0xB3, 0x26, 0x3E),
            danger_soft: Color32::from_rgb(0xFB, 0xEA, 0xEE),
            danger_border: Color32::from_rgb(0xEB, 0xC0, 0xC8),
            dark_mode: false,
        }
    }
}

thread_local! {
    static PALETTE: std::cell::Cell<Palette> = std::cell::Cell::new(Palette::dark());
}

/// The colours of the theme currently on screen.
pub fn pal() -> Palette {
    PALETTE.with(|p| p.get())
}

fn set_palette(p: Palette) {
    PALETTE.with(|c| c.set(p));
}

fn kind_style(kind: Kind, lang: Language) -> (Color32, &'static str) {
    match kind {
        Kind::Text => (
            Color32::from_rgb(0x8F, 0xA6, 0xC8),
            lang.t("TEXT", "文本"),
        ),
        Kind::Link => (
            Color32::from_rgb(0x5B, 0xC8, 0xF0),
            lang.t("LINK", "链接"),
        ),
        Kind::Code => (
            Color32::from_rgb(0xB7, 0x94, 0xF6),
            lang.t("CODE", "代码"),
        ),
        Kind::Snippet => (
            Color32::from_rgb(0xF6, 0xC1, 0x77),
            lang.t("SNIPPET", "片段"),
        ),
    }
}

fn tint(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

fn margin(x: i8, y: i8) -> egui::Margin {
    egui::Margin {
        left: x,
        right: x,
        top: y,
        bottom: y,
    }
}

fn style_for(style: &mut egui::Style, p: Palette) {
    let mut visuals = if p.dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = p.bg;
    visuals.window_fill = p.panel;
    visuals.extreme_bg_color = p.input;
    visuals.faint_bg_color = p.card;
    visuals.override_text_color = Some(p.text);
    visuals.selection.bg_fill = p.accent;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text2);
    visuals.widgets.inactive.bg_fill = p.card;
    visuals.widgets.inactive.weak_bg_fill = p.card;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text2);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.bg_fill = p.card_hover;
    visuals.widgets.hovered.weak_bg_fill = p.card_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.border_hover);
    visuals.widgets.active.bg_fill = p.accent_soft;
    visuals.widgets.active.weak_bg_fill = p.accent_soft;
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, p.text);
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals = visuals;
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(10.0, 6.0);
    style.spacing.scroll.bar_width = 7.0;
    style.spacing.scroll.floating = true;
    // The card colour used by default would make the handle vanish against the panels.
    style.spacing.scroll.foreground_color = true;
    style.text_styles = [
        (egui::TextStyle::Heading, FontId::proportional(23.0)),
        (egui::TextStyle::Body, FontId::proportional(14.0)),
        (egui::TextStyle::Monospace, FontId::monospace(13.0)),
        (egui::TextStyle::Button, FontId::proportional(13.0)),
        (egui::TextStyle::Small, FontId::proportional(11.5)),
    ]
    .into();
}

/// Dark, light, or whatever the system uses right now.
pub fn apply_theme(ctx: &egui::Context, choice: crate::app::ThemeChoice) {
    ctx.set_theme(choice.pref());
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style_for(style, Palette::dark());
    });
    ctx.style_mut_of(egui::Theme::Light, |style| {
        style_for(style, Palette::light());
    });
    set_palette(match ctx.theme() {
        egui::Theme::Dark => Palette::dark(),
        egui::Theme::Light => Palette::light(),
    });
}

/// A font file to use for Chinese, which face inside it, and whether it should
/// also take over the Latin text so both scripts come from one design.
struct CjkFont {
    path: std::path::PathBuf,
    index: u32,
    primary: bool,
}

/// macOS keeps PingFang in a downloadable-font asset whose folder names carry
/// version hashes, so it has to be looked for rather than hard-coded.
fn pingfang_path() -> Option<std::path::PathBuf> {
    let root = std::path::Path::new("/System/Library/AssetsV2");
    let Ok(assets) = std::fs::read_dir(root) else {
        return None;
    };
    let mut found: Vec<std::path::PathBuf> = Vec::new();
    for asset in assets.flatten() {
        let is_font_asset = asset
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("com_apple_MobileAsset_Font"));
        if !is_font_asset {
            continue;
        }
        // Each asset folder holds one or more "<uuid>.asset" versions.
        let Ok(versions) = std::fs::read_dir(asset.path()) else {
            continue;
        };
        for version in versions.flatten() {
            let is_version = version
                .file_name()
                .to_str()
                .is_some_and(|name| name.ends_with(".asset"));
            if !is_version {
                continue;
            }
            let candidate = version.path().join("AssetData").join("PingFang.ttc");
            if candidate.is_file() {
                found.push(candidate);
            }
        }
    }
    // An older asset can be left behind; the newest one is the one in use.
    found.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::UNIX_EPOCH)
    });
    found.pop()
}

/// Opens a font file, memory-mapping the big ones so a 78 MB collection does not
/// sit in the heap for the life of the app.
fn open_font_data(path: &std::path::Path, index: u32) -> Option<egui::FontData> {
    let blob: std::sync::Arc<dyn AsRef<[u8]> + Send + Sync> = {
        let file = std::fs::File::open(path).ok()?;
        if file.metadata().ok()?.len() > 8 * 1024 * 1024 {
            // Safety: the mapping is read-only and is kept alive inside the font data.
            let map = unsafe { memmap2::Mmap::map(&file) }.ok()?;
            std::sync::Arc::new(map)
        } else {
            std::sync::Arc::new(std::fs::read(path).ok()?)
        }
    };
    Some(egui::FontData::from_blob(blob, index))
}

/// Gives egui a Simplified-Chinese font so Chinese instructions and answers show up.
///
/// The face index matters: these collections hold several regional variants
/// (PingFang HK/TC/SC, Heiti TC/SC) and index 0 is not always the
/// Simplified-Chinese face, so it is chosen per file.
pub fn install_fonts(ctx: &egui::Context) {
    let mut candidates: Vec<CjkFont> = Vec::new();
    // 苹方-简 (PingFang SC) is the standard Simplified-Chinese font on macOS,
    // and its Latin matches the Chinese, so it takes over both.
    if let Some(path) = pingfang_path() {
        candidates.push(CjkFont {
            path,
            index: 3, // PingFang SC Regular
            primary: true,
        });
    }
    // Fallbacks, used only for the glyphs the main font is missing.
    for (path, index) in [
        ("/System/Library/Fonts/PingFang.ttc", 3),
        ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
        ("/System/Library/Fonts/STHeiti Medium.ttc", 1), // Heiti SC Medium
        ("/System/Library/Fonts/STHeiti Light.ttc", 1),
        ("/System/Library/Fonts/Supplemental/Songti.ttc", 0),
        ("C:\\Windows\\Fonts\\msyh.ttc", 0),
        ("C:\\Windows\\Fonts\\simhei.ttf", 0),
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 2),
        ("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc", 2),
    ] {
        candidates.push(CjkFont {
            path: std::path::PathBuf::from(path),
            index,
            primary: false,
        });
    }

    let Some(font) = candidates.iter().find(|font| font.path.is_file()) else {
        return;
    };
    let Some(data) = open_font_data(&font.path, font.index) else {
        return;
    };
    let families = vec![
        egui::InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: if font.primary {
                egui::FontPriority::Highest
            } else {
                egui::FontPriority::Lowest
            },
        },
        // Code keeps its monospace Latin and borrows the Chinese.
        egui::InsertFontFamily {
            family: egui::FontFamily::Monospace,
            priority: egui::FontPriority::Lowest,
        },
    ];
    ctx.add_font(egui::FontInsert::new("cjk-fallback", data, families));
}

/// Switches the palette to match the theme egui resolved for this frame.
pub fn sync_palette(ctx: &egui::Context) {
    set_palette(match ctx.theme() {
        egui::Theme::Dark => Palette::dark(),
        egui::Theme::Light => Palette::light(),
    });
}

pub fn draw(app: &mut ClipFlow, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    sync_palette(&ctx);
    paint_wallpaper(app, ui);
    let central_fill = if app.background.is_some() {
        Color32::TRANSPARENT
    } else {
        pal().bg
    };
    sidebar(app, ui);
    status_bar(app, ui);
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(central_fill)
                .inner_margin(margin(26, 18)),
        )
        .show(ui, |ui| {
            header(app, ui);
            ui.add_space(16.0);
            match app.filter {
                Filter::Snippets => snippets_view(app, ui),
                Filter::Settings => settings_view(app, ui),
                _ => list_view(app, ui),
            }
        });
    ai_window(app, &ctx);
    toast(app, &ctx);
}

fn paint_wallpaper(app: &ClipFlow, ui: &Ui) {
    let Some(texture) = &app.background else {
        return;
    };
    let rect = ui.max_rect();
    let uv = cover_uv(app.background_size, rect.size());
    ui.painter()
        .image(texture.id(), rect, uv, Color32::WHITE);
    let dim = app.settings.bg_dim.clamp(0.0, 0.95);
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, tint(pal().bg, (dim * 255.0) as u8));
}

/// Crops the picture like `background-size: cover` instead of stretching it.
fn cover_uv(image: Vec2, target: Vec2) -> Rect {
    let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    if image.x <= 0.0 || image.y <= 0.0 || target.x <= 0.0 || target.y <= 0.0 {
        return full;
    }
    let image_ratio = image.x / image.y;
    let target_ratio = target.x / target.y;
    if image_ratio > target_ratio {
        let margin = (1.0 - target_ratio / image_ratio) / 2.0;
        Rect::from_min_max(pos2(margin, 0.0), pos2(1.0 - margin, 1.0))
    } else {
        let margin = (1.0 - image_ratio / target_ratio) / 2.0;
        Rect::from_min_max(pos2(0.0, margin), pos2(1.0, 1.0 - margin))
    }
}

/// Panels turn slightly see-through when a background picture is set.
fn glass(app: &ClipFlow, color: Color32, alpha: u8) -> Color32 {
    if app.background.is_some() {
        tint(color, alpha)
    } else {
        color
    }
}

// ---------- sidebar ----------

fn sidebar(app: &mut ClipFlow, ui: &mut egui::Ui) {
    let fill = glass(app, pal().panel, 226);
    egui::Panel::left("nav")
        .exact_size(228.0)
        .resizable(false)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(fill)
                .inner_margin(margin(14, 18)),
        )
        .show(ui, |ui| {
            brand(app, ui);
            let lang = app.settings.language;
            ui.add_space(24.0);
            section_label(ui, lang.t("LIBRARY", "资料库"));
            ui.add_space(6.0);

            let (all, pinned, links, code) = app.counts();
            let mut next: Option<Filter> = None;
            if nav_item(
                ui,
                lang.t("All clips", "全部剪贴"),
                NavIcon::Stack,
                Some(all),
                app.filter == Filter::All,
            ) {
                next = Some(Filter::All);
            }
            if nav_item(
                ui,
                lang.t("Pinned", "已固定"),
                NavIcon::Diamond,
                Some(pinned),
                app.filter == Filter::Pinned,
            ) {
                next = Some(Filter::Pinned);
            }
            if nav_item(
                ui,
                lang.t("Links", "链接"),
                NavIcon::Link,
                Some(links),
                app.filter == Filter::Links,
            ) {
                next = Some(Filter::Links);
            }
            if nav_item(
                ui,
                lang.t("Code", "代码"),
                NavIcon::Code,
                Some(code),
                app.filter == Filter::Code,
            ) {
                next = Some(Filter::Code);
            }
            if let Some(f) = next {
                app.filter = f;
                app.selected = None;
                app.search.clear();
            }

            ui.add_space(14.0);
            section_label(ui, lang.t("TOOLS", "工具"));
            ui.add_space(6.0);
            if nav_item(
                ui,
                lang.t("Snippets", "片段"),
                NavIcon::Doc,
                Some(app.snippets.len()),
                app.filter == Filter::Snippets,
            ) {
                app.filter = Filter::Snippets;
                app.search.clear();
            }
            if nav_item(
                ui,
                lang.t("Settings", "设置"),
                NavIcon::Gear,
                None,
                app.filter == Filter::Settings,
            ) {
                app.filter = Filter::Settings;
                app.search.clear();
                app.selected = None;
            }

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {} {}",
                        lang.t("Newest", "最近"),
                        MAX_HISTORY,
                        lang.t("clips kept on this Mac", "条剪贴保存在这台 Mac 上")
                    ))
                    .size(10.5)
                    .color(pal().muted),
                );
                ui.add_space(10.0);
                clear_history_button(app, ui, lang);
                ui.add_space(8.0);
                pause_row(app, ui, lang);
                ui.add_space(12.0);
            });
        });
}

fn brand(app: &ClipFlow, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(10), pal().accent);
        let inner = rect.shrink(8.0);
        p.rect_filled(
            Rect::from_min_size(
                pos2(inner.left(), inner.top()),
                vec2(inner.width() - 4.0, 3.0),
            ),
            CornerRadius::same(2),
            Color32::from_white_alpha(230),
        );
        p.rect_filled(
            Rect::from_min_size(
                pos2(inner.left() + 2.0, inner.top() + 5.0),
                vec2(inner.width() - 4.0, 3.0),
            ),
            CornerRadius::same(2),
            Color32::from_white_alpha(180),
        );
        p.rect_filled(
            Rect::from_min_size(
                pos2(inner.left(), inner.top() + 10.0),
                vec2(inner.width() - 8.0, 3.0),
            ),
            CornerRadius::same(2),
            Color32::from_white_alpha(130),
        );
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("ClipFlow").size(16.5).color(pal().text));
            ui.add_space(1.0);
            ui.label(
                RichText::new(
                    app.settings
                        .language
                        .t("Clipboard history", "剪贴历史"),
                )
                .size(10.5)
                .color(pal().muted),
            );
        });
    });
}

fn section_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(10.0).color(pal().muted));
}

#[derive(Clone, Copy)]
enum NavIcon {
    Stack,
    Diamond,
    Link,
    Code,
    Doc,
    Gear,
}

fn nav_item(
    ui: &mut Ui,
    label: &str,
    icon: NavIcon,
    count: Option<usize>,
    selected: bool,
) -> bool {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let hovered = response.hovered();
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(8), pal().accent_soft);
    } else if hovered {
        p.rect_filled(rect, CornerRadius::same(8), pal().card);
    }
    let icon_color = if selected { pal().accent } else { pal().text2 };
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 17.0, rect.center().y), Vec2::splat(14.0));
    paint_nav_icon(p, icon_rect, icon, icon_color);
    p.text(
        pos2(rect.left() + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        if selected { pal().accent_text } else { pal().text2 },
    );
    if let Some(count) = count {
        p.text(
            pos2(rect.right() - 10.0, rect.center().y),
            Align2::RIGHT_CENTER,
            count.to_string(),
            FontId::proportional(11.0),
            if selected { pal().accent } else { pal().muted },
        );
    }
    response.clicked()
}

fn pause_row(app: &mut ClipFlow, ui: &mut Ui, lang: Language) {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 30.0), Sense::click());
    let p = ui.painter();
    if response.hovered() {
        p.rect_filled(rect, CornerRadius::same(8), pal().card);
    }
    p.text(
        pos2(rect.left() + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        lang.t("Pause history", "暂停记录"),
        FontId::proportional(12.5),
        pal().text2,
    );
    let track = Rect::from_center_size(pos2(rect.right() - 27.0, rect.center().y), vec2(30.0, 16.0));
    p.rect_filled(
        track,
        CornerRadius::same(8),
        if app.paused { pal().accent } else { pal().border_hover },
    );
    let knob_x = if app.paused {
        track.right() - 8.0
    } else {
        track.left() + 8.0
    };
    p.circle_filled(pos2(knob_x, track.center().y), 5.5, pal().panel);
    if response.clicked() {
        let paused = !app.paused;
        app.pause(paused);
    }
}

fn clear_history_button(app: &mut ClipFlow, ui: &mut Ui, lang: Language) {
    let armed = app.now < app.clear_armed_until;
    let label = if armed {
        lang.t("Click again to confirm", "再点一次确认")
    } else {
        lang.t("Clear history", "清空历史")
    };
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    let p = ui.painter();
    let (fill, stroke, color) = if armed {
        (pal().danger_soft, pal().danger_border, pal().danger)
    } else if response.hovered() {
        (pal().card_hover, pal().border_hover, pal().text)
    } else {
        (Color32::TRANSPARENT, pal().border, pal().text2)
    };
    p.rect_filled(rect, CornerRadius::same(8), fill);
    p.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        color,
    );
    if response.clicked() {
        if armed {
            app.clear_history();
            app.clear_armed_until = 0.0;
        } else {
            app.clear_armed_until = app.now + 3.0;
        }
    }
    if armed {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
    }
}

// ---------- status bar ----------

fn status_bar(app: &mut ClipFlow, ui: &mut egui::Ui) {
    let fill = glass(app, pal().panel, 226);
    egui::Panel::bottom("status")
        .exact_size(32.0)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(fill)
                .inner_margin(margin(20, 0)),
        )
        .show(ui, |ui| {
            let lang = app.settings.language;
            ui.horizontal_centered(|ui| {
                let (dot, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                ui.painter().circle_filled(
                    dot.center(),
                    3.5,
                    if app.paused { pal().warn } else { pal().success },
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(if app.paused {
                        lang.t("Paused", "已暂停")
                    } else {
                        lang.t("Watching the clipboard", "正在监听剪贴板")
                    })
                    .size(11.5)
                    .color(pal().text2),
                );
                ui.add_space(10.0);
                ui.label(RichText::new("•").size(11.0).color(pal().border_hover));
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!(
                        "{} {}  ·  {} {}  ·  {} {}",
                        app.clips.len(),
                        lang.t("clips", "条剪贴"),
                        app.clips.iter().filter(|c| c.pinned).count(),
                        lang.t("pinned", "已固定"),
                        app.snippets.len(),
                        lang.t("snippets", "片段"),
                    ))
                    .size(11.5)
                    .color(pal().muted),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(lang.t(
                            "⌘F search  ·  ↑↓ select  ·  ↵ copy",
                            "⌘F 搜索  ·  ↑↓ 选择  ·  ↵ 复制",
                        ))
                        .size(11.0)
                        .color(pal().muted),
                    );
                });
            });
        });
}

// ---------- header ----------

fn header(app: &mut ClipFlow, ui: &mut Ui) {
    let lang = app.settings.language;
    let (title, subtitle) = match app.filter {
        Filter::All => (
            lang.t("All clips", "全部剪贴"),
            format!(
                "{} {}",
                app.clips.len(),
                lang.t("clips in your history", "条剪贴记录")
            ),
        ),
        Filter::Pinned => (
            lang.t("Pinned", "已固定"),
            format!(
                "{} {}",
                app.clips.iter().filter(|c| c.pinned).count(),
                lang.t("clips kept at the top", "条固定在顶部")
            ),
        ),
        Filter::Links => (
            lang.t("Links", "链接"),
            format!(
                "{} {}",
                app.clips
                    .iter()
                    .filter(|c| detect_kind(&c.text) == Kind::Link)
                    .count(),
                lang.t("links copied recently", "个最近复制的链接")
            ),
        ),
        Filter::Code => (
            lang.t("Code", "代码"),
            format!(
                "{} {}",
                app.clips
                    .iter()
                    .filter(|c| detect_kind(&c.text) == Kind::Code)
                    .count(),
                lang.t("code clips", "条代码剪贴")
            ),
        ),
        Filter::Snippets => (
            lang.t("Snippets", "片段"),
            format!(
                "{} {}",
                app.snippets.len(),
                lang.t("saved pieces of text", "段已保存的文字")
            ),
        ),
        Filter::Settings => (
            lang.t("Settings", "设置"),
            lang.t("Appearance and the AI connection", "外观与 AI 连接").to_string(),
        ),
    };

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(22.0).color(pal().text));
            ui.add_space(1.0);
            ui.label(RichText::new(subtitle).size(12.0).color(pal().muted));
        });
        if app.filter == Filter::Settings {
            return;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.filter == Filter::Snippets {
                let button = egui::Button::new(
                    RichText::new(lang.t("Paste here", "粘贴到这里"))
                        .size(12.5)
                        .color(Color32::WHITE),
                )
                .fill(pal().accent)
                .corner_radius(CornerRadius::same(9))
                .min_size(vec2(0.0, 32.0));
                if ui
                    .add(button)
                    .on_hover_text(lang.t("Save a new snippet below", "在下方保存新片段"))
                    .clicked()
                {
                    app.snippet_name.clear();
                    app.snippet_text.clear();
                }
                ui.add_space(10.0);
            }
            search_box(app, ui, lang);
        });
    });
}

fn search_box(app: &mut ClipFlow, ui: &mut Ui, lang: Language) {
    let id = egui::Id::new("search_field");
    let (rect, _) = ui.allocate_exact_size(vec2(252.0, 34.0), Sense::hover());
    let focused = ui.memory(|m| m.has_focus(id));
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(9), pal().input);
    p.rect_stroke(
        rect,
        CornerRadius::same(9),
        Stroke::new(1.0, if focused { pal().accent } else { pal().border }),
        StrokeKind::Inside,
    );
    paint_magnifier(
        p,
        pos2(rect.left() + 16.0, rect.center().y),
        if focused { pal().accent } else { pal().muted },
    );

    let bg_click = ui.interact(rect, id.with("area"), Sense::click());
    if bg_click.clicked() {
        ui.memory_mut(|m| m.request_focus(id));
    }

    let text_rect = Rect::from_min_max(
        pos2(rect.left() + 30.0, rect.top() + 7.0),
        pos2(rect.right() - 12.0, rect.bottom() - 7.0),
    );
    let response = ui.put(
        text_rect,
        egui::TextEdit::singleline(&mut app.search)
            .id(id)
            .frame(egui::Frame::NONE)
            .hint_text(
                RichText::new(lang.t("Search clips...", "搜索剪贴…"))
                    .size(13.0)
                    .color(pal().muted),
            )
            .desired_width(text_rect.width()),
    );
    if response.changed() {
        app.selected = None;
    }
}

// ---------- list ----------

fn list_view(app: &mut ClipFlow, ui: &mut Ui) {
    let clips = app.visible_clips();
    if clips.is_empty() {
        empty_state(app, ui, app.search.is_empty());
        return;
    }

    let mut actions: Vec<Action> = Vec::new();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            for clip in &clips {
                if let Some(action) = clip_card(app, ui, clip) {
                    actions.push(action);
                }
            }
            ui.add_space(8.0);
        });
    for action in actions {
        app.apply(action);
    }
}

fn clip_card(app: &ClipFlow, ui: &mut Ui, clip: &Clip) -> Option<Action> {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 74.0), Sense::hover());
    let selected = app.selected == Some(clip.id);
    let card_id = ui.id().with(("clip", clip.id));
    let card_click = ui.interact(rect, card_id, Sense::click());
    let hovered = card_click.hovered() || ui.rect_contains_pointer(rect);

    let p = ui.painter();
    let fill = if selected {
        glass(app, pal().card_selected, 238)
    } else if hovered {
        glass(app, pal().card_hover, 238)
    } else {
        glass(app, pal().card, 238)
    };
    let border = if selected {
        pal().accent_border
    } else if hovered {
        pal().border_hover
    } else {
        pal().border
    };
    p.rect_filled(rect, CornerRadius::same(12), fill);
    p.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    if selected {
        p.rect_filled(
            Rect::from_min_size(
                pos2(rect.left() + 1.0, rect.center().y - 14.0),
                vec2(3.0, 28.0),
            ),
            CornerRadius::same(2),
            pal().accent,
        );
    }

    let lang = app.settings.language;
    let kind = detect_kind(&clip.text);
    let (kind_color, _) = kind_style(kind, lang);
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 32.0, rect.center().y), Vec2::splat(30.0));
    p.rect_filled(icon_rect, CornerRadius::same(9), tint(kind_color, 30));
    paint_kind_icon(p, icon_rect.shrink(8.0), kind, kind_color);

    // text block
    let text_left = rect.left() + 56.0;
    let text_right = rect.right() - 188.0;
    let text_width = (text_right - text_left).max(40.0);
    let preview = first_line(&clip.text);
    let preview_font = if kind == Kind::Code {
        FontId::monospace(12.5)
    } else {
        FontId::proportional(13.5)
    };
    let galley = one_line(ui, &preview, preview_font, pal().text, text_width);
    p.galley(pos2(text_left, rect.top() + 15.0), galley, pal().text);
    let meta = format!(
        "{}  ·  {}  ·  {} {}",
        clip.source,
        rel_time(clip.created_ms, app.settings.language),
        clip.text.chars().count(),
        app.settings.language.t("chars", "个字符")
    );
    let meta_galley = one_line(
        ui,
        &meta,
        FontId::proportional(11.0),
        pal().muted,
        text_width,
    );
    p.galley(pos2(text_left, rect.top() + 40.0), meta_galley, pal().muted);

    // Buttons: one row of the same height, corner radius and spacing. The row is
    // laid out from the right edge inward so the gaps stay even by construction.
    let btn_h = 26.0;
    let gap = 6.0;
    let copy_w = 56.0;
    let icon_w = 30.0;
    let del_rect = Rect::from_center_size(
        pos2(rect.right() - 12.0 - icon_w / 2.0, rect.center().y),
        vec2(icon_w, btn_h),
    );
    let pin_rect = Rect::from_center_size(
        pos2(del_rect.left() - gap - icon_w / 2.0, rect.center().y),
        vec2(icon_w, btn_h),
    );
    let ai_rect = Rect::from_center_size(
        pos2(pin_rect.left() - gap - icon_w / 2.0, rect.center().y),
        vec2(icon_w, btn_h),
    );
    let copy_rect = Rect::from_center_size(
        pos2(ai_rect.left() - gap - copy_w / 2.0, rect.center().y),
        vec2(copy_w, btn_h),
    );
    let copy_resp = ui.interact(copy_rect, card_id.with("copy"), Sense::click());
    let ai_resp = ui
        .interact(ai_rect, card_id.with("ai"), Sense::click())
        .on_hover_text(lang.t("Analyse this clip with AI", "用 AI 分析这条剪贴"));
    let pin_resp = ui
        .interact(pin_rect, card_id.with("pin"), Sense::click())
        .on_hover_text(lang.t(
            if clip.pinned { "Unpin this clip" } else { "Pin this clip" },
            if clip.pinned { "取消固定这条剪贴" } else { "固定这条剪贴" },
        ));
    let del_resp = ui
        .interact(del_rect, card_id.with("delete"), Sense::click())
        .on_hover_text(lang.t("Remove this clip", "移除这条剪贴"));

    let p = ui.painter();
    let rest_fill = tint(pal().text, 12);
    let hover_fill = tint(pal().text, 16);
    // Copy is the primary action, AI the accent-coloured one, and pin/remove are
    // the same chip in a quiet colour so all four read as one set.
    p.rect_filled(
        copy_rect,
        CornerRadius::same(7),
        if copy_resp.hovered() { pal().accent } else { rest_fill },
    );
    p.text(
        copy_rect.center(),
        Align2::CENTER_CENTER,
        lang.t("Copy", "复制"),
        FontId::proportional(11.0),
        if copy_resp.hovered() {
            Color32::WHITE
        } else {
            pal().text2
        },
    );
    p.rect_filled(
        ai_rect,
        CornerRadius::same(7),
        if ai_resp.hovered() {
            pal().accent
        } else {
            tint(pal().accent, 34)
        },
    );
    p.text(
        ai_rect.center(),
        Align2::CENTER_CENTER,
        "AI",
        FontId::proportional(10.5),
        if ai_resp.hovered() {
            Color32::WHITE
        } else {
            pal().accent_text
        },
    );
    p.rect_filled(
        pin_rect,
        CornerRadius::same(7),
        if pin_resp.hovered() { hover_fill } else { rest_fill },
    );
    paint_diamond(
        p,
        pin_rect.shrink2(vec2(8.0, 6.0)),
        if clip.pinned {
            pal().warn
        } else if pin_resp.hovered() {
            pal().text
        } else {
            pal().text2
        },
        clip.pinned,
    );
    p.rect_filled(
        del_rect,
        CornerRadius::same(7),
        if del_resp.hovered() { hover_fill } else { rest_fill },
    );
    p.text(
        del_rect.center(),
        Align2::CENTER_CENTER,
        "×",
        FontId::proportional(14.0),
        if del_resp.hovered() { pal().text } else { pal().text2 },
    );

    if copy_resp.clicked() {
        return Some(Action::CopyClip(clip.id));
    }
    if pin_resp.clicked() {
        return Some(Action::TogglePin(clip.id));
    }
    if del_resp.clicked() {
        return Some(Action::DeleteClip(clip.id));
    }
    if ai_resp.clicked() {
        return Some(Action::OpenAi(clip.id));
    }
    if card_click.clicked() {
        return Some(Action::CopyClip(clip.id));
    }
    None
}

// ---------- snippets ----------

fn snippets_view(app: &mut ClipFlow, ui: &mut Ui) {
    snippet_composer(app, ui);
    ui.add_space(14.0);

    let snippets = app.visible_snippets();
    if snippets.is_empty() {
        empty_state(app, ui, app.search.is_empty());
        return;
    }

    let mut actions: Vec<Action> = Vec::new();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            for snippet in &snippets {
                if let Some(action) = snippet_card(app, ui, snippet) {
                    actions.push(action);
                }
            }
            ui.add_space(8.0);
        });
    for action in actions {
        app.apply(action);
    }
}

fn snippet_composer(app: &mut ClipFlow, ui: &mut Ui) {
    let lang = app.settings.language;
    let mut save = false;
    let fill = glass(app, pal().card, 238);
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, pal().border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(margin(14, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.snippet_name)
                        .id(egui::Id::new("snippet_name"))
                        .frame(egui::Frame::NONE)
                        .hint_text(
                            RichText::new(lang.t("Name (optional)", "名称（可选）"))
                                .size(12.5)
                                .color(pal().muted),
                        )
                        .desired_width(220.0),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let button = egui::Button::new(
                        RichText::new(lang.t("Save snippet", "保存片段"))
                            .size(12.0)
                            .color(Color32::WHITE),
                    )
                    .fill(pal().accent)
                    .corner_radius(CornerRadius::same(8))
                    .min_size(vec2(0.0, 28.0));
                    if ui.add(button).clicked() {
                        save = true;
                    }
                });
            });
            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::multiline(&mut app.snippet_text)
                    .id(egui::Id::new("snippet_text"))
                    .frame(egui::Frame::NONE)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .hint_text(
                        RichText::new(lang.t("Text you paste often...", "经常要粘贴的文字…"))
                            .size(12.5)
                            .color(pal().muted),
                    ),
            );
        });
    if save {
        app.add_snippet();
    }
}

fn snippet_card(app: &ClipFlow, ui: &mut Ui, snippet: &Snippet) -> Option<Action> {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 66.0), Sense::hover());
    let card_id = ui.id().with(("snippet", snippet.id));
    let card_click = ui.interact(rect, card_id, Sense::click());
    let hovered = card_click.hovered() || ui.rect_contains_pointer(rect);

    let p = ui.painter();
    p.rect_filled(
        rect,
        CornerRadius::same(12),
        if hovered {
            glass(app, pal().card_hover, 238)
        } else {
            glass(app, pal().card, 238)
        },
    );
    p.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, if hovered { pal().border_hover } else { pal().border }),
        StrokeKind::Inside,
    );

    let (color, _) = kind_style(Kind::Snippet, app.settings.language);
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 32.0, rect.center().y), Vec2::splat(28.0));
    p.rect_filled(icon_rect, CornerRadius::same(8), tint(color, 30));
    paint_kind_icon(p, icon_rect.shrink(8.0), Kind::Snippet, color);

    let text_left = rect.left() + 54.0;
    let text_right = rect.right() - 116.0;
    let text_width = (text_right - text_left).max(40.0);
    let galley = one_line(
        ui,
        &snippet.name,
        FontId::proportional(13.5),
        pal().text,
        text_width,
    );
    p.galley(pos2(text_left, rect.top() + 13.0), galley, pal().text);
    let preview = first_line(&snippet.text);
    let preview_galley = one_line(
        ui,
        &preview,
        FontId::proportional(11.0),
        pal().muted,
        text_width,
    );
    p.galley(pos2(text_left, rect.top() + 36.0), preview_galley, pal().muted);

    // Same button set as the clip cards: equal height, radius and gap.
    let btn_h = 26.0;
    let gap = 6.0;
    let copy_w = 56.0;
    let icon_w = 30.0;
    let del_rect = Rect::from_center_size(
        pos2(rect.right() - 12.0 - icon_w / 2.0, rect.center().y),
        vec2(icon_w, btn_h),
    );
    let copy_rect = Rect::from_center_size(
        pos2(del_rect.left() - gap - copy_w / 2.0, rect.center().y),
        vec2(copy_w, btn_h),
    );
    let copy_resp = ui.interact(copy_rect, card_id.with("copy"), Sense::click());
    let del_resp = ui
        .interact(del_rect, card_id.with("delete"), Sense::click())
        .on_hover_text(app.settings.language.t("Remove this snippet", "移除这个片段"));

    let p = ui.painter();
    p.rect_filled(
        copy_rect,
        CornerRadius::same(7),
        if copy_resp.hovered() {
            pal().accent
        } else {
            tint(pal().text, 12)
        },
    );
    p.text(
        copy_rect.center(),
        Align2::CENTER_CENTER,
        app.settings.language.t("Copy", "复制"),
        FontId::proportional(11.0),
        if copy_resp.hovered() { Color32::WHITE } else { pal().text2 },
    );
    p.rect_filled(
        del_rect,
        CornerRadius::same(7),
        if del_resp.hovered() {
            tint(pal().text, 16)
        } else {
            tint(pal().text, 12)
        },
    );
    p.text(
        del_rect.center(),
        Align2::CENTER_CENTER,
        "×",
        FontId::proportional(14.0),
        if del_resp.hovered() { pal().text } else { pal().text2 },
    );

    if copy_resp.clicked() || card_click.clicked() {
        return Some(Action::CopySnippet(snippet.id));
    }
    if del_resp.clicked() {
        return Some(Action::DeleteSnippet(snippet.id));
    }
    None
}

// ---------- settings ----------

fn settings_view(app: &mut ClipFlow, ui: &mut Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_max_width(680.0);
            appearance_card(app, ui);
            ui.add_space(12.0);
            ai_settings_card(app, ui);
            ui.add_space(10.0);
        });
}

fn settings_card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(pal().card)
        .stroke(Stroke::new(1.0, pal().border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(margin(18, 16))
        .show(ui, add);
}

fn section_title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(14.0).color(pal().text));
}

fn field_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(pal().text2));
}

fn input_frame(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(pal().input)
        .stroke(Stroke::new(1.0, pal().border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(margin(10, 7))
        .show(ui, add);
}

fn appearance_card(app: &mut ClipFlow, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    settings_card(ui, |ui| {
        let lang = app.settings.language;
        section_title(ui, lang.t("Appearance", "外观"));
        ui.label(
            RichText::new(lang.t(
                "Pick the look you like. ClipFlow remembers it for next time.",
                "选一个你喜欢的样式，ClipFlow 会记住。",
            ))
            .size(11.5)
            .color(pal().muted),
        );
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let options = [
                (
                    ThemeChoice::Dark,
                    lang.t("Dark", "深色"),
                    lang.t("Calm and dim", "安静、柔和"),
                ),
                (
                    ThemeChoice::Light,
                    lang.t("Light", "浅色"),
                    lang.t("Bright and crisp", "明亮、清爽"),
                ),
                (
                    ThemeChoice::System,
                    lang.t("Match system", "跟随系统"),
                    lang.t("Follows your Mac", "跟随你的 Mac"),
                ),
            ];
            for (choice, label, hint) in options {
                if theme_tile(ui, choice, label, hint, app.settings.theme == choice).clicked() {
                    app.set_theme(&ctx, choice);
                }
            }
        });

        ui.add_space(18.0);
        ui.label(
            RichText::new(lang.t("Language", "语言"))
                .size(12.5)
                .color(pal().text),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(lang.t(
                "The words on screen. English is the default; choose Chinese to read ClipFlow \
                 in 简体中文.",
                "界面文字。默认为英文，选择中文后以简体中文显示。",
            ))
            .size(11.0)
            .color(pal().muted),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let options = [
                (
                    Language::English,
                    lang.t("English", "English"),
                    lang.t("The original words", "默认英文"),
                ),
                (
                    Language::Chinese,
                    lang.t("简体中文", "简体中文"),
                    lang.t("In Simplified Chinese", "界面显示简体中文"),
                ),
            ];
            for (choice, label, hint) in options {
                if theme_tile(
                    ui,
                    ThemeChoice::Dark,
                    label,
                    hint,
                    app.settings.language == choice,
                )
                .clicked()
                {
                    app.set_language(choice);
                }
            }
        });

        ui.add_space(18.0);
        ui.label(
            RichText::new(lang.t("Background picture", "背景图片"))
                .size(12.5)
                .color(pal().text),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(lang.t(
                "Use a picture of your own behind the clips. The file stays where it is on your \
                 Mac; ClipFlow only reads it.",
                "在剪贴后面放一张自己的图片。文件留在原处，ClipFlow 只读取它。",
            ))
            .size(11.0)
            .color(pal().muted),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let choose = egui::Button::new(
                RichText::new(lang.t("Choose image…", "选择图片…"))
                    .size(12.0)
                    .color(Color32::WHITE),
            )
            .fill(pal().accent)
            .corner_radius(CornerRadius::same(8))
            .min_size(vec2(0.0, 30.0));
            if ui.add(choose).clicked() {
                app.choose_background(&ctx);
            }
            if !app.settings.bg_image.is_empty() {
                let remove = egui::Button::new(
                    RichText::new(lang.t("Remove", "移除")).size(12.0).color(pal().text2),
                )
                .fill(pal().input)
                .stroke(Stroke::new(1.0, pal().border))
                .corner_radius(CornerRadius::same(8))
                .min_size(vec2(0.0, 30.0));
                if ui.add(remove).clicked() {
                    app.clear_background();
                }
            }
        });

        if let Some(texture) = &app.background {
            ui.add_space(10.0);
            let mut dim = app.settings.bg_dim;
            let mut dim_changed = false;
            ui.horizontal(|ui| {
                let preview = egui::Image::from_texture(egui::load::SizedTexture::new(
                    texture.id(),
                    vec2(160.0, 90.0),
                ))
                .corner_radius(CornerRadius::same(8));
                ui.add(preview);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    let name = std::path::Path::new(app.settings.bg_image.trim())
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| app.settings.bg_image.clone());
                    ui.add(
                        egui::Label::new(RichText::new(name).size(11.5).color(pal().text))
                            .wrap_mode(egui::TextWrapMode::Truncate),
                    );
                    ui.add_space(2.0);
                    dim_changed |= ui
                        .add(
                            egui::Slider::new(&mut dim, 0.15..=0.9)
                                .show_value(false)
                                .text(
                                    RichText::new(lang.t("Dim", "压暗"))
                                        .size(10.5)
                                        .color(pal().muted),
                                ),
                        )
                        .changed();
                });
            });
            if dim_changed {
                app.settings.bg_dim = dim;
                app.save_settings();
            }
        }

        if let Some(error) = &app.bg_error {
            ui.add_space(6.0);
            ui.add(
                egui::Label::new(RichText::new(error).size(11.0).color(pal().danger))
                    .wrap_mode(egui::TextWrapMode::Wrap),
            );
        } else if !app.settings.bg_image.is_empty() && app.background.is_none() {
            ui.add_space(6.0);
            ui.label(
                RichText::new(lang.t("The picture could not be loaded.", "图片无法加载。"))
                    .size(11.0)
                    .color(pal().danger),
            );
        }
    });
}

fn theme_tile(
    ui: &mut Ui,
    swatch: ThemeChoice,
    label: &str,
    hint: &str,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(158.0, 62.0), Sense::click());
    let p = ui.painter();
    let fill = if selected {
        pal().accent_soft
    } else if response.hovered() {
        pal().card_hover
    } else {
        pal().input
    };
    p.rect_filled(rect, CornerRadius::same(9), fill);
    p.rect_stroke(
        rect,
        CornerRadius::same(9),
        Stroke::new(1.0, if selected { pal().accent } else { pal().border }),
        StrokeKind::Inside,
    );

    let swatch_rect = Rect::from_min_size(pos2(rect.left() + 12.0, rect.top() + 15.0), Vec2::splat(22.0));
    match swatch {
        ThemeChoice::Dark => {
            p.rect_filled(swatch_rect, CornerRadius::same(6), Color32::from_rgb(0x12, 0x14, 0x18));
        }
        ThemeChoice::Light => {
            p.rect_filled(swatch_rect, CornerRadius::same(6), Color32::WHITE);
            p.rect_stroke(
                swatch_rect,
                CornerRadius::same(6),
                Stroke::new(1.0, Color32::from_rgb(0xD5, 0xDA, 0xE4)),
                StrokeKind::Inside,
            );
        }
        ThemeChoice::System => {
            let left = Rect::from_min_max(swatch_rect.min, pos2(swatch_rect.center().x, swatch_rect.bottom()));
            let right = Rect::from_min_max(pos2(swatch_rect.center().x, swatch_rect.top()), swatch_rect.max);
            p.rect_filled(left, CornerRadius::same(6), Color32::from_rgb(0x12, 0x14, 0x18));
            p.rect_filled(right, CornerRadius::same(6), Color32::WHITE);
        }
    }
    p.circle_filled(swatch_rect.center(), 4.0, pal().accent);

    p.text(
        pos2(rect.left() + 44.0, rect.top() + 14.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(12.5),
        if selected { pal().accent_text } else { pal().text },
    );
    p.text(
        pos2(rect.left() + 44.0, rect.top() + 34.0),
        Align2::LEFT_TOP,
        hint,
        FontId::proportional(10.5),
        pal().muted,
    );
    if selected {
        p.circle_filled(pos2(rect.right() - 14.0, rect.top() + 15.0), 3.5, pal().accent);
    }
    response
}

fn ai_settings_card(app: &mut ClipFlow, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    settings_card(ui, |ui| {
        let lang = app.settings.language;
        let mut changed = false;
        section_title(ui, lang.t("AI connection", "AI 连接"));
        ui.label(
            RichText::new(lang.t(
                "The AI button on each clip sends that text to this address and shows the \
                 answer. It is used only for that one request.",
                "每条剪贴上的 AI 按钮会把文字发送到这里并显示回答，只用于这一次请求。",
            ))
            .size(11.5)
            .color(pal().muted),
        );
        ui.add_space(12.0);

        field_label(ui, lang.t("Request URL", "请求地址"));
        ui.add_space(3.0);
        input_frame(ui, |ui| {
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut app.settings.ai_url)
                        .id(egui::Id::new("ai_url"))
                        .frame(egui::Frame::NONE)
                        .desired_width(f32::INFINITY)
                        .hint_text(
                            RichText::new("https://api.openai.com/v1")
                                .size(12.5)
                                .color(pal().muted),
                        ),
                )
                .changed();
        });
        ui.add_space(8.0);

        field_label(ui, lang.t("Model", "模型"));
        ui.add_space(3.0);
        input_frame(ui, |ui| {
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut app.settings.ai_model)
                        .id(egui::Id::new("ai_model"))
                        .frame(egui::Frame::NONE)
                        .desired_width(f32::INFINITY)
                        .hint_text(
                            RichText::new("gpt-4o-mini").size(12.5).color(pal().muted),
                        ),
                )
                .changed();
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            field_label(ui, lang.t("API key", "API 密钥"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = if app.show_key {
                    lang.t("Hide", "隐藏")
                } else {
                    lang.t("Show", "显示")
                };
                if ui
                    .link(RichText::new(label).size(10.5).color(pal().accent))
                    .clicked()
                {
                    app.show_key = !app.show_key;
                }
            });
        });
        ui.add_space(3.0);
        input_frame(ui, |ui| {
            let mut edit = egui::TextEdit::singleline(&mut app.settings.ai_key)
                .id(egui::Id::new("ai_key"))
                .frame(egui::Frame::NONE)
                .desired_width(f32::INFINITY)
                .hint_text(RichText::new("sk-…").size(12.5).color(pal().muted));
            if !app.show_key {
                edit = edit.password(true);
            }
            changed |= ui.add(edit).changed();
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let label = if app.test.running {
                lang.t("Testing…", "测试中…")
            } else {
                lang.t("Test connection", "测试连接")
            };
            let button = egui::Button::new(
                RichText::new(label).size(12.0).color(if app.test.running {
                    pal().muted
                } else {
                    pal().text
                }),
            )
            .fill(pal().input)
            .stroke(Stroke::new(1.0, pal().border))
            .corner_radius(CornerRadius::same(8))
            .min_size(vec2(0.0, 32.0));
            if ui.add_enabled(!app.test.running, button).clicked() {
                app.test_connection(&ctx);
            }
            if app.test.running {
                ui.add(egui::Spinner::new().size(14.0));
            }
        });
        if let Some((ok, message)) = &app.test.message {
            ui.add_space(6.0);
            ui.add(
                egui::Label::new(
                    RichText::new(message)
                        .size(11.0)
                        .color(if *ok { pal().success } else { pal().danger }),
                )
                .wrap_mode(egui::TextWrapMode::Wrap),
            );
        }

        ui.add_space(10.0);
        ui.label(
            RichText::new(lang.t(
                "Saved on this Mac in plain text. The key is sent only to the URL above.",
                "以纯文本保存在这台 Mac 上，密钥只会发往上面填写的地址。",
            ))
            .size(10.5)
            .color(pal().muted),
        );

        if changed {
            app.save_settings();
        }
    });
}

// ---------- AI dialog ----------

fn chip(ui: &mut Ui, label: &str) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), FontId::proportional(11.0), pal().text2);
    let height = 24.0;
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 18.0, height), Sense::click());
    let p = ui.painter();
    p.rect_filled(
        rect,
        CornerRadius::same(12),
        if response.hovered() {
            pal().accent_soft
        } else {
            pal().card
        },
    );
    p.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, pal().border),
        StrokeKind::Inside,
    );
    p.galley(
        pos2(rect.left() + 9.0, rect.center().y - galley.size().y / 2.0),
        galley,
        pal().text2,
    );
    response
}

/// Stable id for the "source expanded" flag, shared with the preview hook.
pub fn source_expand_id(clip_id: u64) -> egui::Id {
    egui::Id::new("ai_source_expanded").with(clip_id)
}

fn ai_window(app: &mut ClipFlow, ctx: &egui::Context) {
    let Some(id) = app.ai.clip_id else {
        return;
    };
    let Some(clip) = app.clips.iter().find(|c| c.id == id).cloned() else {
        app.close_ai();
        return;
    };

    let mut open = true;
    let mut close = false;
    let mut run = false;
    let mut copy_result: Option<String> = None;

    let screen = ctx.content_rect();
    let window_id = egui::Id::new("ai_window");
    let max_h = (screen.height() - 48.0).max(360.0);
    // Use last frame's size of this window once the person has resized it.
    let area = ctx.memory(|memory| memory.area_rect(window_id));
    // Where the window's inside ends, so the result can fill whatever is left.
    let panel_bottom = area.map_or(screen.bottom() - 64.0, |rect| rect.bottom() - 16.0);

    let lang = app.settings.language;
    // A copied document can be very long, so the source starts collapsed and only
    // expands when the person asks for it.
    let source_expand_id = source_expand_id(id);
    let clip_len = clip.text.chars().count();
    const SOURCE_PREVIEW_CHARS: usize = 160;
    const SOURCE_EXPANDED_CHARS: usize = 6000;
    egui::Window::new(lang.t("Analyse with AI", "用 AI 分析"))
        .id(window_id)
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .vscroll(true)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
        .default_size(vec2(640.0, max_h))
        .min_size(vec2(520.0, 360.0))
        .max_size(vec2(
            (screen.width() - 48.0).max(520.0),
            max_h,
        ))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            egui::Frame::new()
                .fill(pal().panel)
                .stroke(Stroke::new(1.0, pal().border))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(margin(20, 16)),
        )
        .show(ctx, |ui| {
            ui.set_width(ui.available_width());
            let mut scroll_style = ui.spacing().scroll.clone();
            scroll_style.bar_width = 9.0;
            scroll_style.floating = false;
            scroll_style.bar_inner_margin = 6.0;
            scroll_style.bar_outer_margin = 4.0;
            scroll_style.handle_min_length = 24.0;
            // Use the text colour for the handle; the card colour would vanish on these boxes.
            scroll_style.foreground_color = true;
            ui.spacing_mut().scroll = scroll_style;

            // Measure the room the window really gives us, so the layout can adapt.
            let content_top = ui.cursor().top();
            let avail = (panel_bottom - content_top).max(180.0);
            let mut expanded =
                ctx.memory(|m| m.data.get_temp::<bool>(source_expand_id).unwrap_or(false));

            let done = matches!(app.ai.status, AiStatus::Done(_));
            let running = matches!(app.ai.status, AiStatus::Running);
            let ready = app.ai_configured();
            // The buttons and the answer stay visible; only the part above them
            // scrolls, so a short window can never hide the answer.
            //
            // egui adds the 8pt item spacing after every widget, so the block
            // below the scrolling part is taller than it looks. Reserve room for
            // all of it, plus the smallest answer we are willing to show.
            let note_h = if ready { 0.0 } else { 25.0 };
            let footer_h = if done { 32.0 } else { 0.0 };
            let reserve = 8.0 + 12.0 + 42.0 + note_h + 9.0 + 10.0 + 32.0 + 4.0 + 18.0
                + footer_h
                + 48.0
                + 8.0;
            let upper_max = (avail - reserve).clamp(48.0, f32::INFINITY);
            let instruction_h = (avail * 0.10).clamp(44.0, 76.0);
            let source_h = (avail * 0.25).clamp(64.0, 150.0);

            egui::ScrollArea::vertical()
                .id_salt(("ai_upper", app.ai.epoch))
                .min_scrolled_height(0.0)
                .max_height(upper_max)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // The clip being analysed: one quiet line by default, so a long
                    // document does not crowd out the question or the answer.
                    let kind = detect_kind(&clip.text);
                    let (kind_color, kind_label) = kind_style(kind, lang);
                    egui::Frame::new()
                        .fill(pal().input)
                        .stroke(Stroke::new(1.0, pal().border))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(margin(11, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (icon_rect, _) =
                                    ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
                                let p = ui.painter();
                                p.rect_filled(
                                    icon_rect,
                                    CornerRadius::same(7),
                                    tint(kind_color, 34),
                                );
                                paint_kind_icon(p, icon_rect.shrink(6.0), kind, kind_color);
                                ui.add_space(9.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(kind_label).size(11.0).color(kind_color),
                                    );
                                    ui.add_space(1.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {}",
                                            clip_len,
                                            lang.t("chars", "个字符")
                                        ))
                                        .size(10.5)
                                        .color(pal().muted),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let toggle = ui.link(
                                        RichText::new(if expanded {
                                            lang.t("Collapse", "收起")
                                        } else {
                                            lang.t("Show source", "展开原文")
                                        })
                                        .size(11.0)
                                        .color(pal().accent),
                                    );
                                    if toggle.clicked() {
                                        expanded = !expanded;
                                        ctx.memory_mut(|m| {
                                            m.data.insert_temp(source_expand_id, expanded);
                                        });
                                    }
                                });
                            });

                            if expanded {
                                ui.add_space(8.0);
                                // Laying out a whole book is slow; a preview is enough.
                                let shown: String =
                                    clip.text.chars().take(SOURCE_EXPANDED_CHARS).collect();
                                egui::ScrollArea::vertical()
                                    .id_salt(("ai_clip_text", app.ai.epoch))
                                    .min_scrolled_height(0.0)
                                    .max_height(source_h)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        let text = if kind == Kind::Code {
                                            RichText::new(&shown).monospace().size(12.0)
                                        } else {
                                            RichText::new(&shown).size(12.5)
                                        };
                                        ui.add(
                                            egui::Label::new(text.color(pal().text))
                                                .selectable(true)
                                                .wrap_mode(egui::TextWrapMode::Wrap),
                                        );
                                    });
                                if clip_len > SOURCE_EXPANDED_CHARS {
                                    ui.add_space(4.0);
                                    let note = if lang.is_chinese() {
                                        format!(
                                            "仅显示前 {SOURCE_EXPANDED_CHARS} 字，共 {clip_len} 字。"
                                        )
                                    } else {
                                        format!(
                                            "Showing the first {SOURCE_EXPANDED_CHARS} of {clip_len} characters."
                                        )
                                    };
                                    ui.label(
                                        RichText::new(note).size(10.0).color(pal().muted),
                                    );
                                }
                            } else {
                                ui.add_space(6.0);
                                let preview = clip_preview(&clip.text, SOURCE_PREVIEW_CHARS);
                                let clicked = ui
                                    .add(
                                        egui::Label::new(
                                            RichText::new(preview).size(12.0).color(pal().muted),
                                        )
                                        .truncate()
                                        .sense(Sense::click()),
                                    )
                                    .clicked();
                                if clicked {
                                    expanded = true;
                                    ctx.memory_mut(|m| {
                                        m.data.insert_temp(source_expand_id, expanded);
                                    });
                                }
                            }
                        });

                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(lang.t("WHAT SHOULD THE AI DO?", "想让 AI 做什么？"))
                            .size(10.0)
                            .color(pal().muted),
                    );
                    ui.add_space(5.0);
                input_frame(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(("ai_instruction_scroll", app.ai.epoch))
                        .min_scrolled_height(0.0)
                        .max_height(instruction_h)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut app.ai.instruction)
                                    .id(egui::Id::new("ai_instruction"))
                                    .frame(egui::Frame::NONE)
                                    .desired_rows(2)
                                    .desired_width(f32::INFINITY)
                                    .hint_text(
                                        RichText::new(lang.t(
                                            "Summarise this, explain it, rewrite it clearly…",
                                            "总结这段话、解释含义、改写得更清楚…",
                                        ))
                                        .size(12.5)
                                        .color(pal().muted),
                                    ),
                            );
                        });
                });

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let presets = [
                        ("Summarise", "Summarise this."),
                        ("Explain", "Explain what this means."),
                        ("Rewrite clearly", "Rewrite this clearly."),
                        ("Key points", "List the key points."),
                        ("Translate to Chinese", "翻译成简体中文。"),
                    ];
                    for (en_label, en_instruction) in presets {
                        let label = lang.t(en_label, match en_label {
                            "Summarise" => "总结",
                            "Explain" => "解释",
                            "Rewrite clearly" => "改写清楚",
                            "Key points" => "要点",
                            _ => "翻译成中文",
                        });
                        // The instruction sent to the AI stays in English, which models handle well.
                        if chip(ui, label).clicked() {
                            app.ai.instruction = en_instruction.to_string();
                        }
                    }
                });
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let label = if running {
                    lang.t("Analysing…", "分析中…")
                } else {
                    lang.t("Analyse", "分析")
                };
                let button = egui::Button::new(
                    RichText::new(label).size(12.5).color(Color32::WHITE),
                )
                .fill(if ready { pal().accent } else { pal().muted })
                .corner_radius(CornerRadius::same(9))
                .min_size(vec2(96.0, 34.0));
                if ui.add_enabled(!running && ready, button).clicked() {
                    run = true;
                }
                if running {
                    ui.add(egui::Spinner::new().size(14.0).color(pal().accent));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .link(
                            RichText::new(lang.t("Close", "关闭"))
                                .size(11.5)
                                .color(pal().muted),
                        )
                        .clicked()
                    {
                        close = true;
                    }
                });
            });
            if !ready {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(lang.t(
                        "Set the request URL and API key in Settings to use this.",
                        "请先在设置里填写请求地址和 API 密钥。",
                    ))
                    .size(11.0)
                    .color(pal().muted),
                );
            }

            // A quiet rule separates the question from the answer.
            let rule_width = ui.available_width();
            let (rule_rect, _) = ui.allocate_exact_size(vec2(rule_width, 1.0), Sense::hover());
            ui.painter()
                .rect_filled(rule_rect, CornerRadius::ZERO, pal().border);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(lang.t("RESULT", "回答"))
                        .size(10.0)
                        .color(pal().muted),
                );
                if let AiStatus::Done(answer) = &app.ai.status {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if chip(ui, lang.t("Copy result", "复制回答")).clicked() {
                            copy_result = Some(answer.clone());
                        }
                    });
                }
            });
            ui.add_space(4.0);
            // Fill the space left below the buttons, keeping room for the note.
            let result_h = (panel_bottom - ui.cursor().top() - 18.0 - footer_h).max(32.0);
            egui::Frame::new()
                .fill(pal().input)
                .stroke(Stroke::new(1.0, pal().border))
                .corner_radius(CornerRadius::same(9))
                .inner_margin(margin(12, 9))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(("ai_result_text", app.ai.epoch))
                        .min_scrolled_height(0.0)
                        .max_height(result_h)
                        .auto_shrink([false, false])
                        .show(ui, |ui| match &app.ai.status {
                            AiStatus::Idle => {
                                ui.add_space(((result_h - 16.0) * 0.5).max(0.0));
                                ui.vertical_centered(|ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(lang.t(
                                                "The answer will appear here.",
                                                "回答会显示在这里。",
                                            ))
                                            .size(12.0)
                                            .color(pal().muted),
                                        )
                                        .wrap_mode(egui::TextWrapMode::Wrap),
                                    );
                                });
                            }
                            AiStatus::Running => {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Spinner::new().size(13.0).color(pal().accent),
                                    );
                                    ui.label(
                                        RichText::new(lang.t("Asking the AI…", "正在询问 AI…"))
                                            .size(12.0)
                                            .color(pal().muted),
                                    );
                                });
                            }
                            AiStatus::Failed(message) => {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(message).size(11.5).color(pal().danger),
                                    )
                                    .selectable(true)
                                    .wrap_mode(egui::TextWrapMode::Wrap),
                                );
                            }
                            AiStatus::Done(answer) => {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(answer).size(12.5).color(pal().text),
                                    )
                                    .selectable(true)
                                    .wrap_mode(egui::TextWrapMode::Wrap),
                                );
                            }
                        });
                });
            if done {
                ui.add_space(5.0);
                ui.label(
                    RichText::new(lang.t("This result is not saved.", "这个回答不会被保存。"))
                        .size(10.5)
                        .color(pal().muted),
                );
            }
        });

    if let Some(text) = copy_result {
        app.copy_text(&text);
    }
    if run {
        app.run_ai(ctx);
    }
    if !open || close {
        app.close_ai();
    }
}

// ---------- empty + toast ----------

fn empty_state(app: &ClipFlow, ui: &mut Ui, no_search: bool) {
    let lang = app.settings.language;
    let height = ui.available_height().max(240.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let center = pos2(rect.center().x, rect.center().y - 26.0);
    let p = ui.painter();
    p.circle_filled(center, 30.0, tint(pal().accent, 22));
    paint_kind_icon(
        p,
        Rect::from_center_size(center, Vec2::splat(24.0)),
        Kind::Text,
        pal().accent,
    );
    let (title, hint) = if no_search {
        (
            lang.t("Nothing here yet", "这里还没有内容"),
            lang.t("Copy something and it will appear here.", "复制内容后就会出现在这里。"),
        )
    } else {
        (
            lang.t("No matches", "没有匹配结果"),
            lang.t("Try a shorter word, or press Esc to clear.", "换个更短的词，或按 Esc 清除。"),
        )
    };
    p.text(
        pos2(center.x, center.y + 52.0),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(15.0),
        pal().text,
    );
    p.text(
        pos2(center.x, center.y + 74.0),
        Align2::CENTER_CENTER,
        hint,
        FontId::proportional(12.0),
        pal().muted,
    );
}

fn toast(app: &ClipFlow, ctx: &egui::Context) {
    let Some((message, at)) = &app.toast else {
        return;
    };
    let age = (app.now - at).max(0.0);
    if age > 2.2 {
        return;
    }
    let alpha = if age > 1.7 {
        ((2.2 - age) / 0.5).clamp(0.0, 1.0)
    } else {
        1.0
    };
    egui::Area::new(egui::Id::new("toast"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -52.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let fill = pal().panel;
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(
                    fill.r(),
                    fill.g(),
                    fill.b(),
                    (245.0 * alpha) as u8,
                ))
                .stroke(Stroke::new(1.0, tint(pal().accent, (170.0 * alpha) as u8)))
                .corner_radius(CornerRadius::same(9))
                .inner_margin(margin(14, 8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(message).size(12.0).color(Color32::from_rgba_unmultiplied(
                            pal().text.r(),
                            pal().text.g(),
                            pal().text.b(),
                            (255.0 * alpha) as u8,
                        )),
                    );
                });
        });
}

// ---------- helpers ----------

/// The first `max` characters of `text` with runs of whitespace squeezed into one
/// space, so a long document can be previewed on a single line.
fn clip_preview(text: &str, max: usize) -> String {
    let mut out = String::new();
    let mut count = 0;
    let mut pending_space = false;
    let mut truncated = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = count > 0;
            continue;
        }
        if count >= max {
            truncated = true;
            break;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(ch);
        count += 1;
    }
    if truncated {
        out.push('…');
    }
    out
}

fn first_line(text: &str) -> String {
    let line = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    line.replace('\t', " ")
}

fn one_line(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::default();
    job.wrap = egui::text::TextWrapping {
        max_width: width.max(10.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    ui.ctx().fonts_mut(|f| f.layout_job(job))
}

fn paint_magnifier(p: &egui::Painter, center: egui::Pos2, color: Color32) {
    let stroke = Stroke::new(1.4, color);
    p.circle_stroke(pos2(center.x - 1.0, center.y - 1.0), 4.2, stroke);
    p.line_segment(
        [pos2(center.x + 2.0, center.y + 2.0), pos2(center.x + 5.5, center.y + 5.5)],
        stroke,
    );
}

fn paint_diamond(p: &egui::Painter, rect: Rect, color: Color32, filled: bool) {
    let c = rect.center();
    let points = vec![
        pos2(c.x, rect.top()),
        pos2(rect.right(), c.y),
        pos2(c.x, rect.bottom()),
        pos2(rect.left(), c.y),
    ];
    if filled {
        p.add(egui::Shape::convex_polygon(
            points,
            color,
            Stroke::NONE,
        ));
    } else {
        p.add(egui::Shape::closed_line(points, Stroke::new(1.3, color)));
    }
}

fn paint_nav_icon(p: &egui::Painter, rect: Rect, icon: NavIcon, color: Color32) {
    let stroke = Stroke::new(1.3, color);
    match icon {
        NavIcon::Stack => {
            for (i, frac) in [0.28, 0.5, 0.72].iter().enumerate() {
                let y = rect.top() + rect.height() * frac;
                let inset = if i == 1 { 0.0 } else { 1.5 };
                p.line_segment(
                    [
                        pos2(rect.left() + inset, y),
                        pos2(rect.right() - inset, y),
                    ],
                    Stroke::new(1.4, color),
                );
            }
        }
        NavIcon::Diamond => paint_diamond(p, rect.shrink(1.0), color, false),
        NavIcon::Link => {
            p.circle_stroke(pos2(rect.left() + 4.5, rect.top() + 4.5), 3.0, stroke);
            p.circle_stroke(pos2(rect.right() - 4.5, rect.bottom() - 4.5), 3.0, stroke);
            p.line_segment(
                [
                    pos2(rect.left() + 6.5, rect.top() + 6.5),
                    pos2(rect.right() - 6.5, rect.bottom() - 6.5),
                ],
                stroke,
            );
        }
        NavIcon::Code => {
            p.line_segment(
                [
                    pos2(rect.left() + 4.5, rect.top() + 3.5),
                    pos2(rect.left() + 0.5, rect.center().y),
                ],
                stroke,
            );
            p.line_segment(
                [
                    pos2(rect.left() + 0.5, rect.center().y),
                    pos2(rect.left() + 4.5, rect.bottom() - 3.5),
                ],
                stroke,
            );
            p.line_segment(
                [
                    pos2(rect.right() - 4.5, rect.top() + 3.5),
                    pos2(rect.right() - 0.5, rect.center().y),
                ],
                stroke,
            );
            p.line_segment(
                [
                    pos2(rect.right() - 0.5, rect.center().y),
                    pos2(rect.right() - 4.5, rect.bottom() - 3.5),
                ],
                stroke,
            );
        }
        NavIcon::Gear => {
            let c = rect.center();
            p.circle_stroke(c, 4.4, stroke);
            p.circle_filled(c, 1.6, color);
            for i in 0..8 {
                let angle = i as f32 / 8.0 * std::f32::consts::TAU;
                let (sin, cos) = angle.sin_cos();
                p.line_segment(
                    [
                        pos2(c.x + cos * 5.8, c.y + sin * 5.8),
                        pos2(c.x + cos * 7.0, c.y + sin * 7.0),
                    ],
                    Stroke::new(1.3, color),
                );
            }
        }
        NavIcon::Doc => {
            p.rect_stroke(
                rect.shrink2(vec2(1.5, 0.5)),
                CornerRadius::same(2),
                stroke,
                StrokeKind::Inside,
            );
            p.line_segment(
                [
                    pos2(rect.left() + 4.0, rect.top() + rect.height() * 0.5),
                    pos2(rect.right() - 4.0, rect.top() + rect.height() * 0.5),
                ],
                Stroke::new(1.2, color),
            );
            p.line_segment(
                [
                    pos2(rect.left() + 4.0, rect.top() + rect.height() * 0.72),
                    pos2(rect.right() - 4.0, rect.top() + rect.height() * 0.72),
                ],
                Stroke::new(1.2, color),
            );
        }
    }
}

fn paint_kind_icon(p: &egui::Painter, rect: Rect, kind: Kind, color: Color32) {
    let stroke = Stroke::new(1.4, color);
    match kind {
        Kind::Text => {
            p.line_segment(
                [
                    pos2(rect.center().x, rect.top()),
                    pos2(rect.center().x, rect.bottom()),
                ],
                stroke,
            );
            p.line_segment(
                [
                    pos2(rect.center().x, rect.top()),
                    pos2(rect.right(), rect.top()),
                ],
                stroke,
            );
        }
        Kind::Link => {
            p.circle_stroke(pos2(rect.left() + 4.0, rect.top() + 4.0), 2.8, stroke);
            p.circle_stroke(pos2(rect.right() - 4.0, rect.bottom() - 4.0), 2.8, stroke);
            p.line_segment(
                [
                    pos2(rect.left() + 6.0, rect.top() + 6.0),
                    pos2(rect.right() - 6.0, rect.bottom() - 6.0),
                ],
                stroke,
            );
        }
        Kind::Code => {
            let mid_y = rect.center().y;
            p.line_segment(
                [
                    pos2(rect.left() + 4.0, rect.top() + 2.0),
                    pos2(rect.left(), mid_y),
                ],
                stroke,
            );
            p.line_segment(
                [pos2(rect.left(), mid_y), pos2(rect.left() + 4.0, rect.bottom() - 2.0)],
                stroke,
            );
            p.line_segment(
                [
                    pos2(rect.right() - 4.0, rect.top() + 2.0),
                    pos2(rect.right(), mid_y),
                ],
                stroke,
            );
            p.line_segment(
                [
                    pos2(rect.right(), mid_y),
                    pos2(rect.right() - 4.0, rect.bottom() - 2.0),
                ],
                stroke,
            );
        }
        Kind::Snippet => {
            p.rect_stroke(
                rect.shrink2(vec2(1.0, 0.0)),
                CornerRadius::same(2),
                stroke,
                StrokeKind::Inside,
            );
            for (i, frac) in [0.38, 0.62].iter().enumerate() {
                let y = rect.top() + rect.height() * frac;
                p.line_segment(
                    [
                        pos2(rect.left() + 3.0, y),
                        pos2(rect.right() - 3.0 - i as f32 * 2.0, y),
                    ],
                    Stroke::new(1.2, color),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_glyphs_are_available() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        ctx.begin_pass(Default::default());
        let font = FontId::proportional(14.0);
        let found = ctx.fonts_mut(|fonts| fonts.has_glyph(&font, '把'));
        let mut output = ctx.end_pass();
        output.textures_delta.clear();
        assert!(found, "the CJK fallback font did not provide the glyph");
    }
}
