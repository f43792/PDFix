use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::num::NonZero;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use mupdf::device::{BlendMode, DefaultColorspaces, DeviceFlag, Metatext, NativeDevice, Structure};
use mupdf::{
    ColorParams, Colorspace, Device, Document, DocumentWriter, Function, Image, LineCap, LineJoin,
    Matrix, Path, PathWalker, Rect, Shade, StrokeState, Text,
};
use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem, SubmenuBuilder};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, Wry};

struct ThemeMenuState(Mutex<Option<MenuItem<Wry>>>);

const ABOUT_WINDOW_LABEL: &str = "about";


fn show_about_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(ABOUT_WINDOW_LABEL) {
        let _ = win.set_focus();
        return;
    }
    // 16:8.727 ≈ 1.8333 (HD aspect requested by user)
    const W: f64 = 660.0;
    const H: f64 = 360.0;
    match WebviewWindowBuilder::new(
        app,
        ABOUT_WINDOW_LABEL,
        WebviewUrl::App("about".into()),
    )
    .title("About pdfix")
    .inner_size(W, H)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .center()
    .build()
    {
        Ok(win) => {
            // Strip the inherited app menu (File/Edit/Help) from this window.
            let _ = win.remove_menu();
        }
        Err(e) => eprintln!("about window build failed: {e}"),
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(serde::Serialize)]
struct FileStat {
    size: u64,
    modified_ms: i64,
}

#[tauri::command]
fn file_stat(path: String) -> Result<FileStat, String> {
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    let modified_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Ok(FileStat {
        size: meta.len(),
        modified_ms,
    })
}

fn output_path_for(input: &str) -> String {
    let p = std::path::Path::new(input);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("pdf");
    let parent = p.parent().unwrap_or(std::path::Path::new(""));
    parent
        .join(format!("{stem}_rs.{ext}"))
        .to_string_lossy()
        .into_owned()
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RuleSpec {
    page: String,
    line_width: f32,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ConvertOptions {
    original_mode: String, // "rename" | "delete"
    suffix: String,
    random_colors: bool,
    hairline_threshold: f32,
    replace_with: f32,
    units: String,
    page_mode: String,
    #[serde(default)]
    current_page: Option<u32>,
    range_from: Option<u32>,
    range_to: Option<u32>,
    rules: Vec<RuleSpec>,
    save_log: bool,
    #[serde(default)]
    ignore_filled_shapes: bool,
}

const DEFAULTS_JSON: &str = include_str!("../defaults.json");

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Defaults {
    hairlines: HairlinesDefaults,
    page_range: PageRangeDefaults,
    rules: RulesDefaults,
    original_mode: String,
    rename_suffix: String,
    random_colors: bool,
    save_log: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct HairlinesDefaults {
    narrower_than: f32,
    replace_with: f32,
    units: String,
    include_type3: bool,
    include_patterns: bool,
    #[serde(default)]
    ignore_filled_shapes: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PageRangeDefaults {
    mode: String,
    #[serde(default)]
    current: Option<u32>,
    from: Option<u32>,
    to: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RulesDefaults {
    items: Vec<RuleItem>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RuleItem {
    page: String,
    line_width: f32,
}

/// Short git commit baked in at build time (see `build.rs`). Empty
/// string when the build happened outside a git checkout.
#[tauri::command]
fn get_git_commit() -> &'static str {
    env!("PDFIX_GIT_COMMIT")
}

#[tauri::command]
fn load_defaults() -> Result<Defaults, String> {
    serde_json::from_str(DEFAULTS_JSON)
        .map_err(|e| format!("parse defaults.json: {e}"))
}

const SETTINGS_FILENAME: &str = "settings.json";

/// Read user settings from `<dir>/settings.json`. Pure helper — no Tauri.
/// Errors on missing/corrupt file so the caller can fall back to bundled
/// defaults.
fn read_settings_at(dir: &std::path::Path) -> Result<Defaults, String> {
    let path = dir.join(SETTINGS_FILENAME);
    let bytes = std::fs::read(&path)
        .map_err(|e| format!("read settings ({}): {e}", path.display()))?;
    serde_json::from_slice::<Defaults>(&bytes)
        .map_err(|e| format!("parse settings ({}): {e}", path.display()))
}

/// Atomically write user settings to `<dir>/settings.json` via a temp
/// file + rename so a partial write can never corrupt the live file.
fn write_settings_at(dir: &std::path::Path, settings: &Defaults) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("create config dir ({}): {e}", dir.display()))?;
    let final_path = dir.join(SETTINGS_FILENAME);
    let tmp_path = dir.join(format!("{SETTINGS_FILENAME}.tmp"));
    let json = serde_json::to_vec_pretty(settings)
        .map_err(|e| format!("serialize settings: {e}"))?;
    std::fs::write(&tmp_path, &json)
        .map_err(|e| format!("write temp ({}): {e}", tmp_path.display()))?;
    if let Err(e) = std::fs::rename(&tmp_path, &final_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!(
            "rename {} -> {}: {e}",
            tmp_path.display(),
            final_path.display()
        ));
    }
    Ok(())
}

/// Delete the user settings file. Missing file is treated as success
/// (idempotent reset).
fn remove_settings_at(dir: &std::path::Path) -> Result<(), String> {
    let path = dir.join(SETTINGS_FILENAME);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove settings ({}): {e}", path.display())),
    }
}

fn settings_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("resolve config dir: {e}"))
}

#[tauri::command]
fn load_user_settings(app: AppHandle) -> Result<Defaults, String> {
    let dir = settings_dir(&app)?;
    read_settings_at(&dir)
}

#[tauri::command]
fn save_user_settings(app: AppHandle, settings: Defaults) -> Result<(), String> {
    let dir = settings_dir(&app)?;
    write_settings_at(&dir, &settings)
}

#[tauri::command]
fn reset_user_settings(app: AppHandle) -> Result<Defaults, String> {
    let dir = settings_dir(&app)?;
    remove_settings_at(&dir)?;
    load_defaults()
}

fn unit_to_points(value: f32, units: &str) -> f32 {
    match units {
        "Picas" => value * 12.0,
        "Milimiters" | "Millimeters" => value * 2.834_645_7,
        "Centimeters" => value * 28.346_457,
        "Inches" => value * 72.0,
        _ => value, // Points (and unknown → treat as points)
    }
}

fn rule_token_matches(token: &str, page: u32) -> bool {
    let s = token.trim();
    if s.is_empty() {
        return false;
    }
    let (cmp, rest): (fn(u32, u32) -> bool, &str) = if let Some(r) = s.strip_prefix(">=") {
        (|a, b| a >= b, r)
    } else if let Some(r) = s.strip_prefix("<=") {
        (|a, b| a <= b, r)
    } else if let Some(r) = s.strip_prefix('>') {
        (|a, b| a > b, r)
    } else if let Some(r) = s.strip_prefix('<') {
        (|a, b| a < b, r)
    } else if let Some(r) = s.strip_prefix('=') {
        (|a, b| a == b, r)
    } else {
        (|a, b| a == b, s)
    };
    rest.trim().parse::<u32>().map(|n| cmp(page, n)).unwrap_or(false)
}

/// Match a comma-separated rule spec like `"1, 2, 3"` or `">=4"` or
/// `"1, 3, >=10"`. Tokens are OR'd; empty tokens are ignored.
fn rule_matches(spec: &str, page: u32) -> bool {
    spec.split(',').any(|t| rule_token_matches(t, page))
}

#[derive(Default, Clone, Copy)]
struct PageCounters {
    fills: u64,
    strokes: u64,
    substituted: u64,
}

struct Forwarder {
    out: Rc<Device>,
    out_cs: Colorspace,
    cp: ColorParams,
    page_replace_pt: Option<f32>,
    threshold_pt: f32,
    random_colors: bool,
    ignore_filled_shapes: bool,
    rng: RefCell<XorShift64>,
    log: Rc<RefCell<Box<dyn Write>>>,
    counters: Rc<Cell<PageCounters>>,
    op_idx: Cell<usize>,
}

impl Forwarder {
    fn next_color(&self) -> Option<[f32; 3]> {
        if self.random_colors {
            Some(self.rng.borrow_mut().next_color())
        } else {
            None
        }
    }
    fn next_idx(&self) -> usize {
        let i = self.op_idx.get();
        self.op_idx.set(i + 1);
        i
    }
    fn bump_fill(&self) {
        let mut c = self.counters.get();
        c.fills += 1;
        self.counters.set(c);
    }
    fn bump_stroke(&self, substituted: bool) {
        let mut c = self.counters.get();
        c.strokes += 1;
        if substituted {
            c.substituted += 1;
        }
        self.counters.set(c);
    }
}

impl NativeDevice for Forwarder {
    fn fill_path(
        &mut self,
        path: &Path,
        even_odd: bool,
        cmt: Matrix,
        cs: &Colorspace,
        color: &[f32],
        alpha: f32,
        cp: ColorParams,
    ) {
        // Match python reference: preserve fill color/alpha; only strokes get recolored.
        let idx = self.next_idx();
        self.bump_fill();
        let mut log = self.log.borrow_mut();
        let _ = writeln!(
            log,
            "[{:03}] fill (even_odd={}) | color [{}]/α{:.3} (preserved)",
            idx,
            even_odd,
            fmt_color(color),
            alpha,
        );
        let _ = path.walk(PathLogger { out: &mut *log });
        drop(log);
        let _ = self
            .out
            .fill_path(path, even_odd, &cmt, cs, color, alpha, cp);
    }

    fn stroke_path(
        &mut self,
        path: &Path,
        ss: &StrokeState,
        cmt: Matrix,
        cs: &Colorspace,
        color: &[f32],
        alpha: f32,
        cp: ColorParams,
    ) {
        let idx = self.next_idx();
        let randomized = self.next_color();
        let orig_user = ss.line_width();
        let scale = ctm_scale(&cmt);
        let orig_dev = orig_user * scale;
        let filled_skip = self.ignore_filled_shapes && is_closed_fillable(path);
        let substitute = self.page_replace_pt.is_some()
            && orig_dev <= self.threshold_pt
            && !filled_skip;
        self.bump_stroke(substitute);

        let color_label = match randomized {
            Some(rc) => format!("[{}]/α1.000", fmt_color(&rc)),
            None => format!("[{}]/α{:.3} (preserved)", fmt_color(color), alpha),
        };

        if substitute {
            let new_dev = self.page_replace_pt.unwrap();
            let local_w = new_dev / scale;
            let mut log = self.log.borrow_mut();
            let _ = writeln!(
                log,
                "[{:03}] stroke SUB | width {:.4} -> {:.4} (device {:.4}pt -> {:.4}pt, ctm-scale {:.4}) | color [{}]/α{:.3} -> {}",
                idx, orig_user, local_w, orig_dev, new_dev, scale,
                fmt_color(color), alpha, color_label,
            );
            let _ = path.walk(PathLogger { out: &mut *log });
            drop(log);
            let stroke = match build_stroke(local_w) {
                Ok(s) => s,
                Err(_) => return,
            };
            match randomized {
                Some(rc) => {
                    let _ = self.out.stroke_path(
                        path, &stroke, &cmt, &self.out_cs, &rc, 1.0, self.cp,
                    );
                }
                None => {
                    let _ = self.out.stroke_path(
                        path, &stroke, &cmt, cs, color, alpha, cp,
                    );
                }
            }
        } else {
            let mut log = self.log.borrow_mut();
            let reason = if filled_skip {
                format!("filled shape (kept), width {:.4}", orig_user)
            } else {
                format!(
                    "width {:.4} (device {:.4}pt > {:.4}pt threshold)",
                    orig_user, orig_dev, self.threshold_pt
                )
            };
            let _ = writeln!(
                log,
                "[{:03}] stroke KEEP | {} | color [{}]/α{:.3} -> {}",
                idx, reason,
                fmt_color(color), alpha, color_label,
            );
            let _ = path.walk(PathLogger { out: &mut *log });
            drop(log);
            match randomized {
                Some(rc) => {
                    let _ = self.out.stroke_path(
                        path, ss, &cmt, &self.out_cs, &rc, 1.0, self.cp,
                    );
                }
                None => {
                    let _ = self.out.stroke_path(
                        path, ss, &cmt, cs, color, alpha, cp,
                    );
                }
            }
        }
    }

    fn clip_path(&mut self, path: &Path, even_odd: bool, cmt: Matrix, _scissor: Rect) {
        let _ = self.out.clip_path(path, even_odd, &cmt);
    }

    fn clip_stroke_path(
        &mut self,
        path: &Path,
        ss: &StrokeState,
        cmt: Matrix,
        _scissor: Rect,
    ) {
        let _ = self.out.clip_stroke_path(path, ss, &cmt);
    }

    fn fill_text(
        &mut self,
        text: &Text,
        cmt: Matrix,
        cs: &Colorspace,
        color: &[f32],
        alpha: f32,
        cp: ColorParams,
    ) {
        let _ = self.out.fill_text(text, &cmt, cs, color, alpha, cp);
    }

    fn stroke_text(
        &mut self,
        text: &Text,
        ss: &StrokeState,
        cmt: Matrix,
        cs: &Colorspace,
        color: &[f32],
        alpha: f32,
        cp: ColorParams,
    ) {
        let _ = self.out.stroke_text(text, ss, &cmt, cs, color, alpha, cp);
    }

    fn clip_text(&mut self, text: &Text, cmt: Matrix, _scissor: Rect) {
        let _ = self.out.clip_text(text, &cmt);
    }

    fn clip_stroke_text(
        &mut self,
        text: &Text,
        ss: &StrokeState,
        cmt: Matrix,
        _scissor: Rect,
    ) {
        let _ = self.out.clip_stroke_text(text, ss, &cmt);
    }

    fn ignore_text(&mut self, text: &Text, cmt: Matrix) {
        let _ = self.out.ignore_text(text, &cmt);
    }

    fn fill_shade(&mut self, shade: &Shade, cmt: Matrix, alpha: f32, cp: ColorParams) {
        let _ = self.out.fill_shade(shade, &cmt, alpha, cp);
    }

    fn fill_image(&mut self, img: &Image, cmt: Matrix, alpha: f32, cp: ColorParams) {
        let _ = self.out.fill_image(img, &cmt, alpha, cp);
    }

    fn fill_image_mask(
        &mut self,
        img: &Image,
        cmt: Matrix,
        cs: &Colorspace,
        color: &[f32],
        alpha: f32,
        cp: ColorParams,
    ) {
        let _ = self.out.fill_image_mask(img, &cmt, cs, color, alpha, cp);
    }

    fn clip_image_mask(&mut self, img: &Image, cmt: Matrix, _scissor: Rect) {
        let _ = self.out.clip_image_mask(img, &cmt);
    }

    fn pop_clip(&mut self) {
        let _ = self.out.pop_clip();
    }

    fn begin_mask(
        &mut self,
        area: Rect,
        luminosity: bool,
        cs: &Colorspace,
        color: &[f32],
        cp: ColorParams,
    ) {
        let _ = self.out.begin_mask(area, luminosity, cs, color, cp);
    }

    fn end_mask(&mut self, f: &Function) {
        let _ = self.out.end_mask(Some(f));
    }

    fn begin_group(
        &mut self,
        area: Rect,
        cs: &Colorspace,
        isolated: bool,
        knockout: bool,
        blendmode: BlendMode,
        alpha: f32,
    ) {
        let _ = self
            .out
            .begin_group(area, cs, isolated, knockout, blendmode, alpha);
    }

    fn end_group(&mut self) {
        let _ = self.out.end_group();
    }

    fn begin_tile(
        &mut self,
        area: Rect,
        view: Rect,
        x_step: f32,
        y_step: f32,
        ctm: Matrix,
        id: Option<NonZero<i32>>,
        doc_id: Option<NonZero<i32>>,
    ) -> Option<NonZero<i32>> {
        self.out
            .begin_tile(area, view, x_step, y_step, &ctm, id, doc_id)
            .ok()
            .flatten()
    }

    fn end_tile(&mut self) {
        let _ = self.out.end_tile();
    }

    fn begin_layer(&mut self, name: &str) {
        let _ = self.out.begin_layer(name);
    }

    fn end_layer(&mut self) {
        let _ = self.out.end_layer();
    }

    fn begin_structure(&mut self, standard: Structure, raw: &str, idx: i32) {
        let _ = self.out.begin_structure(standard, raw, idx);
    }

    fn end_structure(&mut self) {
        let _ = self.out.end_structure();
    }

    fn begin_metatext(&mut self, meta: Metatext, text: &str) {
        let _ = self.out.begin_metatext(meta, text);
    }

    fn end_metatext(&mut self) {
        let _ = self.out.end_metatext();
    }

    fn render_flags(&mut self, _set: DeviceFlag, _clear: DeviceFlag) {}

    fn set_default_colorspaces(&mut self, _default_cs: &DefaultColorspaces) {}
}

struct PathLogger<'a, W: Write> {
    out: &'a mut W,
}

impl<W: Write> PathWalker for PathLogger<'_, W> {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = writeln!(self.out, "    m  {:.2},{:.2}", x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let _ = writeln!(self.out, "    l  {:.2},{:.2}", x, y);
    }
    fn curve_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, ex: f32, ey: f32) {
        let _ = writeln!(
            self.out,
            "    c  {:.2},{:.2}  {:.2},{:.2}  {:.2},{:.2}",
            cx1, cy1, cx2, cy2, ex, ey
        );
    }
    fn close(&mut self) {
        let _ = writeln!(self.out, "    z");
    }
    fn rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let _ = writeln!(self.out, "    re {:.2},{:.2}  {:.2},{:.2}", x1, y1, x2, y2);
    }
}

fn fmt_color(c: &[f32]) -> String {
    c.iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>()
        .join(", ")
}

fn log_path_for(input_path: &str) -> PathBuf {
    let p = std::path::Path::new(input_path);
    let parent = p.parent().unwrap_or(std::path::Path::new(""));
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    parent.join(format!("{stem}.txt"))
}

// Tiny seedable PRNG (XorShift64) — reproducible random colors without a new dep.
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0xDEAD_BEEF_C0DE_BABE } else { seed })
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_f32(&mut self) -> f32 {
        // 24 bits of mantissa precision
        ((self.next_u64() >> 40) as f32) / ((1u32 << 24) as f32)
    }
    fn next_color(&mut self) -> [f32; 3] {
        [self.next_f32(), self.next_f32(), self.next_f32()]
    }
}

/// Path that PDF can fill as an enclosed region: any subpath uses `close`,
/// or the path contains a `rect` op. Open polylines/curves return false.
fn is_closed_fillable(path: &Path) -> bool {
    struct Probe {
        closed: bool,
    }
    impl PathWalker for Probe {
        fn move_to(&mut self, _: f32, _: f32) {}
        fn line_to(&mut self, _: f32, _: f32) {}
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
        fn close(&mut self) {
            self.closed = true;
        }
        fn rect(&mut self, _: f32, _: f32, _: f32, _: f32) {
            self.closed = true;
        }
    }
    let mut probe = Probe { closed: false };
    if path.walk(&mut probe).is_err() {
        return false;
    }
    probe.closed
}

fn ctm_scale(m: &Matrix) -> f32 {
    let det = (m.a * m.d - m.b * m.c).abs().sqrt();
    if det > 1e-9 { det } else { 1.0 }
}

fn build_stroke(width_userspace: f32) -> Result<StrokeState, String> {
    StrokeState::new(
        LineCap::Round,
        LineCap::Round,
        LineCap::Round,
        LineJoin::Round,
        width_userspace,
        10.0,
        0.0,
        &[],
    )
    .map_err(|e| e.to_string())
}

fn convert_one(input_path: &str, output_path: &str, opts: &ConvertOptions) -> Result<(), String> {
    let log: Box<dyn Write> = if opts.save_log {
        let log_path = log_path_for(input_path);
        let log_file = File::create(&log_path).map_err(|e| e.to_string())?;
        Box::new(BufWriter::new(log_file))
    } else {
        Box::new(io::sink())
    };
    let log = Rc::new(RefCell::new(log));

    let threshold_pt = unit_to_points(opts.hairline_threshold, &opts.units);
    let default_replace_pt = unit_to_points(opts.replace_with, &opts.units);

    {
        let mut log = log.borrow_mut();
        let _ = writeln!(log, "Conversion log");
        let _ = writeln!(log, "  input : {input_path}");
        let _ = writeln!(log, "  output: {output_path}");
        let color_mode = if opts.random_colors {
            "RANDOM (per path)"
        } else {
            "ORIGINAL (preserved)"
        };
        let _ = writeln!(
            log,
            "  hairlines : threshold={:.4} {} ({:.4}pt), replaceWith={:.4} {} ({:.4}pt)",
            opts.hairline_threshold,
            opts.units,
            threshold_pt,
            opts.replace_with,
            opts.units,
            default_replace_pt,
        );
        let _ = writeln!(
            log,
            "  page-mode : {}{}",
            opts.page_mode,
            match opts.page_mode.as_str() {
                "range" => format!(
                    " [{}..={}]",
                    opts.range_from.unwrap_or(1),
                    opts.range_to.map(|x| x.to_string()).unwrap_or_else(|| "end".into()),
                ),
                "rule" => format!(
                    " ({} rules in {})",
                    opts.rules.len(),
                    opts.units
                ),
                "current" => format!(" [page {}]", opts.current_page.unwrap_or(1)),
                _ => String::new(),
            }
        );
        let _ = writeln!(log, "  color : {}, lineCap=Round, lineJoin=Round, alpha=1.000", color_mode);
        let _ = writeln!(log, "  ignoreFilledShapes : {}", opts.ignore_filled_shapes);
        let _ = writeln!(log);
    }

    let doc = Document::open(input_path).map_err(|e| e.to_string())?;
    let pages = doc.page_count().map_err(|e| e.to_string())?;
    let cp = ColorParams::default();
    let identity = Matrix::IDENTITY;
    // compress=yes  -> deflate-compress all content streams (the main payload)
    // garbage=deduplicate -> dedupe duplicate objects + compact xref (Python's garbage=3)
    let mut writer = DocumentWriter::new(
        output_path,
        "pdf",
        "compress=yes,garbage=deduplicate",
    )
    .map_err(|e| e.to_string())?;

    let mut totals = PageCounters::default();

    // Single RNG seed for the whole document so random colors stay
    // deterministic across pages.
    let mut doc_rng = XorShift64::new(0xDEAD_BEEF_C0DE_BABE);

    for i in 0..pages {
        let page_num = (i + 1) as u32;
        let total_pages = pages as u32;

        // Resolve effective replace-width (in points) for this page, or None to skip substitution.
        let page_replace_pt: Option<f32> = match opts.page_mode.as_str() {
            "rule" => opts
                .rules
                .iter()
                .find(|r| rule_matches(&r.page, page_num))
                .map(|r| unit_to_points(r.line_width, &opts.units)),
            "range" => {
                let lo = opts.range_from.unwrap_or(1);
                let hi = opts.range_to.unwrap_or(total_pages);
                if page_num >= lo && page_num <= hi {
                    Some(default_replace_pt)
                } else {
                    None
                }
            }
            "current" => {
                if page_num == opts.current_page.unwrap_or(1) {
                    Some(default_replace_pt)
                } else {
                    None
                }
            }
            _ => Some(default_replace_pt), // "all"
        };

        let page = doc.load_page(i).map_err(|e| e.to_string())?;
        let bounds = page.bounds().map_err(|e| e.to_string())?;

        {
            let mut log = log.borrow_mut();
            let _ = writeln!(
                log,
                "=== Page {}/{} (replace={}) ===",
                page_num,
                total_pages,
                page_replace_pt
                    .map(|w| format!("{:.4}pt", w))
                    .unwrap_or_else(|| "none (passthrough)".into()),
            );
        }

        let out_dev = writer.begin_page(bounds).map_err(|e| e.to_string())?;
        let out_rc = Rc::new(out_dev);
        let counters = Rc::new(Cell::new(PageCounters::default()));

        let forwarder = Forwarder {
            out: Rc::clone(&out_rc),
            out_cs: Colorspace::device_rgb(),
            cp,
            page_replace_pt,
            threshold_pt,
            random_colors: opts.random_colors,
            ignore_filled_shapes: opts.ignore_filled_shapes,
            rng: RefCell::new(XorShift64::new(doc_rng.next_u64())),
            log: Rc::clone(&log),
            counters: Rc::clone(&counters),
            op_idx: Cell::new(0),
        };

        let device = Device::from_native(forwarder).map_err(|e| e.to_string())?;
        let run_res = page.run(&device, &identity).map_err(|e| e.to_string());
        // Drop the input device: drops Forwarder → releases its Rc clones,
        // so out_rc refcount returns to 1 and we can consume it via end_page.
        drop(device);
        run_res?;

        let out_dev = Rc::try_unwrap(out_rc)
            .map_err(|_| "internal: out device still borrowed".to_string())?;
        writer.end_page(out_dev).map_err(|e| e.to_string())?;

        let pc = counters.get();
        totals.fills += pc.fills;
        totals.strokes += pc.strokes;
        totals.substituted += pc.substituted;
    }

    {
        let mut log = log.borrow_mut();
        let _ = writeln!(log);
        let _ = writeln!(
            log,
            "Totals: pages={} strokes={} fills={} strokes-substituted={}",
            pages, totals.strokes, totals.fills, totals.substituted,
        );
        log.flush().map_err(|e| e.to_string())?;
    }

    drop(writer);
    Ok(())
}

fn rename_target(input_path: &str, suffix: &str) -> Result<PathBuf, String> {
    let p = std::path::Path::new(input_path);
    let parent = p.parent().unwrap_or(std::path::Path::new(""));
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid filename".to_string())?;
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("pdf");
    Ok(parent.join(format!("{stem}{suffix}.{ext}")))
}

fn temp_target(input_path: &str) -> Result<PathBuf, String> {
    let p = std::path::Path::new(input_path);
    let parent = p.parent().unwrap_or(std::path::Path::new(""));
    let name = p
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid filename".to_string())?;
    Ok(parent.join(format!(".{name}.converting")))
}

fn convert_with_rename(input_path: &str, opts: &ConvertOptions) -> Result<String, String> {
    let delete_mode = opts.original_mode == "delete";

    // For "rename" with empty suffix, fall back to writing alongside the input.
    if !delete_mode && opts.suffix.is_empty() {
        let out = output_path_for(input_path);
        convert_one(input_path, &out, opts)?;
        return Ok(out);
    }

    // For delete mode we still rename the original first (to a .pdfix-bak safety
    // copy) so a mid-flow failure can be rolled back; the bak is removed at the end.
    let suffix: &str = if delete_mode { ".pdfix-bak" } else { opts.suffix.as_str() };

    let renamed = rename_target(input_path, suffix)?;
    if renamed.exists() {
        return Err(format!(
            "rename target already exists: {}",
            renamed.display()
        ));
    }
    let tmp = temp_target(input_path)?;
    let tmp_str = tmp.to_str().ok_or_else(|| "invalid temp path".to_string())?;
    convert_one(input_path, tmp_str, opts)?;
    if let Err(e) = std::fs::rename(input_path, &renamed) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.to_string());
    }
    if let Err(e) = std::fs::rename(&tmp, input_path) {
        // Best-effort restore of original name.
        let _ = std::fs::rename(&renamed, input_path);
        let _ = std::fs::remove_file(&tmp);
        return Err(e.to_string());
    }
    if delete_mode {
        // Conversion is in place at input_path; the bak is no longer needed.
        let _ = std::fs::remove_file(&renamed);
    }
    Ok(input_path.to_string())
}

#[derive(serde::Serialize, Clone)]
struct ConvertProgress {
    index: usize,
    total: usize,
    path: String,
    file: String,
    phase: &'static str,
    error: Option<String>,
    output: Option<String>,
}

fn basename_of(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

#[tauri::command]
fn convert_pdfs(
    app: AppHandle,
    paths: Vec<String>,
    options: ConvertOptions,
) -> Result<Vec<String>, String> {
    let total = paths.len();
    let mut outputs: Vec<String> = Vec::with_capacity(total);

    for (index, path) in paths.iter().enumerate() {
        let file = basename_of(path);
        let _ = app.emit(
            "convert:progress",
            ConvertProgress {
                index,
                total,
                path: path.clone(),
                file: file.clone(),
                phase: "start",
                error: None,
                output: None,
            },
        );

        match convert_with_rename(path, &options) {
            Ok(out) => {
                outputs.push(out.clone());
                let _ = app.emit(
                    "convert:progress",
                    ConvertProgress {
                        index,
                        total,
                        path: path.clone(),
                        file: file.clone(),
                        phase: "done",
                        error: None,
                        output: Some(out),
                    },
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "convert:progress",
                    ConvertProgress {
                        index,
                        total,
                        path: path.clone(),
                        file: file.clone(),
                        phase: "error",
                        error: Some(e),
                        output: None,
                    },
                );
            }
        }
    }
    Ok(outputs)
}

#[tauri::command]
fn set_theme_menu_label(state: State<'_, ThemeMenuState>, label: String) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(item) = guard.as_ref() {
        item.set_text(&label).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::Destroyed = event {
                    // Main window gone — close any viewers / about so the
                    // app actually exits instead of leaving orphans alive.
                    for (label, w) in window.app_handle().webview_windows() {
                        if label != "main" {
                            let _ = w.close();
                        }
                    }
                }
            }
        })
        .setup(|app| {
            let handle = app.handle();

            let add_pdf =
                MenuItem::with_id(handle, "add_pdf", "Add PDF…", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(handle)?;
            // Custom MenuItem instead of PredefinedMenuItem::quit — the
            // predefined variant is not rendered on Linux. We dispatch the
            // exit ourselves in on_menu_event.
            let quit = MenuItem::with_id(handle, "exit", "Exit", true, None::<&str>)?;
            let file_menu = SubmenuBuilder::new(handle, "File")
                .item(&add_pdf)
                .item(&separator)
                .item(&quit)
                .build()?;

            let sort_list =
                MenuItem::with_id(handle, "sort_list", "Sort list", true, None::<&str>)?;
            let clear_list =
                MenuItem::with_id(handle, "clear_list", "Clear list", true, None::<&str>)?;
            let edit_separator = PredefinedMenuItem::separator(handle)?;
            let reset_settings = MenuItem::with_id(
                handle,
                "reset_settings",
                "Reset settings to defaults",
                true,
                None::<&str>,
            )?;
            let toggle_theme = MenuItem::with_id(
                handle,
                "toggle_theme",
                "Toggle Dark/Light Mode",
                true,
                None::<&str>,
            )?;
            let theme_separator = PredefinedMenuItem::separator(handle)?;
            let edit_menu = SubmenuBuilder::new(handle, "Edit")
                .item(&sort_list)
                .item(&clear_list)
                .item(&theme_separator)
                .item(&toggle_theme)
                .item(&edit_separator)
                .item(&reset_settings)
                .build()?;

            app.manage(ThemeMenuState(Mutex::new(Some(toggle_theme))));

            let about =
                MenuItem::with_id(handle, "about", "About pdfix…", true, None::<&str>)?;
            let help_menu = SubmenuBuilder::new(handle, "Help").item(&about).build()?;

            let menu = MenuBuilder::new(handle)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&help_menu)
                .build()?;

            // Attach menu only to the main window. Viewer/about windows
            // therefore inherit nothing and we attach their own menus
            // (or none) without fighting an inherited app-wide menu.
            if let Some(main) = app.get_webview_window("main") {
                main.set_menu(menu)?;
            } else {
                eprintln!("setup: main window not found, falling back to app menu");
                app.set_menu(menu)?;
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            // Window-scoped viewer menu: id is "close::<window-label>".
            if let Some(label) = id.strip_prefix("close::") {
                if let Some(win) = app.get_webview_window(label) {
                    let _ = win.close();
                }
                return;
            }
            match id {
                "add_pdf" => {
                    let _ = app.emit("menu:add-pdf", ());
                }
                "sort_list" => {
                    let _ = app.emit("menu:sort-list", ());
                }
                "clear_list" => {
                    let _ = app.emit("menu:clear-list", ());
                }
                "reset_settings" => {
                    let _ = app.emit("menu:reset-settings", ());
                }
                "toggle_theme" => {
                    let _ = app.emit("menu:toggle-theme", ());
                }
                "exit" => {
                    app.exit(0);
                }
                "about" => {
                    show_about_window(app);
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            file_stat,
            convert_pdfs,
            load_defaults,
            load_user_settings,
            save_user_settings,
            reset_user_settings,
            get_git_commit,
            set_theme_menu_label
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_for_replaces_extension() {
        let out = output_path_for("/tmp/foo.pdf");
        assert!(out.ends_with("foo_rs.pdf"));
    }

    #[test]
    fn output_path_for_no_extension_defaults_to_pdf() {
        let out = output_path_for("/tmp/foo");
        assert!(out.ends_with("foo_rs.pdf"));
    }

    #[test]
    fn output_path_for_no_parent() {
        let out = output_path_for("foo.pdf");
        assert_eq!(out, "foo_rs.pdf");
    }

    #[test]
    fn unit_to_points_known_values() {
        assert!((unit_to_points(1.0, "Points") - 1.0).abs() < 1e-6);
        assert!((unit_to_points(1.0, "Picas") - 12.0).abs() < 1e-6);
        assert!((unit_to_points(1.0, "Inches") - 72.0).abs() < 1e-6);
        // Both spellings map to the same factor
        let mm_typo = unit_to_points(10.0, "Milimiters");
        let mm_correct = unit_to_points(10.0, "Millimeters");
        assert!((mm_typo - mm_correct).abs() < 1e-6);
        // 10 mm ≈ 28.346 pt
        assert!((mm_typo - 28.346_457).abs() < 1e-3);
        // 1 cm ≈ 28.346 pt
        assert!((unit_to_points(1.0, "Centimeters") - 28.346_457).abs() < 1e-3);
    }

    #[test]
    fn unit_to_points_unknown_passes_through() {
        // Unknown units treated as Points (identity)
        assert_eq!(unit_to_points(7.5, "Furlongs"), 7.5);
    }

    #[test]
    fn rule_matches_basic() {
        assert!(rule_matches("3", 3));
        assert!(!rule_matches("3", 2));
        assert!(rule_matches("=3", 3));
        assert!(!rule_matches("=3", 4));
    }

    #[test]
    fn rule_matches_inequalities() {
        assert!(rule_matches(">3", 4));
        assert!(!rule_matches(">3", 3));
        assert!(rule_matches(">=3", 3));
        assert!(rule_matches(">=3", 4));
        assert!(rule_matches("<3", 2));
        assert!(!rule_matches("<3", 3));
        assert!(rule_matches("<=3", 3));
        assert!(rule_matches("<=3", 2));
    }

    #[test]
    fn rule_matches_whitespace_and_garbage() {
        assert!(rule_matches("  >= 5  ", 7));
        assert!(!rule_matches("", 1));
        assert!(!rule_matches("not-a-number", 1));
        assert!(!rule_matches(">=abc", 1));
    }

    #[test]
    fn rule_matches_comma_list() {
        // Plain list of pages
        assert!(rule_matches("1, 2, 3", 1));
        assert!(rule_matches("1, 2, 3", 2));
        assert!(rule_matches("1, 2, 3", 3));
        assert!(!rule_matches("1, 2, 3", 4));
        // Mixed list with inequalities (OR semantics)
        assert!(rule_matches("1, >=10", 1));
        assert!(rule_matches("1, >=10", 10));
        assert!(rule_matches("1, >=10", 99));
        assert!(!rule_matches("1, >=10", 5));
        // Whitespace tolerance and stray empty tokens
        assert!(rule_matches(" 1 ,  2 ", 2));
        assert!(rule_matches("1,,2", 2));
        assert!(!rule_matches(",,", 1));
    }

    #[test]
    fn ctm_scale_identity_and_uniform() {
        let id = Matrix::IDENTITY;
        assert!((ctm_scale(&id) - 1.0).abs() < 1e-6);
        let scaled = Matrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 0.0,
            f: 0.0,
        };
        assert!((ctm_scale(&scaled) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn ctm_scale_zero_falls_back_to_one() {
        let degenerate = Matrix {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 0.0,
            f: 0.0,
        };
        assert_eq!(ctm_scale(&degenerate), 1.0);
    }

    #[test]
    fn ctm_scale_rotation_preserves_unit_scale() {
        // 90° rotation: a=0 b=1 c=-1 d=0
        let rot = Matrix {
            a: 0.0,
            b: 1.0,
            c: -1.0,
            d: 0.0,
            e: 0.0,
            f: 0.0,
        };
        assert!((ctm_scale(&rot) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn basename_of_handles_separators() {
        // Forward-slash form is portable across all platforms (Windows accepts `/` too).
        assert_eq!(basename_of("/tmp/foo.pdf"), "foo.pdf");
        assert_eq!(basename_of("/a/b/c/foo.pdf"), "foo.pdf");
        assert_eq!(basename_of("foo.pdf"), "foo.pdf");

        // Native Windows backslash paths only round-trip on Windows.
        #[cfg(windows)]
        {
            assert_eq!(basename_of(r"C:\tmp\foo.pdf"), "foo.pdf");
            assert_eq!(basename_of(r"D:\dir\subdir\file.pdf"), "file.pdf");
        }
    }

    #[test]
    fn rename_target_appends_suffix() {
        let p = rename_target("/dir/foo.pdf", "_old").unwrap();
        assert!(p.ends_with("foo_old.pdf"));
        let parent = p.parent().unwrap();
        assert!(parent.ends_with("dir"));

        #[cfg(windows)]
        {
            let p = rename_target(r"C:\dir\foo.pdf", "_old").unwrap();
            assert!(p.ends_with("foo_old.pdf"));
            let parent = p.parent().unwrap();
            assert!(parent.ends_with("dir"));
        }
    }

    #[test]
    fn rename_target_no_extension_defaults_to_pdf() {
        let p = rename_target("/dir/foo", "_old").unwrap();
        assert!(p.ends_with("foo_old.pdf"));
    }

    #[test]
    fn temp_target_uses_hidden_form() {
        let p = temp_target("/dir/foo.pdf").unwrap();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(name, ".foo.pdf.converting");
    }

    #[test]
    fn log_path_for_swaps_extension_to_txt() {
        let p = log_path_for("/dir/foo.pdf");
        assert!(p.ends_with("foo.txt"));
    }

    #[test]
    fn xorshift_is_deterministic() {
        let mut a = XorShift64::new(42);
        let mut b = XorShift64::new(42);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn xorshift_zero_seed_uses_constant() {
        // seed=0 is replaced with a fixed non-zero seed inside ::new
        let mut a = XorShift64::new(0);
        let mut b = XorShift64::new(0);
        assert_eq!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn xorshift_color_in_unit_range() {
        let mut r = XorShift64::new(1);
        for _ in 0..1000 {
            let c = r.next_color();
            for v in c {
                assert!(v >= 0.0 && v < 1.0, "value out of range: {v}");
            }
        }
    }

    #[test]
    fn fmt_color_formats_components() {
        assert_eq!(fmt_color(&[]), "");
        assert_eq!(fmt_color(&[0.5]), "0.500");
        assert_eq!(fmt_color(&[0.0, 1.0, 0.25]), "0.000, 1.000, 0.250");
    }

    #[test]
    fn load_defaults_parses_bundled_json() {
        let d = load_defaults().expect("defaults.json must parse");
        // sanity-check a few fields the UI relies on
        assert!(d.hairlines.replace_with > 0.0);
        assert!(!d.hairlines.units.is_empty());
        assert!(matches!(d.original_mode.as_str(), "delete" | "rename"));
    }
}

// ---------------------------------------------------------------------------
// Integration tests — exercise the full convert pipeline against the
// committed synthetic fixture. Live in the same `#[cfg(test)]` umbrella
// so they can reach private items (`convert_one`, `convert_with_rename`,
// `ConvertOptions`, etc.). The fixture is copied into a fresh tempdir
// per test so file-state side effects don't leak.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod integration {
    use super::*;
    use mupdf::text_page::TextPageFlags;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    const FIXTURE: &[u8] = include_bytes!("../tests/assets/synthetic.pdf");

    fn stage_fixture(dir: &Path, name: &str) -> PathBuf {
        let dst = dir.join(name);
        fs::write(&dst, FIXTURE).expect("write fixture");
        dst
    }

    fn opts(original_mode: &str) -> ConvertOptions {
        ConvertOptions {
            original_mode: original_mode.to_string(),
            suffix: "_old".to_string(),
            random_colors: false,
            hairline_threshold: 0.5,
            replace_with: 0.7,
            units: "Points".to_string(),
            page_mode: "all".to_string(),
            current_page: None,
            range_from: None,
            range_to: None,
            rules: vec![],
            save_log: false,
            ignore_filled_shapes: false,
        }
    }

    fn read_text(path: &Path) -> String {
        let doc = Document::open(path.to_str().unwrap()).expect("open output pdf");
        let pages = doc.page_count().unwrap();
        let mut out = String::new();
        for i in 0..pages {
            let page = doc.load_page(i).unwrap();
            let tp = page.to_text_page(TextPageFlags::empty()).unwrap();
            for block in tp.blocks() {
                for line in block.lines() {
                    for ch in line.chars() {
                        if let Some(c) = ch.char() {
                            out.push(c);
                        }
                    }
                    out.push(' ');
                }
            }
        }
        out
    }

    #[test]
    fn convert_one_smoke_writes_two_page_output() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "in.pdf");
        let output = dir.path().join("out.pdf");
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &opts("rename"))
            .expect("convert_one");
        assert!(output.exists(), "output file must exist");
        let doc = Document::open(output.to_str().unwrap()).unwrap();
        assert_eq!(doc.page_count().unwrap(), 2);
    }

    #[test]
    fn convert_preserves_text_content() {
        // Regression test for the python-parity bug: text on the input
        // page must survive into the output (was being silently dropped
        // by the old record-then-replay device).
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "in.pdf");
        let output = dir.path().join("out.pdf");
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &opts("rename"))
            .expect("convert_one");
        let text = read_text(&output);
        assert!(
            text.contains("PDFIX Synthetic Test Page 1"),
            "page 1 title missing from output text: {text:?}"
        );
        assert!(
            text.contains("PDFIX Synthetic Test Page 2"),
            "page 2 title missing from output text: {text:?}"
        );
    }

    #[test]
    fn rename_mode_renames_original_in_place() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        convert_with_rename(input.to_str().unwrap(), &opts("rename")).expect("convert");
        // Original path now holds the converted PDF
        assert!(input.exists(), "converted output should be at original path");
        // Renamed backup exists
        let renamed = dir.path().join("doc_old.pdf");
        assert!(renamed.exists(), "renamed original should exist");
        // No leftover temp file
        let tmp = dir.path().join(".doc.pdf.converting");
        assert!(!tmp.exists(), "temp file should be cleaned up");
    }

    #[test]
    fn delete_mode_replaces_original_no_backup() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        convert_with_rename(input.to_str().unwrap(), &opts("delete")).expect("convert");
        assert!(input.exists(), "converted output should be at original path");
        // No .pdfix-bak should be left behind
        let bak = dir.path().join("doc.pdfix-bak.pdf");
        assert!(!bak.exists(), "delete mode should remove the safety backup");
        let tmp = dir.path().join(".doc.pdf.converting");
        assert!(!tmp.exists(), "temp file should be cleaned up");
    }

    #[test]
    fn rename_errors_when_target_exists() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        // Pre-create the rename target so convert can't claim it
        fs::write(dir.path().join("doc_old.pdf"), b"existing").unwrap();
        let res = convert_with_rename(input.to_str().unwrap(), &opts("rename"));
        assert!(res.is_err(), "convert should fail when rename target exists");
        // Original should remain untouched (no temp / no rename happened)
        let original = fs::read(&input).unwrap();
        assert_eq!(
            &original[..8],
            b"%PDF-1.4",
            "original should still be the input PDF"
        );
        let tmp = dir.path().join(".doc.pdf.converting");
        assert!(!tmp.exists(), "temp file should be cleaned up on error");
    }

    #[test]
    fn rollback_when_input_is_not_a_pdf() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("not-a-pdf.pdf");
        fs::write(&input, b"this is not a valid PDF").unwrap();
        let original = fs::read(&input).unwrap();
        let res = convert_with_rename(input.to_str().unwrap(), &opts("rename"));
        assert!(res.is_err(), "should error on garbage input");
        // Original must still exist with original bytes
        let after = fs::read(&input).unwrap();
        assert_eq!(after, original, "original file must be untouched after failure");
        // No leftover temp
        let tmp = dir.path().join(".not-a-pdf.pdf.converting");
        assert!(!tmp.exists());
    }

    #[test]
    fn save_log_writes_totals_when_enabled() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log_path = dir.path().join("doc.txt");
        assert!(log_path.exists(), "log file should be written");
        let log = fs::read_to_string(&log_path).unwrap();
        assert!(log.contains("Totals:"), "log should contain Totals line: {log}");
        assert!(log.contains("strokes-substituted="));
    }

    #[test]
    fn save_log_disabled_writes_no_file() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &opts("rename")).unwrap();
        let log_path = dir.path().join("doc.txt");
        assert!(!log_path.exists(), "no log file should be created when save_log is false");
    }

    #[test]
    fn hairline_substitution_recorded_in_log() {
        // With threshold = 0.5pt, the fixture's 0.05pt + 0.1pt strokes
        // must be substituted; the 1pt + 2pt strokes must pass through.
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log = fs::read_to_string(dir.path().join("doc.txt")).unwrap();

        // Page 1 has 3 strokes (0.05, 0.1, 1.0). Page 2 has 2 (0.05, 2.0).
        // Total strokes = 5, substituted = 3 (the two hairlines on page 1
        // plus the hairline on page 2).
        assert!(log.contains("Totals: pages=2 strokes=5 fills=1 strokes-substituted=3"),
            "unexpected totals line in log:\n{log}");
    }

    #[test]
    fn no_substitution_when_threshold_zero() {
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.hairline_threshold = 0.0;
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log = fs::read_to_string(dir.path().join("doc.txt")).unwrap();
        assert!(log.contains("strokes-substituted=0"),
            "no strokes should be substituted at threshold=0:\n{log}");
    }

    #[test]
    fn random_colors_are_deterministic() {
        let dir1 = TempDir::new().unwrap();
        let in1 = stage_fixture(dir1.path(), "in.pdf");
        let out1 = dir1.path().join("out.pdf");
        let mut o = opts("rename");
        o.random_colors = true;
        convert_one(in1.to_str().unwrap(), out1.to_str().unwrap(), &o).unwrap();
        let bytes1 = fs::read(&out1).unwrap();

        let dir2 = TempDir::new().unwrap();
        let in2 = stage_fixture(dir2.path(), "in.pdf");
        let out2 = dir2.path().join("out.pdf");
        convert_one(in2.to_str().unwrap(), out2.to_str().unwrap(), &o).unwrap();
        let bytes2 = fs::read(&out2).unwrap();

        assert_eq!(bytes1, bytes2, "random_colors mode must be deterministic across runs");
    }

    #[test]
    fn page_range_limits_substitution() {
        // Substitute only on page 1 (range 1..=1). Page 2's hairline must
        // therefore be passed through, leaving 2 substitutions total
        // instead of 3.
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.page_mode = "range".to_string();
        o.range_from = Some(1);
        o.range_to = Some(1);
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log = fs::read_to_string(dir.path().join("doc.txt")).unwrap();
        assert!(log.contains("strokes-substituted=2"),
            "only page 1 hairlines should substitute when range is 1..=1:\n{log}");
    }

    #[test]
    fn current_page_limits_substitution() {
        // "current" mode targets a single page. Pick page 2: only its one
        // hairline substitutes, page 1's two pass through.
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.page_mode = "current".to_string();
        o.current_page = Some(2);
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log = fs::read_to_string(dir.path().join("doc.txt")).unwrap();
        assert!(log.contains("strokes-substituted=1"),
            "only page 2's hairline should substitute when current page is 2:\n{log}");
    }

    #[test]
    fn page_rule_per_page_widths() {
        // Page rule: substitute on page 2 only (rule "2") with width 0.9pt.
        // Hairlines on page 1 should pass through.
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.page_mode = "rule".to_string();
        o.rules = vec![RuleSpec { page: "2".to_string(), line_width: 0.9 }];
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log = fs::read_to_string(dir.path().join("doc.txt")).unwrap();
        assert!(log.contains("strokes-substituted=1"),
            "only page 2's hairline should substitute under rule '2':\n{log}");
    }

    #[test]
    fn page_rule_applies_per_page_widths() {
        // Two rules with different widths per page. Both pages have at least one
        // hairline; total substitutions = 3 (2 on page 1, 1 on page 2).
        // Page 1's substitution log line must show 0.9pt; page 2's 0.7pt.
        let dir = TempDir::new().unwrap();
        let input = stage_fixture(dir.path(), "doc.pdf");
        let output = dir.path().join("out.pdf");
        let mut o = opts("rename");
        o.page_mode = "rule".to_string();
        o.rules = vec![
            RuleSpec { page: "1".to_string(), line_width: 0.9 },
            RuleSpec { page: "2".to_string(), line_width: 0.7 },
        ];
        o.save_log = true;
        convert_one(input.to_str().unwrap(), output.to_str().unwrap(), &o).unwrap();
        let log = fs::read_to_string(dir.path().join("doc.txt")).unwrap();
        assert!(log.contains("Totals: pages=2 strokes=5 fills=1 strokes-substituted=3"),
            "expected 3 substitutions across both pages:\n{log}");
        // Pages are logged in order, so split on the page-2 banner.
        let (pg1, pg2) = log.split_once("=== Page 2/2").expect("page 2 banner");
        assert!(pg1.contains("-> 0.9000pt"), "page 1 SUB should use 0.9pt:\n{pg1}");
        assert!(pg2.contains("-> 0.7000pt"), "page 2 SUB should use 0.7pt:\n{pg2}");
    }

    #[test]
    fn defaults_serde_roundtrip() {
        let d = load_defaults().unwrap();
        let json = serde_json::to_string(&d).unwrap();
        let back: Defaults = serde_json::from_str(&json).unwrap();
        // Compare via JSON to avoid having to derive PartialEq on every nested type
        let again = serde_json::to_string(&back).unwrap();
        assert_eq!(json, again);
    }

    #[test]
    fn settings_save_then_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let mut d = load_defaults().unwrap();
        // Mutate a few fields so we know we're really reading what we wrote
        d.original_mode = "rename".to_string();
        d.rename_suffix = "_backup_xyz".to_string();
        d.random_colors = !d.random_colors;
        write_settings_at(dir.path(), &d).unwrap();
        let loaded = read_settings_at(dir.path()).unwrap();
        assert_eq!(loaded.rename_suffix, "_backup_xyz");
        assert_eq!(loaded.original_mode, "rename");
        assert_eq!(loaded.random_colors, d.random_colors);
    }

    #[test]
    fn settings_load_missing_file_errors() {
        let dir = TempDir::new().unwrap();
        // dir is empty — no settings.json
        let res = read_settings_at(dir.path());
        assert!(res.is_err(), "missing file should fail loud");
    }

    #[test]
    fn settings_load_corrupt_file_errors() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(SETTINGS_FILENAME), b"not valid json {").unwrap();
        let res = read_settings_at(dir.path());
        assert!(res.is_err(), "corrupt file should fail loud");
    }

    #[test]
    fn settings_atomic_write_leaves_no_tmp() {
        let dir = TempDir::new().unwrap();
        let d = load_defaults().unwrap();
        write_settings_at(dir.path(), &d).unwrap();
        let tmp = dir.path().join(format!("{SETTINGS_FILENAME}.tmp"));
        assert!(!tmp.exists(), "atomic write should clean up its temp file");
        assert!(dir.path().join(SETTINGS_FILENAME).exists());
    }

    #[test]
    fn settings_remove_is_idempotent() {
        let dir = TempDir::new().unwrap();
        // Removing when file is absent must succeed
        remove_settings_at(dir.path()).unwrap();
        let d = load_defaults().unwrap();
        write_settings_at(dir.path(), &d).unwrap();
        // Now it exists; removing once should clear it
        remove_settings_at(dir.path()).unwrap();
        assert!(!dir.path().join(SETTINGS_FILENAME).exists());
    }

    #[test]
    fn settings_write_creates_missing_dir() {
        let parent = TempDir::new().unwrap();
        let nested = parent.path().join("not-yet-there");
        let d = load_defaults().unwrap();
        write_settings_at(&nested, &d).unwrap();
        assert!(nested.join(SETTINGS_FILENAME).exists());
    }

    #[test]
    fn unit_choice_does_not_change_outcome_at_equivalent_threshold() {
        // 0.5pt should give identical substitution counts to ~0.176mm
        // (0.5pt / 2.834645 ≈ 0.1764mm).
        let dir1 = TempDir::new().unwrap();
        let in1 = stage_fixture(dir1.path(), "in.pdf");
        let out1 = dir1.path().join("out.pdf");
        let mut a = opts("rename");
        a.units = "Points".to_string();
        a.hairline_threshold = 0.5;
        a.save_log = true;
        convert_one(in1.to_str().unwrap(), out1.to_str().unwrap(), &a).unwrap();
        let log_a = fs::read_to_string(dir1.path().join("in.txt")).unwrap();

        let dir2 = TempDir::new().unwrap();
        let in2 = stage_fixture(dir2.path(), "in.pdf");
        let out2 = dir2.path().join("out.pdf");
        let mut b = opts("rename");
        b.units = "Milimiters".to_string();
        b.hairline_threshold = 0.5 / 2.834_645_7;
        b.save_log = true;
        convert_one(in2.to_str().unwrap(), out2.to_str().unwrap(), &b).unwrap();
        let log_b = fs::read_to_string(dir2.path().join("in.txt")).unwrap();

        let count = |s: &str| {
            s.lines()
                .find(|l| l.starts_with("Totals:"))
                .map(|l| l.to_string())
                .unwrap_or_default()
        };
        assert_eq!(count(&log_a), count(&log_b), "totals should match across equivalent units");
    }
}
