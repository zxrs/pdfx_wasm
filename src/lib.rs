use pdfium_render::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::Blob;

#[wasm_bindgen]
pub async fn concat(files: Vec<Blob>) -> Blob {
    let pdfium = Pdfium::new(Pdfium::bind_to_system_library().unwrap());
    let mut dst_doc = pdfium.create_new_pdf().unwrap();
    for f in files {
        let src_doc = pdfium.load_pdf_from_blob(f, None).await.unwrap();
        dst_doc.pages_mut().append(&src_doc).unwrap();
    }
    dst_doc.save_to_blob().unwrap()
}

#[wasm_bindgen]
pub async fn extract(file: Blob, args: &str) -> Blob {
    let pdfium = Pdfium::new(Pdfium::bind_to_system_library().unwrap());
    let src_doc = pdfium.load_pdf_from_blob(file, None).await.unwrap();
    let pages = parse_pages(&src_doc, args);
    let mut dst_doc = pdfium.create_new_pdf().unwrap();
    for page in pages {
        for i in page.start..=page.end {
            let len = dst_doc.pages().len();
            dst_doc
                .pages_mut()
                .copy_page_from_document(&src_doc, i, len)
                .unwrap();
            if let Some(rotate) = page.rotate {
                let origin_rotate = dst_doc.pages().last().unwrap().rotation().unwrap();
                if rotate.ne(&origin_rotate) {
                    dst_doc.pages().last().unwrap().set_rotation(rotate);
                }
            }
        }
    }
    dst_doc.save_to_blob().unwrap()
}

struct Pages {
    start: u16,
    end: u16,
    rotate: Option<PdfPageRenderRotation>,
}

fn rotate(rotate: Option<&str>) -> Option<PdfPageRenderRotation> {
    match rotate {
        Some("0") => Some(PdfPageRenderRotation::None),
        Some("90") => Some(PdfPageRenderRotation::Degrees90),
        Some("180") => Some(PdfPageRenderRotation::Degrees180),
        Some("270") => Some(PdfPageRenderRotation::Degrees270),
        None => None,
        _ => None,
    }
}

fn parse_pages(doc: &PdfDocument, pages: &str) -> Vec<Pages> {
    pages
        .split(" ")
        .map(|p| {
            let mut s = p.split("-");
            if s.clone().count() > 1 {
                let start = s.next().unwrap().parse::<u16>().unwrap() - 1;
                let mut s = s.next().unwrap().split("r");
                let end = match s.next() {
                    Some("end") => doc.pages().len() - 1,
                    Some(v) => v.parse::<u16>().unwrap() - 1,
                    _ => {
                        panic!("no end page.")
                    }
                };
                let rotate = rotate(s.next());
                Pages { start, end, rotate }
            } else {
                let mut s = s.next().unwrap().split("r");
                let start = match s.next() {
                    Some("end") => doc.pages().len() - 1,
                    Some(v) => v.parse::<u16>().unwrap() - 1,
                    _ => {
                        panic!("no start page.")
                    }
                };
                let end = start;
                let rotate = rotate(s.next());
                Pages { start, end, rotate }
            }
        })
        .collect()
}
