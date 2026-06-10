//! Quick smoke check that the generated fixture opens and has the
//! expected page count + extractable text.

use mupdf::{text_page::TextPageFlags, Document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = format!(
        "{}/tests/assets/synthetic.pdf",
        env!("CARGO_MANIFEST_DIR")
    );
    let doc = Document::open(&path)?;
    let pages = doc.page_count()?;
    println!("opened {} ({} pages)", path, pages);
    for i in 0..pages {
        let page = doc.load_page(i)?;
        let tp = page.to_text_page(TextPageFlags::empty())?;
        let mut text = String::new();
        for block in tp.blocks() {
            for line in block.lines() {
                for ch in line.chars() {
                    if let Some(c) = ch.char() {
                        text.push(c);
                    }
                }
                text.push(' ');
            }
        }
        println!("  page {}: {:?}", i + 1, text.trim());
    }
    Ok(())
}
