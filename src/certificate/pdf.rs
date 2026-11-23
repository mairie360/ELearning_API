//! A minimal PDF 1.4 writer: pages of text lines and rectangles in the two standard Helvetica
//! fonts, nothing else. No dependency: the attestations only need text placed on an A4 page.
//!
//! The strings are encoded in Windows-1252 (`/Encoding /WinAnsiEncoding`), so the French accents
//! of the attestations print with the base fonts every PDF reader embeds. A character outside
//! Windows-1252 is written as `?`.

/// Width of an A4 page, in points.
pub const A4_WIDTH: f32 = 595.0;
/// Height of an A4 page, in points.
pub const A4_HEIGHT: f32 = 842.0;

/// One of the two standard fonts of the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Regular,
    Bold,
}

impl Font {
    fn resource(self) -> &'static str {
        match self {
            Font::Regular => "/F1",
            Font::Bold => "/F2",
        }
    }
}

/// Approximate advance width of `c` in Helvetica, in thousandths of the font size (the AFM widths
/// of the common characters, 556 for the others): enough to center a line.
fn glyph_width(c: char, font: Font) -> u32 {
    let regular = match c {
        ' ' | '!' | ',' | '.' | '/' | ':' | ';' | 'I' | 'f' | 't' | '[' | ']' | '\'' => 278,
        'i' | 'j' | 'l' | 'í' | 'ì' | 'î' | 'ï' => 222,
        'r' | '(' | ')' | '-' => 333,
        'm' => 833,
        'w' | 'M' => 778,
        'W' => 944,
        'A' | 'B' | 'E' | 'K' | 'P' | 'S' | 'V' | 'X' | 'Y' | 'À' | 'Â' | 'É' | 'È' | 'Ê' => {
            667
        }
        'C' | 'D' | 'H' | 'N' | 'R' | 'U' | 'Ç' => 722,
        'G' | 'O' | 'Q' => 778,
        'F' | 'T' | 'Z' => 611,
        'J' | 'c' | 'k' | 's' | 'v' | 'x' | 'y' | 'z' | 'ç' => 500,
        'L' => 556,
        '«' | '»' => 556,
        _ => 556,
    };
    match font {
        Font::Regular => regular,
        // Helvetica-Bold is about 5 % wider on average.
        Font::Bold => regular + regular / 20,
    }
}

/// Width of `text` set in `font` at `size` points.
pub fn text_width(text: &str, font: Font, size: f32) -> f32 {
    let thousandths: u32 = text.chars().map(|c| glyph_width(c, font)).sum();
    // Lossless: a line is far below 2^24 thousandths.
    #[allow(clippy::cast_precision_loss)]
    let thousandths = thousandths as f32;
    thousandths * size / 1000.0
}

/// The Windows-1252 byte of `c`, `?` when it has none.
fn cp1252(c: char) -> u8 {
    let code = u32::from(c);
    if code < 0x80 || (0xA0..=0xFF).contains(&code) {
        return u8::try_from(code).unwrap_or(b'?');
    }
    match c {
        '€' => 0x80,
        '‚' => 0x82,
        'ƒ' => 0x83,
        '„' => 0x84,
        '…' => 0x85,
        '†' => 0x86,
        '‡' => 0x87,
        'ˆ' => 0x88,
        '‰' => 0x89,
        'Š' => 0x8A,
        '‹' => 0x8B,
        'Œ' => 0x8C,
        'Ž' => 0x8E,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '˜' => 0x98,
        '™' => 0x99,
        'š' => 0x9A,
        '›' => 0x9B,
        'œ' => 0x9C,
        'ž' => 0x9E,
        'Ÿ' => 0x9F,
        // Narrow no-break space (French typography) → no-break space.
        '\u{202F}' => 0xA0,
        _ => b'?',
    }
}

/// `text` as a PDF literal string: Windows-1252 bytes, `\`, `(` and `)` escaped, every byte outside
/// printable ASCII written as an octal escape, so the content stream stays ASCII.
pub fn literal_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('(');
    for c in text.chars() {
        let byte = cp1252(c);
        match byte {
            b'\\' | b'(' | b')' => {
                out.push('\\');
                out.push(char::from(byte));
            }
            0x20..=0x7E => out.push(char::from(byte)),
            _ => out.push_str(&format!("\\{byte:03o}")),
        }
    }
    out.push(')');
    out
}

/// Something drawn on a page.
#[derive(Debug, Clone, PartialEq)]
enum Mark {
    Text {
        x: f32,
        y: f32,
        font: Font,
        size: f32,
        text: String,
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        line_width: f32,
    },
}

/// An A4 page being laid out. The origin is the bottom-left corner, in points.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Page {
    marks: Vec<Mark>,
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes `text` with its baseline starting at (`x`, `y`).
    pub fn text(&mut self, x: f32, y: f32, font: Font, size: f32, text: &str) {
        self.marks.push(Mark::Text {
            x,
            y,
            font,
            size,
            text: text.to_string(),
        });
    }

    /// Writes `text` centered horizontally on the page, its baseline at `y`.
    pub fn centered(&mut self, y: f32, font: Font, size: f32, text: &str) {
        let x = ((A4_WIDTH - text_width(text, font, size)) / 2.0).max(36.0);
        self.text(x, y, font, size, text);
    }

    /// Strokes a rectangle whose bottom-left corner is (`x`, `y`).
    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, line_width: f32) {
        self.marks.push(Mark::Rectangle {
            x,
            y,
            width,
            height,
            line_width,
        });
    }

    /// The content stream of the page.
    fn content(&self) -> String {
        let mut out = String::new();
        for mark in &self.marks {
            match mark {
                Mark::Text {
                    x,
                    y,
                    font,
                    size,
                    text,
                } => out.push_str(&format!(
                    "BT {} {size:.1} Tf {x:.2} {y:.2} Td {} Tj ET\n",
                    font.resource(),
                    literal_string(text)
                )),
                Mark::Rectangle {
                    x,
                    y,
                    width,
                    height,
                    line_width,
                } => out.push_str(&format!(
                    "{line_width:.2} w {x:.2} {y:.2} {width:.2} {height:.2} re S\n"
                )),
            }
        }
        out
    }
}

/// Splits `text` into lines no wider than `max_width` once set in `font` at `size` (on spaces; a
/// word wider than the line stays whole).
pub fn wrap(text: &str, font: Font, size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && text_width(&candidate, font, size) > max_width {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Serializes `pages` into a PDF 1.4 document (`title` goes into the document information).
pub fn document(title: &str, pages: &[Page]) -> Vec<u8> {
    // Objects: 1 catalog, 2 page tree, 3 Helvetica, 4 Helvetica-Bold, 5 information, then a page
    // object and its content stream per page.
    let page_ids: Vec<usize> = (0..pages.len()).map(|index| 6 + 2 * index).collect();
    let mut objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            page_ids
                .iter()
                .map(|id| format!("{id} 0 R"))
                .collect::<Vec<_>>()
                .join(" "),
            pages.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
            .to_string(),
        format!(
            "<< /Title {} /Producer (Mairie 360 ELearning API) >>",
            literal_string(title)
        ),
    ];
    for (page, id) in pages.iter().zip(&page_ids) {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {A4_WIDTH:.0} {A4_HEIGHT:.0}] \
             /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
            id + 1
        ));
        let content = page.content();
        objects.push(format!(
            "<< /Length {} >>\nstream\n{content}endstream",
            content.len()
        ));
    }

    // The binary comment line tells transfer tools the file is not plain text.
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = out.len();
    let mut table = format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        table.push_str(&format!("{offset:010} 00000 n \n"));
    }
    out.extend_from_slice(table.as_bytes());
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let mut first = Page::new();
        first.rectangle(30.0, 30.0, 535.0, 782.0, 1.5);
        first.centered(700.0, Font::Bold, 24.0, "Attestation de formation");
        first.text(72.0, 600.0, Font::Regular, 12.0, "Durée : 2 h (test) \\ é");
        let mut second = Page::new();
        second.centered(700.0, Font::Regular, 12.0, "Deuxième page");
        document("Attestations", &[first, second])
    }

    /// The text between `start` and the next `end`, from `from`.
    fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
        let begin = text.find(start).unwrap() + start.len();
        let stop = text[begin..].find(end).unwrap() + begin;
        &text[begin..stop]
    }

    #[test]
    fn the_document_starts_and_ends_like_a_pdf() {
        let pdf = sample();
        assert!(pdf.starts_with(b"%PDF-1.4\n"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn every_xref_offset_points_at_its_object() {
        let pdf = sample();
        let text = String::from_utf8_lossy(&pdf).into_owned();
        // The lossy decoding only touches the 4 bytes of the binary comment, whose replacement
        // characters are longer: offsets are checked on the raw bytes.
        let startxref: usize = between(&text, "startxref\n", "\n").parse().unwrap();
        assert!(pdf[startxref..].starts_with(b"xref\n"));
        let table = String::from_utf8(pdf[startxref..].to_vec()).unwrap();
        let mut lines = table.lines().skip(1);
        let header = lines.next().unwrap();
        let count: usize = header.split(' ').nth(1).unwrap().parse().unwrap();
        assert_eq!(
            count,
            5 + 2 * 2 + 1,
            "free entry + 5 shared objects + 2 per page"
        );
        assert_eq!(lines.next().unwrap(), "0000000000 65535 f ");
        for number in 1..count {
            let entry = lines.next().unwrap();
            assert_eq!(
                entry.len(),
                19,
                "xref entries are 20 bytes with their line feed"
            );
            let offset: usize = entry[..10].parse().unwrap();
            let expected = format!("{number} 0 obj\n");
            assert!(
                pdf[offset..].starts_with(expected.as_bytes()),
                "object {number} at {offset}"
            );
        }
        assert!(table.contains(&format!("/Size {count}")));
    }

    #[test]
    fn stream_lengths_match_their_content() {
        let pdf = sample();
        let text = String::from_utf8_lossy(&pdf).into_owned();
        let mut rest = text.as_str();
        let mut streams = 0;
        while let Some(at) = rest.find("/Length ") {
            rest = &rest[at + "/Length ".len()..];
            let length: usize = rest[..rest.find(' ').unwrap()].parse().unwrap();
            let body = &rest[rest.find("stream\n").unwrap() + "stream\n".len()..];
            assert_eq!(&body[length..length + "endstream".len()], "endstream");
            streams += 1;
        }
        assert_eq!(streams, 2);
    }

    #[test]
    fn accented_text_is_encoded_in_windows_1252_and_escaped() {
        let pdf = sample();
        let text = String::from_utf8_lossy(&pdf).into_owned();
        assert!(text.contains("/Encoding /WinAnsiEncoding"));
        assert!(
            text.contains("(Dur\\351e : 2 h \\(test\\) \\\\ \\351) Tj"),
            "{text}"
        );
        assert!(text.contains("(Deuxi\\350me page) Tj"));
        assert!(text.contains("/BaseFont /Helvetica-Bold"));
        // The content streams are ASCII: no raw UTF-8 byte of the accents.
        assert!(!text.contains('é'));
    }

    #[test]
    fn windows_1252_specials_and_unknown_characters() {
        assert_eq!(
            literal_string("€ œ « » ’"),
            "(\\200 \\234 \\253 \\273 \\222)"
        );
        assert_eq!(literal_string("日本"), "(??)");
        assert_eq!(literal_string("a\nb"), "(a\\012b)");
    }

    #[test]
    fn long_text_is_wrapped_on_spaces() {
        let text =
            "Prévention des risques professionnels et premiers secours sur le lieu de travail";
        let lines = wrap(text, Font::Bold, 18.0, 300.0);
        assert!(lines.len() > 1);
        assert_eq!(lines.join(" "), text);
        for line in &lines {
            assert!(text_width(line, Font::Bold, 18.0) <= 300.0 || !line.contains(' '));
        }
        assert!(wrap("", Font::Regular, 12.0, 100.0).is_empty());
    }
}
