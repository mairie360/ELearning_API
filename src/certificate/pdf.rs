//! A minimal PDF 1.4 writer: pages of text lines and rectangles, nothing else.
//!
//! The text is set in DejaVu Sans and DejaVu Sans Bold (`fonts/`, Bitstream Vera licence plus
//! public domain changes, see `fonts/LICENSE`), embedded in the document so names and course
//! titles keep their characters whatever their script: Latin with its extensions (`Łukasz`,
//! `Őrs`, `Şule`), Greek, Cyrillic... (MAIR-1149). Each font is written as a `Type0` font with
//! the `Identity-H` encoding over a `CIDFontType2` descendant: the text strings are glyph ids,
//! `/W` gives their widths, `/ToUnicode` maps them back to Unicode so the text can be copied
//! and searched, and `FontFile2` holds the font program, subset to the glyphs the document uses
//! (`subsetter`). A character the fonts have no glyph for (CJK, emoji) prints as the `.notdef`
//! box of the font.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use ttf_parser::{Face, GlyphId};

/// Width of an A4 page, in points.
pub const A4_WIDTH: f32 = 595.0;
/// Height of an A4 page, in points.
pub const A4_HEIGHT: f32 = 842.0;

static REGULAR_DATA: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
static BOLD_DATA: &[u8] = include_bytes!("fonts/DejaVuSans-Bold.ttf");

static REGULAR_FACE: LazyLock<Face<'static>> = LazyLock::new(|| {
    Face::parse(REGULAR_DATA, 0).expect("the embedded DejaVu Sans is a valid TrueType font")
});
static BOLD_FACE: LazyLock<Face<'static>> = LazyLock::new(|| {
    Face::parse(BOLD_DATA, 0).expect("the embedded DejaVu Sans Bold is a valid TrueType font")
});

/// One of the two fonts of the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Regular,
    Bold,
}

impl Font {
    const ALL: [Font; 2] = [Font::Regular, Font::Bold];

    fn resource(self) -> &'static str {
        match self {
            Font::Regular => "/F1",
            Font::Bold => "/F2",
        }
    }

    fn data(self) -> &'static [u8] {
        match self {
            Font::Regular => REGULAR_DATA,
            Font::Bold => BOLD_DATA,
        }
    }

    fn face(self) -> &'static Face<'static> {
        match self {
            Font::Regular => &REGULAR_FACE,
            Font::Bold => &BOLD_FACE,
        }
    }

    fn postscript_name(self) -> &'static str {
        match self {
            Font::Regular => "DejaVuSans",
            Font::Bold => "DejaVuSans-Bold",
        }
    }

    /// The glyph of `c`, `.notdef` (0) when the font has none.
    fn glyph(self, c: char) -> u16 {
        self.face().glyph_index(c).map_or(0, |glyph| glyph.0)
    }

    /// `value` in font units, in thousandths of the font size (the unit of the PDF metrics).
    fn thousandths(self, value: f32) -> f32 {
        value * 1000.0 / f32::from(self.face().units_per_em())
    }

    /// Advance width of a glyph, in thousandths of the font size.
    fn advance(self, glyph: u16) -> f32 {
        let advance = self.face().glyph_hor_advance(GlyphId(glyph)).unwrap_or(0);
        self.thousandths(f32::from(advance))
    }
}

/// Width of `text` set in `font` at `size` points.
pub fn text_width(text: &str, font: Font, size: f32) -> f32 {
    let thousandths: f32 = text.chars().map(|c| font.advance(font.glyph(c))).sum();
    thousandths * size / 1000.0
}

/// `text` as a PDF text string: UTF-16BE behind its byte order mark, written in hexadecimal so
/// the object stays ASCII (used by the document information).
pub fn text_string(text: &str) -> String {
    let mut out = String::from("<FEFF");
    for unit in text.encode_utf16() {
        out.push_str(&format!("{unit:04X}"));
    }
    out.push('>');
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

    /// The content stream of the page, the text written as the glyph ids of `subsets`.
    fn content(&self, subsets: &[EmbeddedFont; 2]) -> String {
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
                    subsets[font_index(*font)].hex_string(text)
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

fn font_index(font: Font) -> usize {
    match font {
        Font::Regular => 0,
        Font::Bold => 1,
    }
}

/// One font as embedded in a document: its program and how its glyphs are addressed.
struct EmbeddedFont {
    font: Font,
    /// Glyph id in the original font → CID written in the content streams.
    cids: BTreeMap<u16, u16>,
    /// CID → the text it stands for (the first character met for that glyph).
    unicode: BTreeMap<u16, char>,
    /// The font program: the subset, or the whole font when subsetting failed.
    program: Vec<u8>,
    /// Whether `program` is a subset (its name then carries a subset tag).
    subset: bool,
}

impl EmbeddedFont {
    /// Embeds `font` for the characters of `texts`.
    fn new<'a>(font: Font, texts: impl Iterator<Item = &'a str>) -> Self {
        let mut remapper = subsetter::GlyphRemapper::new();
        let mut used: Vec<(u16, char)> = Vec::new();
        for c in texts.flat_map(str::chars) {
            let glyph = font.glyph(c);
            remapper.remap(glyph);
            used.push((glyph, c));
        }
        let (cids, program, subset) = match subsetter::subset(font.data(), 0, &remapper) {
            Ok(program) => {
                let cids = remapper
                    .remapped_gids()
                    .filter_map(|glyph| remapper.get(glyph).map(|cid| (glyph, cid)))
                    .collect();
                (cids, program, true)
            }
            Err(error) => {
                // Never expected with the fonts shipped here: the whole font keeps the
                // attestation readable, glyph ids are then the CIDs.
                tracing::error!(?error, "subsetting the attestation font failed");
                let cids = remapper
                    .remapped_gids()
                    .map(|glyph| (glyph, glyph))
                    .collect();
                (cids, font.data().to_vec(), false)
            }
        };
        let mut embedded = Self {
            font,
            cids,
            unicode: BTreeMap::new(),
            program,
            subset,
        };
        for (glyph, c) in used {
            if glyph != 0 {
                let cid = embedded.cid(glyph);
                embedded.unicode.entry(cid).or_insert(c);
            }
        }
        embedded
    }

    fn cid(&self, glyph: u16) -> u16 {
        self.cids.get(&glyph).copied().unwrap_or(0)
    }

    /// `text` as a hexadecimal string of two-byte CIDs (`Identity-H`).
    fn hex_string(&self, text: &str) -> String {
        let mut out = String::from("<");
        for c in text.chars() {
            out.push_str(&format!("{:04X}", self.cid(self.font.glyph(c))));
        }
        out.push('>');
        out
    }

    /// `/BaseFont` name: a subset carries a six-letter tag derived from its glyphs.
    fn base_name(&self) -> String {
        if !self.subset {
            return self.font.postscript_name().to_string();
        }
        let mut hash: u32 = 2_166_136_261;
        for glyph in self.cids.keys() {
            for byte in glyph.to_be_bytes() {
                hash = (hash ^ u32::from(byte)).wrapping_mul(16_777_619);
            }
        }
        let tag: String = (0..6)
            .map(|_| {
                let letter = char::from(b'A' + u8::try_from(hash % 26).unwrap_or(0));
                hash /= 26;
                letter
            })
            .collect();
        format!("{tag}+{}", self.font.postscript_name())
    }

    /// The `/W` array: `cid [width]` for every glyph of the subset.
    fn widths(&self) -> String {
        let entries: Vec<String> = self
            .cids
            .iter()
            .map(|(glyph, cid)| format!("{cid} [{:.0}]", self.font.advance(*glyph)))
            .collect();
        format!("[{}]", entries.join(" "))
    }

    /// The `/ToUnicode` CMap: every CID back to the character it was written for.
    fn to_unicode(&self) -> String {
        let mut out = String::from(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
             /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
             /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
             1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
        );
        let entries: Vec<(&u16, &char)> = self.unicode.iter().collect();
        // A bfchar block holds at most 100 entries.
        for chunk in entries.chunks(100) {
            out.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for (cid, c) in chunk {
                let mut units = [0u16; 2];
                let hex: String = c
                    .encode_utf16(&mut units)
                    .iter()
                    .map(|unit| format!("{unit:04X}"))
                    .collect();
                out.push_str(&format!("<{cid:04X}> <{hex}>\n"));
            }
            out.push_str("endbfchar\n");
        }
        out.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
        out
    }

    /// The `FontDescriptor` dictionary, its program in object `file`.
    fn descriptor(&self, base_name: &str, file: usize) -> String {
        let face = self.font.face();
        let bbox = face.global_bounding_box();
        let scale = |value: i16| self.font.thousandths(f32::from(value));
        let ascent = scale(face.ascender());
        let cap_height = face.capital_height().map_or(ascent, scale);
        let (flags, stem) = match self.font {
            // Nonsymbolic, plus ForceBold for the bold face.
            Font::Regular => (32, 80),
            Font::Bold => (32 + 262_144, 140),
        };
        format!(
            "<< /Type /FontDescriptor /FontName /{base_name} /Flags {flags} \
             /FontBBox [{:.0} {:.0} {:.0} {:.0}] /ItalicAngle 0 /Ascent {ascent:.0} \
             /Descent {:.0} /CapHeight {cap_height:.0} /StemV {stem} /FontFile2 {file} 0 R >>",
            scale(bbox.x_min),
            scale(bbox.y_min),
            scale(bbox.x_max),
            scale(bbox.y_max),
            scale(face.descender()),
        )
    }
}

/// A stream object: `dictionary` entries then `data`, `/Length` added.
fn stream(dictionary: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< /Length {}{dictionary} >>\nstream\n", data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
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

/// Number of the objects every document starts with: catalog, page tree, the two `Type0`
/// fonts, information, then four objects per font (descendant, descriptor, program, CMap).
const SHARED_OBJECTS: usize = 5 + 4 * 2;

/// Serializes `pages` into a PDF 1.4 document (`title` goes into the document information).
pub fn document(title: &str, pages: &[Page]) -> Vec<u8> {
    let subsets = Font::ALL.map(|font| {
        EmbeddedFont::new(
            font,
            pages.iter().flat_map(|page| {
                page.marks.iter().filter_map(move |mark| match mark {
                    Mark::Text {
                        font: used, text, ..
                    } if *used == font => Some(text.as_str()),
                    _ => None,
                })
            }),
        )
    });

    // Objects: 1 catalog, 2 page tree, 3 and 4 the Type0 fonts, 5 information, 6 to 13 the
    // pieces of the two fonts, then a page object and its content stream per page.
    let page_ids: Vec<usize> = (0..pages.len())
        .map(|index| SHARED_OBJECTS + 1 + 2 * index)
        .collect();
    let mut objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            page_ids
                .iter()
                .map(|id| format!("{id} 0 R"))
                .collect::<Vec<_>>()
                .join(" "),
            pages.len()
        )
        .into_bytes(),
    ];
    let mut font_objects: Vec<Vec<u8>> = Vec::new();
    for (index, subset) in subsets.iter().enumerate() {
        // Descendant, descriptor, program and CMap of this font.
        let first = 6 + 4 * index;
        let base_name = subset.base_name();
        objects.push(
            format!(
                "<< /Type /Font /Subtype /Type0 /BaseFont /{base_name} /Encoding /Identity-H \
                 /DescendantFonts [{first} 0 R] /ToUnicode {} 0 R >>",
                first + 3
            )
            .into_bytes(),
        );
        font_objects.push(
            format!(
                "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{base_name} \
                 /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> \
                 /FontDescriptor {} 0 R /DW 1000 /W {} /CIDToGIDMap /Identity >>",
                first + 1,
                subset.widths()
            )
            .into_bytes(),
        );
        font_objects.push(subset.descriptor(&base_name, first + 2).into_bytes());
        font_objects.push(stream(
            &format!(" /Length1 {}", subset.program.len()),
            &subset.program,
        ));
        font_objects.push(stream("", subset.to_unicode().as_bytes()));
    }
    objects.push(
        format!(
            "<< /Title {} /Producer (Mairie 360 ELearning API) >>",
            text_string(title)
        )
        .into_bytes(),
    );
    objects.extend(font_objects);
    for (page, id) in pages.iter().zip(&page_ids) {
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {A4_WIDTH:.0} {A4_HEIGHT:.0}] \
                 /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                id + 1
            )
            .into_bytes(),
        );
        objects.push(stream("", page.content(&subsets).as_bytes()));
    }

    // The binary comment line tells transfer tools the file is not plain text.
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend_from_slice(object);
        out.extend_from_slice(b"\nendobj\n");
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

/// Reads back what [`document`] wrote, for the unit and endpoint tests (it panics on any other
/// PDF): the text strings are glyph ids, only the `/ToUnicode` CMaps tell what they say.
#[doc(hidden)]
pub mod read_back {
    use std::collections::BTreeMap;

    /// Position of `needle` in `haystack` from `from`.
    pub fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
        haystack[from..]
            .windows(needle.len())
            .position(|window| window == needle)
            .map(|at| at + from)
    }

    /// The bytes of object `number` (between `N 0 obj\n` and `\nendobj`).
    pub fn object(pdf: &[u8], number: usize) -> &[u8] {
        let start = format!("\n{number} 0 obj\n");
        let begin = find(pdf, start.as_bytes(), 0).expect("object present") + start.len();
        let end = find(pdf, b"\nendobj\n", begin).expect("object closed");
        &pdf[begin..end]
    }

    /// The data of the stream object `number`.
    pub fn stream_data(pdf: &[u8], number: usize) -> &[u8] {
        let object = object(pdf, number);
        let text = String::from_utf8_lossy(&object[..find(object, b"stream\n", 0).unwrap()]);
        let length: usize = after(&text, "/Length ").parse().unwrap();
        let begin = find(object, b"stream\n", 0).unwrap() + "stream\n".len();
        &object[begin..begin + length]
    }

    /// The token following `key` in `text`.
    pub fn after<'a>(text: &'a str, key: &str) -> &'a str {
        let rest = &text[text.find(key).unwrap() + key.len()..];
        rest.split(|c: char| c.is_whitespace() || c == '>' || c == '/' || c == ']')
            .next()
            .unwrap()
    }

    /// CID → text of the `/ToUnicode` CMap of the font object `font`.
    fn cmap(pdf: &[u8], font: usize) -> BTreeMap<u16, String> {
        let dictionary = String::from_utf8_lossy(object(pdf, font)).into_owned();
        let to_unicode: usize = after(&dictionary, "/ToUnicode ").parse().unwrap();
        let data = String::from_utf8(stream_data(pdf, to_unicode).to_vec()).unwrap();
        let mut map = BTreeMap::new();
        let mut inside = false;
        for line in data.lines() {
            if line.ends_with("beginbfchar") {
                inside = true;
            } else if line == "endbfchar" {
                inside = false;
            } else if inside {
                let (cid, text) = line.split_once(' ').unwrap();
                let cid = u16::from_str_radix(cid.trim_matches(['<', '>']), 16).unwrap();
                let hex = text.trim_matches(['<', '>']);
                let units: Vec<u16> = (0..hex.len())
                    .step_by(4)
                    .map(|at| u16::from_str_radix(&hex[at..at + 4], 16).unwrap())
                    .collect();
                map.insert(cid, String::from_utf16(&units).unwrap());
            }
        }
        map
    }

    /// Every text of the document, in order, decoded through the `/ToUnicode` CMaps; a CID
    /// the CMap does not map (`.notdef`) reads as U+FFFD.
    pub fn texts(pdf: &[u8]) -> Vec<String> {
        let maps = [cmap(pdf, 3), cmap(pdf, 4)];
        let catalog = String::from_utf8_lossy(object(pdf, 2)).into_owned();
        let kids = &catalog[catalog.find("/Kids [").unwrap() + "/Kids [".len()..];
        let kids: Vec<usize> = kids[..kids.find(']').unwrap()]
            .split(" 0 R")
            .filter_map(|id| id.trim().parse().ok())
            .collect();
        let mut texts = Vec::new();
        for page in kids {
            let dictionary = String::from_utf8_lossy(object(pdf, page)).into_owned();
            let contents: usize = after(&dictionary, "/Contents ").parse().unwrap();
            let content = String::from_utf8(stream_data(pdf, contents).to_vec()).unwrap();
            for line in content.lines().filter(|line| line.starts_with("BT ")) {
                let map = if line.starts_with("BT /F1 ") {
                    &maps[0]
                } else {
                    &maps[1]
                };
                let hex = &line[line.find('<').unwrap() + 1..line.find('>').unwrap()];
                texts.push(
                    (0..hex.len())
                        .step_by(4)
                        .map(|at| {
                            let cid = u16::from_str_radix(&hex[at..at + 4], 16).unwrap();
                            map.get(&cid).cloned().unwrap_or_else(|| "\u{FFFD}".into())
                        })
                        .collect(),
                );
            }
        }
        texts
    }
}

#[cfg(test)]
mod tests {
    use super::read_back::{after, find, object, stream_data, texts};
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

    #[test]
    fn the_document_starts_and_ends_like_a_pdf() {
        let pdf = sample();
        assert!(pdf.starts_with(b"%PDF-1.4\n"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn every_xref_offset_points_at_its_object() {
        let pdf = sample();
        let tail = String::from_utf8(pdf[pdf.len() - 64..].to_vec()).unwrap();
        let startxref: usize = after(&tail, "startxref\n").parse().unwrap();
        assert!(pdf[startxref..].starts_with(b"xref\n"));
        let table = String::from_utf8(pdf[startxref..].to_vec()).unwrap();
        let mut lines = table.lines().skip(1);
        let header = lines.next().unwrap();
        let count: usize = header.split(' ').nth(1).unwrap().parse().unwrap();
        assert_eq!(
            count,
            SHARED_OBJECTS + 2 * 2 + 1,
            "free entry + 13 shared objects + 2 per page"
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
        let mut from = 0;
        let mut streams = 0;
        while let Some(at) = find(&pdf, b"/Length ", from) {
            let start = at + b"/Length ".len();
            let digits = pdf[start..]
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count();
            let length: usize = std::str::from_utf8(&pdf[start..start + digits])
                .unwrap()
                .parse()
                .unwrap();
            let body = find(&pdf, b"stream\n", start).unwrap() + b"stream\n".len();
            assert!(pdf[body + length..].starts_with(b"\nendstream"));
            streams += 1;
            from = body + length;
        }
        // Two pages, and the program and CMap of each font.
        assert_eq!(streams, 2 + 2 * 2);
    }

    #[test]
    fn the_fonts_are_embedded_subset_with_widths_and_a_cmap() {
        let pdf = sample();
        for (font, name) in [(3, "DejaVuSans"), (4, "DejaVuSans-Bold")] {
            let type0 = String::from_utf8_lossy(object(&pdf, font)).into_owned();
            assert!(type0.contains("/Subtype /Type0"), "{type0}");
            assert!(type0.contains("/Encoding /Identity-H"));
            let base = after(&type0, "/BaseFont /");
            assert_eq!(base.len(), 7 + name.len(), "{base}");
            assert!(base.ends_with(&format!("+{name}")));
            let descendant: usize = after(&type0, "/DescendantFonts [").parse().unwrap();
            let cid_font = String::from_utf8_lossy(object(&pdf, descendant)).into_owned();
            assert!(cid_font.contains("/Subtype /CIDFontType2"));
            assert!(cid_font.contains("/CIDToGIDMap /Identity"));
            assert!(cid_font.contains("/W [0 ["), "{cid_font}");
            let descriptor: usize = after(&cid_font, "/FontDescriptor ").parse().unwrap();
            let descriptor = String::from_utf8_lossy(object(&pdf, descriptor)).into_owned();
            let file: usize = after(&descriptor, "/FontFile2 ").parse().unwrap();
            let program = stream_data(&pdf, file);
            // A TrueType program, far smaller than the 700 KB of the whole font.
            assert!(program.starts_with(&[0, 1, 0, 0]));
            assert!(program.len() < 60_000, "{}", program.len());
        }
        // The document information is a UTF-16 text string.
        let info = String::from_utf8_lossy(object(&pdf, 5)).into_owned();
        assert!(info.contains(&text_string("Attestations")));
    }

    #[test]
    fn text_reads_back_through_the_to_unicode_cmap() {
        assert_eq!(
            texts(&sample()),
            [
                "Attestation de formation",
                "Durée : 2 h (test) \\ é",
                "Deuxième page"
            ]
        );
    }

    #[test]
    fn names_in_other_scripts_keep_their_characters() {
        let samples = [
            "Łukasz Kowalski",
            "Őrs Şule Đorđe Ŀ ǅ",
            "Νίκος Παπαδόπουλος",
            "Анна Иванова",
            "€ œ « » ’ Œuvre",
        ];
        let mut page = Page::new();
        for (index, text) in samples.iter().enumerate() {
            let font = if index % 2 == 0 {
                Font::Bold
            } else {
                Font::Regular
            };
            #[allow(clippy::cast_precision_loss)]
            page.centered(700.0 - 30.0 * index as f32, font, 14.0, text);
        }
        let pdf = document("Λ Ł Ж", &[page]);
        assert_eq!(texts(&pdf), samples);
        // Every character has its own glyph: none fell back to `.notdef`.
        for c in samples.concat().chars() {
            assert_ne!(Font::Regular.glyph(c), 0, "{c}");
            assert_ne!(Font::Bold.glyph(c), 0, "{c}");
        }
    }

    #[test]
    fn a_character_without_glyph_prints_notdef_without_panicking() {
        let mut page = Page::new();
        page.centered(700.0, Font::Regular, 12.0, "日本 a\nb 😀");
        page.centered(650.0, Font::Bold, 12.0, "");
        let pdf = document("日本", &[page]);
        assert!(pdf.ends_with(b"%%EOF\n"));
        // DejaVu draws this emoji: outside the BMP, its CMap entry is a surrogate pair.
        assert_eq!(texts(&pdf), ["\u{FFFD}\u{FFFD} a\u{FFFD}b 😀", ""]);
        // `.notdef` is CID 0.
        let content = String::from_utf8_lossy(stream_data(&pdf, SHARED_OBJECTS + 2)).into_owned();
        assert!(content.contains("<00000000"), "{content}");
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

    #[test]
    fn widths_follow_the_font_metrics() {
        // DejaVu Sans: `i` is narrower than `m`, the bold face is wider.
        assert!(text_width("i", Font::Regular, 10.0) < text_width("m", Font::Regular, 10.0));
        assert!(text_width("Mairie", Font::Bold, 10.0) > text_width("Mairie", Font::Regular, 10.0));
        assert!((text_width("", Font::Regular, 10.0)).abs() < f32::EPSILON);
    }
}
