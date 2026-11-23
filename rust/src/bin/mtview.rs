//! mtview - the MTC image viewer, on egui.
//!
//! A white window, the picture at fit-to-window, mouse wheel zoom around the
//! cursor from fit to 8x, drag to pan, double-click or `1` to go back to fit,
//! Esc to close. `.mtc` files are decoded by this crate; PNG/JPEG/WebP/BMP open
//! through the same `image` crate the codec uses.
//!
//! A release build is a GUI binary: without this, double-clicking a `.mtc` opens a
//! console window behind the picture, because cargo builds binaries as console
//! programs by default.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//!
//! `MTCODEC_VIEWER_AUTOCLOSE_MS` closes the window after that many milliseconds
//! and prints `viewer ok`, which is how the build check drives it.
//! `MTCODEC_VIEWER_ZOOM` opens at an absolute zoom factor instead of fit.

use std::path::Path;
use std::time::{Duration, Instant};

use eframe::egui;
use mtc::container;
use mtc::imgio;

const BG: egui::Color32 = egui::Color32::WHITE;
const FG: egui::Color32 = egui::Color32::from_rgb(26, 26, 26);
const DIM: egui::Color32 = egui::Color32::from_rgb(107, 107, 107);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(232, 71, 107);
const BAR_BG: egui::Color32 = egui::Color32::from_rgb(247, 247, 248);
const CHIP_BG: egui::Color32 = egui::Color32::from_rgb(246, 246, 247);
const CHIP_FG: egui::Color32 = egui::Color32::from_rgb(90, 90, 95);
/// Highest zoom the wheel may reach.
const MAX_ZOOM: f32 = 8.0;

// ---------------------------------------------------------------- geometry --

/// Zoom and pan, in points, with the canvas origin at (0, 0).
struct Geometry {
    img_w: f32,
    img_h: f32,
    /// Points per picture pixel.
    scale: f32,
    /// Top-left corner of the picture on the canvas.
    pan: egui::Vec2,
    fit_mode: bool,
    canvas: egui::Vec2,
}

impl Geometry {
    fn new(img_w: usize, img_h: usize) -> Geometry {
        Geometry {
            img_w: img_w as f32,
            img_h: img_h as f32,
            scale: 1.0,
            pan: egui::Vec2::ZERO,
            fit_mode: true,
            canvas: egui::vec2(1.0, 1.0),
        }
    }

    fn fit_scale(&self) -> f32 {
        let cw = self.canvas.x.max(1.0);
        let ch = self.canvas.y.max(1.0);
        (cw / self.img_w).min(ch / self.img_h)
    }

    fn size(&self) -> egui::Vec2 {
        egui::vec2(self.img_w * self.scale, self.img_h * self.scale)
    }

    /// Keep the picture inside the canvas: centred when it is smaller, pinned to
    /// the edges when it is larger.
    fn clamp_pan(&mut self) {
        let s = self.size();
        if s.x <= self.canvas.x {
            self.pan.x = (self.canvas.x - s.x) * 0.5;
        } else {
            self.pan.x = self.pan.x.clamp(self.canvas.x - s.x, 0.0);
        }
        if s.y <= self.canvas.y {
            self.pan.y = (self.canvas.y - s.y) * 0.5;
        } else {
            self.pan.y = self.pan.y.clamp(self.canvas.y - s.y, 0.0);
        }
    }

    fn fit(&mut self) {
        self.scale = self.fit_scale();
        self.fit_mode = true;
        self.clamp_pan();
    }

    fn set_canvas(&mut self, size: egui::Vec2, initial_zoom: Option<f32>) {
        let first = self.canvas.x <= 1.0 && self.canvas.y <= 1.0;
        self.canvas = size;
        match initial_zoom {
            Some(z) if first => {
                self.scale = z.clamp(self.fit_scale(), MAX_ZOOM);
                self.fit_mode = false;
                self.clamp_pan();
            }
            _ => {
                if self.fit_mode || first {
                    self.fit();
                } else {
                    self.scale = self.scale.max(self.fit_scale());
                    self.clamp_pan();
                }
            }
        }
    }

    /// Multiply the zoom by `factor`, keeping whatever sits under `pointer`
    /// where it is. The zoom never goes below fit or above [`MAX_ZOOM`].
    fn zoom_at(&mut self, pointer: egui::Vec2, factor: f32) {
        let lo = self.fit_scale();
        let hi = MAX_ZOOM.max(lo);
        let old = if self.scale > 0.0 { self.scale } else { lo };
        let new = (old * factor).clamp(lo, hi);
        if (new - old).abs() <= 1e-6 {
            return;
        }
        self.pan = pointer - (pointer - self.pan) * (new / old);
        self.scale = new;
        self.fit_mode = false;
        self.clamp_pan();
    }

    fn drag(&mut self, delta: egui::Vec2) {
        self.pan += delta;
        self.fit_mode = false;
        self.clamp_pan();
    }

    fn actual_size(&mut self) {
        self.scale = 1.0f32.max(self.fit_scale());
        self.fit_mode = false;
        self.clamp_pan();
    }

    fn label(&self) -> String {
        let pct = (self.scale * 100.0).round() as i32;
        if self.fit_mode {
            format!("fit {}%", pct)
        } else {
            format!("{}%", pct)
        }
    }
}

// -------------------------------------------------------------------- data --

struct Shot {
    pixels: egui::ColorImage,
    width: usize,
    height: usize,
    meta: String,
    title: String,
}

/// egui ships no CJK glyphs, so a Chinese file name renders as empty boxes.
/// Borrow one of the system fonts and keep it as the fallback face.
fn install_cjk_font(ctx: &egui::Context) {
    const CANDIDATES: [&str; 4] = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\Deng.ttf",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];
    let bytes = match CANDIDATES.iter().find_map(|p| std::fs::read(p).ok()) {
        Some(b) => b,
        None => return,
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert("system-cjk".to_owned(), egui::FontData::from_owned(bytes));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("system-cjk".to_owned());
    }
    ctx.set_fonts(fonts);
}

fn fmt_bytes(n: usize) -> String {    if n >= 1024 * 1024 {
        format!("{:.2} MB", n as f64 / (1024.0 * 1024.0))
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{} B", n)
    }
}

fn load(path: &str) -> Result<Shot, String> {
    let file_bytes = std::fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
    let is_mtc = Path::new(path)
        .extension()
        .map(|e| e.eq_ignore_ascii_case("mtc"))
        .unwrap_or(false);
    let (img, meta) = if is_mtc {
        let data = std::fs::read(path).map_err(|e| format!("{}: {}", path, e))?;
        let head = container::probe(&data)?;
        let decoded = container::decode(&data)?;
        let mut bits = format!(
            "{}x{}  ·  {}  ·  {}  ·  q{}",
            head.width,
            head.height,
            head.mode_name(),
            fmt_bytes(data.len()),
            head.quality
        );
        if head.aq() {
            bits.push_str("  ·  aq");
        }
        if head.has_alpha() {
            bits.push_str("  ·  alpha");
        }
        bits.push_str(&format!("  ·  v{}", head.version));
        (decoded, bits)
    } else {
        let decoded = imgio::load_rgba(path)?;
        let meta = format!(
            "{}x{}  ·  {}",
            decoded.width,
            decoded.height,
            fmt_bytes(file_bytes)
        );
        (decoded, meta)
    };
    let rgba = img.to_rgba();
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width, img.height], &rgba);
    let title = Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    Ok(Shot {
        pixels: color,
        width: img.width,
        height: img.height,
        meta,
        title,
    })
}

// --------------------------------------------------------------------- app --

struct MtView {
    shot: Shot,
    geom: Geometry,
    texture: Option<egui::TextureHandle>,
    nearest: bool,
    initial_zoom: Option<f32>,
    autoclose: Option<Instant>,
    started: Instant,
    closed: bool,
}

impl MtView {
    fn new(cc: &eframe::CreationContext<'_>, shot: Shot) -> MtView {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        install_cjk_font(&cc.egui_ctx);
        let env_u64 = |name: &str| -> Option<u64> {
            std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok())
        };
        let autoclose = env_u64("MTCODEC_VIEWER_AUTOCLOSE_MS")
            .filter(|ms| *ms > 0)
            .map(|ms| Instant::now() + Duration::from_millis(ms));
        let initial_zoom = std::env::var("MTCODEC_VIEWER_ZOOM")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|z| *z > 0.0);
        let geom = Geometry::new(shot.width, shot.height);
        let mut view = MtView {
            shot,
            geom,
            texture: None,
            nearest: false,
            initial_zoom,
            autoclose,
            started: Instant::now(),
            closed: false,
        };
        view.upload(&cc.egui_ctx, false);
        view
    }

    fn upload(&mut self, ctx: &egui::Context, nearest: bool) {
        if self.texture.is_some() && self.nearest == nearest {
            return;
        }
        let options = if nearest {
            egui::TextureOptions::NEAREST
        } else {
            egui::TextureOptions::LINEAR
        };
        self.nearest = nearest;
        self.texture = Some(ctx.load_texture("picture", self.shot.pixels.clone(), options));
    }

    fn header(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("head")
            .frame(
                egui::Frame::none()
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(5.0, 22.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::same(0.0), ACCENT);
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(&self.shot.title)
                            .color(FG)
                            .size(15.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&self.shot.meta).color(DIM).size(12.0));
                    });
                });
            });
    }

    fn footer(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("foot")
            .frame(
                egui::Frame::none()
                    .fill(BAR_BG)
                    .inner_margin(egui::Margin::symmetric(20.0, 9.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "wheel zoom  ·  drag to pan  ·  double-click to fit  ·  1 = 100%  ·  Esc closes",
                        )
                        .color(DIM)
                        .size(12.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{}x{}",
                                self.shot.width, self.shot.height
                            ))
                            .color(DIM)
                            .size(12.0)
                            .monospace(),
                        );
                    });
                });
            });
    }

    fn zoom_chip(&self, ui: &egui::Ui, canvas: egui::Rect) {
        let label = self.geom.label();
        let galley =
            ui.painter()
                .layout_no_wrap(label, egui::FontId::monospace(12.0), CHIP_FG);
        let chip = egui::Rect::from_min_size(
            egui::pos2(
                canvas.right() - galley.size().x - 26.0,
                canvas.top() + 12.0,
            ),
            egui::vec2(galley.size().x + 14.0, galley.size().y + 8.0),
        );
        ui.painter()
            .rect_filled(chip, egui::Rounding::same(4.0), CHIP_BG);
        ui.painter()
            .galley(chip.min + egui::vec2(7.0, 4.0), galley, CHIP_FG);
    }
}

impl eframe::App for MtView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.closed {
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.closed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if let Some(deadline) = self.autoclose {
            if Instant::now() >= deadline {
                println!(
                    "viewer ok file={} ms={}",
                    self.shot.title,
                    self.started.elapsed().as_millis()
                );
                self.closed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            ctx.request_repaint_after(Duration::from_millis(40));
        }

        self.header(ctx);
        self.footer(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                if rect.size() != self.geom.canvas {
                    self.geom.set_canvas(rect.size(), self.initial_zoom);
                    self.initial_zoom = None;
                }
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

                if response.double_clicked() {
                    self.geom.fit();
                }
                if response.dragged() {
                    self.geom.drag(response.drag_delta());
                }
                if let Some(pos) = response.hover_pos() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll.abs() > 0.0 {
                        let local = pos - rect.min;
                        self.geom.zoom_at(local, 1.25f32.powf(scroll / 120.0));
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                    self.geom.actual_size();
                }
                // Above 1:1, show the real pixels instead of a blur.
                self.upload(ctx, self.geom.scale >= 1.5);

                if let Some(tex) = self.texture.as_ref() {
                    let target =
                        egui::Rect::from_min_size(rect.min + self.geom.pan, self.geom.size());
                    ui.painter().image(
                        tex.id(),
                        target,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                self.zoom_chip(ui, rect);
            });
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: mtview <file.mtc|image.png>");
        return std::process::ExitCode::from(2);
    }
    let path = args[0].clone();
    let shot = match load(&path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {}", e);
            return std::process::ExitCode::from(1);
        }
    };
    let title = shot.title.clone();
    let (w, h) = (shot.width, shot.height);
    let inner = egui::vec2(
        (w as f32 + 80.0).clamp(480.0, 1600.0),
        (h as f32 + 150.0).clamp(360.0, 1000.0),
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([inner.x, inner.y])
            .with_min_inner_size([480.0, 360.0])
            .with_title(title),
        ..Default::default()
    };
    match eframe::run_native(
        "MTC viewer",
        options,
        Box::new(move |cc| Box::new(MtView::new(cc, shot))),
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geom(w: usize, h: usize, canvas: egui::Vec2) -> Geometry {
        let mut g = Geometry::new(w, h);
        g.set_canvas(canvas, None);
        g
    }

    #[test]
    fn fit_centres_and_leaves_margins() {
        let g = geom(512, 512, egui::vec2(1024.0, 768.0));
        // Height is the tighter axis.
        assert!((g.scale - 1.5).abs() < 1e-6, "scale {}", g.scale);
        assert!((g.pan.x - 128.0).abs() < 1e-3, "pan.x {}", g.pan.x);
        assert!(g.pan.y.abs() < 1e-3);
        assert!(g.label().starts_with("fit "));
    }

    #[test]
    fn zoom_stops_at_fit_on_the_way_out() {
        let mut g = geom(512, 512, egui::vec2(1024.0, 768.0));
        let fit = g.scale;
        for _ in 0..20 {
            g.zoom_at(egui::vec2(500.0, 400.0), 0.8);
        }
        assert!((g.scale - fit).abs() < 1e-6, "scale {} fit {}", g.scale, fit);
        assert!(g.fit_mode);
    }

    #[test]
    #[test]
    fn zoom_stops_at_the_limits() {
        // Big picture: fit is below 1, so the 8x ceiling is what stops the wheel.
        let mut g = geom(4000, 3000, egui::vec2(800.0, 600.0));
        assert!((g.scale - 0.2).abs() < 1e-6);
        for _ in 0..40 {
            g.zoom_at(egui::vec2(400.0, 300.0), 1.25);
        }
        assert!((g.scale - MAX_ZOOM).abs() < 1e-6, "scale {}", g.scale);

        // Small picture: it is already larger than 8x at fit, so fit is both the
        // floor and the ceiling and the wheel has nowhere to go.
        let mut s = geom(64, 64, egui::vec2(800.0, 600.0));
        let fit = s.fit_scale();
        s.zoom_at(egui::vec2(400.0, 300.0), 40.0);
        assert!((s.scale - fit).abs() < 1e-4, "scale {} fit {}", s.scale, fit);
    }

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let mut g = geom(512, 512, egui::vec2(1024.0, 768.0));
        let cursor = egui::vec2(600.0, 300.0);
        let before = (cursor - g.pan) / g.scale;
        g.zoom_at(cursor, 1.6);
        let after = (cursor - g.pan) / g.scale;
        assert!((before - after).length() < 1e-3, "{:?} -> {:?}", before, after);
        // 1.6x of 1.5 is 2.4, still inside the limits.
        assert!((g.scale - 2.4).abs() < 1e-6);
    }

    #[test]
    fn pan_is_clamped_to_the_picture() {
        let mut g = geom(512, 512, egui::vec2(1024.0, 768.0));
        g.zoom_at(egui::vec2(512.0, 384.0), 2.0);
        g.drag(egui::vec2(5000.0, 5000.0));
        assert!(g.pan.x <= 1e-3 && g.pan.y <= 1e-3, "{:?}", g.pan);
        g.drag(egui::vec2(-9000.0, -9000.0));
        let s = g.size();
        assert!((g.pan.x - (g.canvas.x - s.x)).abs() < 1e-3);
        assert!((g.pan.y - (g.canvas.y - s.y)).abs() < 1e-3);
    }

    #[test]
    fn small_picture_is_never_larger_than_fit() {
        let mut g = geom(64, 64, egui::vec2(800.0, 600.0));
        let fit = g.fit_scale();
        g.zoom_at(egui::vec2(400.0, 300.0), 40.0);
        assert!((g.scale - fit).abs() < 1e-4);
    }

    #[test]
    fn actual_size_and_initial_zoom() {
        // 1:1 only means something when fit is smaller than one pixel per point.
        let mut g = geom(4000, 3000, egui::vec2(800.0, 600.0));
        g.actual_size();
        assert!((g.scale - 1.0).abs() < 1e-6);
        assert_eq!(g.label(), "100%");

        let mut z = Geometry::new(512, 512);
        z.set_canvas(egui::vec2(1024.0, 768.0), Some(4.0));
        assert!((z.scale - 4.0).abs() < 1e-6);
        assert_eq!(z.label(), "400%");
    }

    #[test]
    fn resize_keeps_zoom_when_panned_and_refits_when_fitted() {
        let mut g = geom(512, 512, egui::vec2(1024.0, 768.0));
        g.set_canvas(egui::vec2(2048.0, 1536.0), None);
        assert!((g.scale - 3.0).abs() < 1e-6, "fit after resize {}", g.scale);

        g.zoom_at(egui::vec2(100.0, 100.0), 2.0);
        let zoomed = g.scale;
        g.set_canvas(egui::vec2(512.0, 384.0), None);
        assert!((g.scale - zoomed).abs() < 1e-6);
    }
}
