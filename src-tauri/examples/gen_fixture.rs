//! Synthetic test fixture generator.
//!
//! Writes `src-tauri/tests/assets/synthetic.pdf` — a tiny multi-page PDF
//! with text + fills + a mix of hairline and normal strokes, designed
//! to exercise every code path in the converter.
//!
//! Run with:
//!     cargo run --manifest-path src-tauri/Cargo.toml --example gen_fixture
//!
//! Commit the resulting PDF. Regenerate only when the test suite needs
//! different content.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    // Output path is `<crate-root>/tests/assets/synthetic.pdf`. The
    // example is run via `cargo run`, so CARGO_MANIFEST_DIR points at
    // `src-tauri/`.
    let mut out = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    out.push("tests");
    out.push("assets");
    fs::create_dir_all(&out)?;
    out.push("synthetic.pdf");

    let bytes = build_pdf();
    fs::write(&out, &bytes)?;
    println!("wrote {} ({} bytes)", out.display(), bytes.len());
    Ok(())
}

fn build_pdf() -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = vec![0]; // offsets[0] is unused (object 0 is the free entry)

    // Header
    buf.extend_from_slice(b"%PDF-1.4\n");
    // Binary marker so PDF readers treat it as binary
    buf.extend_from_slice(&[b'%', 0xE2, 0xE3, 0xCF, 0xD3, b'\n']);

    // Helper: write an object and record its offset.
    let write_obj = |buf: &mut Vec<u8>, offsets: &mut Vec<usize>, id: u32, body: &str| {
        offsets.push(buf.len());
        write!(buf, "{} 0 obj\n{}\nendobj\n", id, body).unwrap();
    };

    // 1: Catalog
    write_obj(&mut buf, &mut offsets, 1, "<< /Type /Catalog /Pages 2 0 R >>");

    // 2: Pages dict
    write_obj(
        &mut buf,
        &mut offsets,
        2,
        "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>",
    );

    // 3: Page 1
    write_obj(
        &mut buf,
        &mut offsets,
        3,
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
         /Resources << /Font << /F1 7 0 R >> >> /Contents 4 0 R >>",
    );

    // 4: Page 1 content stream
    let stream1 = page_content(1);
    let body4 = format!(
        "<< /Length {} >>\nstream\n{}endstream",
        stream1.len(),
        stream1
    );
    write_obj(&mut buf, &mut offsets, 4, &body4);

    // 5: Page 2
    write_obj(
        &mut buf,
        &mut offsets,
        5,
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
         /Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>",
    );

    // 6: Page 2 content stream
    let stream2 = page_content(2);
    let body6 = format!(
        "<< /Length {} >>\nstream\n{}endstream",
        stream2.len(),
        stream2
    );
    write_obj(&mut buf, &mut offsets, 6, &body6);

    // 7: Helvetica
    write_obj(
        &mut buf,
        &mut offsets,
        7,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
    );

    // Cross-reference
    let xref_pos = buf.len();
    let n = (offsets.len()) as u32; // includes object 0 placeholder + 7 real objects
    write!(buf, "xref\n0 {}\n", n).unwrap();
    // object 0 entry — required, free with generation 65535
    buf.extend_from_slice(b"0000000000 65535 f \n");
    for off in offsets.iter().skip(1) {
        // 10-digit offset, generation 0, in-use
        write!(buf, "{:010} 00000 n \n", off).unwrap();
    }

    // Trailer
    write!(
        buf,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        n, xref_pos
    )
    .unwrap();

    buf
}

/// Per-page content stream. Page 1 has the full mix; page 2 has a couple
/// of strokes used to validate per-page rule logic.
fn page_content(page: u32) -> String {
    let title = format!("PDFIX Synthetic Test Page {}", page);
    let mut s = String::new();

    // Text — drawn with Helvetica at the top of the page
    s.push_str(&format!(
        "BT\n/F1 24 Tf\n72 720 Td\n({}) Tj\nET\n",
        title
    ));

    if page == 1 {
        // Three strokes of varying widths in black
        s.push_str("0 0 0 RG\n");
        // Hairline 0.05pt — will be substituted whenever threshold > 0.05
        s.push_str("0.05 w\n72 600 m\n540 600 l\nS\n");
        // 0.1pt — substituted when threshold >= 0.1
        s.push_str("0.1 w\n72 580 m\n540 580 l\nS\n");
        // 1.0pt — should always pass through unchanged
        s.push_str("1 w\n72 540 m\n540 540 l\nS\n");

        // Filled red rectangle — fill should be preserved (not recolored
        // even with random_colors mode, per python-parity behaviour)
        s.push_str("0.8 0.2 0.2 rg\n72 400 200 100 re\nf\n");
    } else {
        // Page 2: one hairline + one thicker stroke, useful for per-page
        // rule tests where each page gets a different replacement width.
        s.push_str("0 0 0 RG\n");
        s.push_str("0.05 w\n72 600 m\n540 600 l\nS\n");
        s.push_str("2 w\n72 540 m\n540 540 l\nS\n");
    }

    s
}
