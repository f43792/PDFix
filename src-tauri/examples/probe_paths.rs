// Probe: enumerate per-page vector path operations of a PDF using mupdf-rs,
// and (optionally) write a small synthetic PDF and probe it back.
//
// Goal — verify before committing to the port that mupdf-rs can reach
// the same primitives PyMuPDF exposes via `page.get_drawings()`, AND that we
// can construct a PDF page by emitting fill/stroke ops via DocumentWriter.
//
// Usage:
//   cargo run --example probe_paths -- <input.pdf>
//   cargo run --example probe_paths -- <input.pdf> <synthetic_out.pdf>

use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::rc::Rc;

use mupdf::device::NativeDevice;
use mupdf::path::PathWalker;
use mupdf::{
    ColorParams, Colorspace, Device, Document, DocumentWriter, Image, Matrix, Path, Rect, Shade,
    StrokeState, Text,
};

#[derive(Default, Debug)]
struct Geometry {
    moves: u64,
    lines: u64,
    curves: u64,
    rects: u64,
    closes: u64,
}

impl PathWalker for Geometry {
    fn move_to(&mut self, _x: f32, _y: f32) {
        self.moves += 1;
    }
    fn line_to(&mut self, _x: f32, _y: f32) {
        self.lines += 1;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.curves += 1;
    }
    fn close(&mut self) {
        self.closes += 1;
    }
    fn rect(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.rects += 1;
    }
}

#[derive(Default, Debug)]
struct PageStats {
    fill_paths: u64,
    stroke_paths: u64,
    clip_paths: u64,
    fills_text: u64,
    strokes_text: u64,
    fills_image: u64,
    fills_shade: u64,
    geometry: Geometry,
}

impl PageStats {
    fn merge(&mut self, o: &PageStats) {
        self.fill_paths += o.fill_paths;
        self.stroke_paths += o.stroke_paths;
        self.clip_paths += o.clip_paths;
        self.fills_text += o.fills_text;
        self.strokes_text += o.strokes_text;
        self.fills_image += o.fills_image;
        self.fills_shade += o.fills_shade;
        self.geometry.moves += o.geometry.moves;
        self.geometry.lines += o.geometry.lines;
        self.geometry.curves += o.geometry.curves;
        self.geometry.rects += o.geometry.rects;
        self.geometry.closes += o.geometry.closes;
    }
}

struct Probe {
    stats: Rc<RefCell<PageStats>>,
}

impl NativeDevice for Probe {
    fn fill_path(
        &mut self,
        path: &Path,
        _even_odd: bool,
        _cmt: Matrix,
        _cs: &Colorspace,
        _color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        let mut s = self.stats.borrow_mut();
        s.fill_paths += 1;
        let _ = path.walk(&mut s.geometry);
    }

    fn stroke_path(
        &mut self,
        path: &Path,
        _ss: &StrokeState,
        _cmt: Matrix,
        _cs: &Colorspace,
        _color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        let mut s = self.stats.borrow_mut();
        s.stroke_paths += 1;
        let _ = path.walk(&mut s.geometry);
    }

    fn clip_path(&mut self, _path: &Path, _even_odd: bool, _cmt: Matrix, _scissor: Rect) {
        self.stats.borrow_mut().clip_paths += 1;
    }

    fn fill_text(
        &mut self,
        _text: &Text,
        _cmt: Matrix,
        _cs: &Colorspace,
        _color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        self.stats.borrow_mut().fills_text += 1;
    }

    fn stroke_text(
        &mut self,
        _text: &Text,
        _ss: &StrokeState,
        _cmt: Matrix,
        _cs: &Colorspace,
        _color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        self.stats.borrow_mut().strokes_text += 1;
    }

    fn fill_image(&mut self, _img: &Image, _cmt: Matrix, _alpha: f32, _cp: ColorParams) {
        self.stats.borrow_mut().fills_image += 1;
    }

    fn fill_shade(&mut self, _shade: &Shade, _cmt: Matrix, _alpha: f32, _cp: ColorParams) {
        self.stats.borrow_mut().fills_shade += 1;
    }
}

fn print_row(label: &str, s: &PageStats) {
    println!(
        "{:<14}: fill={:>5} stroke={:>5} clip={:>4} text(f/s)={}/{} image={} shade={} | path-ops m={} l={} c={} re={} z={}",
        label,
        s.fill_paths,
        s.stroke_paths,
        s.clip_paths,
        s.fills_text,
        s.strokes_text,
        s.fills_image,
        s.fills_shade,
        s.geometry.moves,
        s.geometry.lines,
        s.geometry.curves,
        s.geometry.rects,
        s.geometry.closes,
    );
}

fn probe_pdf(path: &str, verbose: bool) -> Result<PageStats, Box<dyn Error>> {
    let doc = Document::open(path)?;
    let pages = doc.page_count()?;
    println!("Document: {} ({} pages)", path, pages);
    let mut totals = PageStats::default();
    for i in 0..pages {
        let page = doc.load_page(i)?;
        let stats = Rc::new(RefCell::new(PageStats::default()));
        let probe = Probe {
            stats: Rc::clone(&stats),
        };
        let device = Device::from_native(probe)?;
        page.run(&device, &Matrix::IDENTITY)?;
        drop(device);
        let s = stats.borrow();
        if verbose {
            print_row(&format!("page {:>3}", i + 1), &s);
        }
        totals.merge(&s);
    }
    print_row("totals", &totals);
    Ok(totals)
}

fn write_synthetic_pdf(out: &str) -> Result<(), Box<dyn Error>> {
    let mut writer = DocumentWriter::new(out, "pdf", "")?;
    let media_box = Rect {
        x0: 0.0,
        y0: 0.0,
        x1: 612.0,
        y1: 792.0,
    };
    let device = writer.begin_page(media_box)?;
    let cs = Colorspace::device_rgb();
    let stroke = StrokeState::default();

    // 1. Stroked rectangle
    {
        let mut p = Path::new()?;
        p.rect(72.0, 72.0, 200.0, 200.0)?;
        device.stroke_path(
            &p,
            &stroke,
            &Matrix::IDENTITY,
            &cs,
            &[0.0, 0.0, 0.0],
            1.0,
            ColorParams::default(),
        )?;
    }

    // 2. Filled triangle (move + 2 lines + close)
    {
        let mut p = Path::new()?;
        p.move_to(300.0, 72.0)?;
        p.line_to(400.0, 200.0)?;
        p.line_to(200.0, 200.0)?;
        p.close()?;
        device.fill_path(
            &p,
            false,
            &Matrix::IDENTITY,
            &cs,
            &[0.5, 0.0, 0.0],
            1.0,
            ColorParams::default(),
        )?;
    }

    // 3. Stroked Bezier curve
    {
        let mut p = Path::new()?;
        p.move_to(72.0, 400.0)?;
        p.curve_to(150.0, 300.0, 250.0, 500.0, 400.0, 400.0)?;
        device.stroke_path(
            &p,
            &stroke,
            &Matrix::IDENTITY,
            &cs,
            &[0.0, 0.0, 0.5],
            1.0,
            ColorParams::default(),
        )?;
    }

    writer.end_page(device)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let input = args.next().ok_or("usage: probe_paths <input.pdf> [out.pdf]")?;
    let synthetic_out = args.next();

    println!("=== Input probe ===");
    probe_pdf(&input, true)?;

    if let Some(out) = synthetic_out {
        println!("\n=== Roundtrip test ===");
        write_synthetic_pdf(&out)?;
        println!("Wrote synthetic PDF to {out}");
        println!("Expected ops: stroke=2 (rect + curve), fill=1 (triangle), path-ops m=2 l=2 c=1 re=1 z=1");
        println!("Re-probing what we just wrote:");
        let totals = probe_pdf(&out, false)?;
        let ok = totals.stroke_paths == 2
            && totals.fill_paths == 1
            && totals.geometry.moves == 2
            && totals.geometry.lines == 2
            && totals.geometry.curves == 1
            && totals.geometry.rects == 1
            && totals.geometry.closes == 1;
        println!(
            "Roundtrip: {}",
            if ok { "OK — read back matches expected" } else { "MISMATCH — see counts above" }
        );
    }

    Ok(())
}
