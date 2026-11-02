//! Bounded, asynchronous previews for local media and document files.
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};

use anyhow::{bail, Context, Result};
use calamine::Reader;
use eframe::egui;

const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_ROWS: usize = 20_000;
const MAX_COLS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    Table,
    Pdf,
    Document,
    Video,
    External,
}

pub fn kind(path: &Path) -> Option<Kind> {
    Some(
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "svg" => {
                Kind::Image
            }
            "csv" | "tsv" | "xlsx" | "xls" | "xlsb" | "ods" => Kind::Table,
            "pdf" => Kind::Pdf,
            "docx" => Kind::Document,
            "mp4" | "webm" | "mkv" | "mov" | "avi" | "m4v" => Kind::Video,
            "doc" | "odt" | "rtf" | "ppt" | "pptx" | "odp" | "mp3" | "wav" | "ogg" | "flac"
            | "m4a" => Kind::External,
            _ => return None,
        },
    )
}

pub fn has_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("csv" | "tsv" | "svg")
    )
}

pub fn check(path: &Path) -> Result<String> {
    Ok(match load(path, 1, 0, None)? {
        Data::Image { detail, pages, .. } => format!("{detail}; {pages} page(s)"),
        Data::Table(table) => format!(
            "{} sheets; {} rows; {} columns",
            table.sheets.len(),
            table.total_rows,
            table.columns
        ),
        Data::Text(text) => format!("Document; {} characters", text.chars().count()),
        Data::External => "System viewer".into(),
    })
}

struct Table {
    sheets: Vec<String>,
    rows: Vec<Vec<String>>,
    columns: usize,
    total_rows: usize,
}

enum Data {
    Image {
        image: egui::ColorImage,
        detail: String,
        pages: usize,
    },
    Table(Table),
    Text(String),
    External,
}

#[derive(Default)]
pub struct Preview {
    pub source_paths: HashSet<PathBuf>,
    path: Option<PathBuf>,
    rx: Option<Receiver<Result<Data, String>>>,
    data: Option<Data>,
    texture: Option<egui::TextureHandle>,
    error: Option<String>,
    page: usize,
    sheet: usize,
    zoom: f32,
    fit: bool,
    filter: String,
    filtered_query: Option<String>,
    filtered_rows: Vec<usize>,
}

impl Preview {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn invalidate(&mut self) {
        self.path = None;
    }
    pub fn wants_preview(&self, path: &Path) -> bool {
        kind(path).is_some() && !self.source_paths.contains(path)
    }

    fn load(&mut self, ctx: &egui::Context, path: &Path, text: Option<String>) {
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.data = None;
        self.texture = None;
        self.error = None;
        self.filtered_query = None;
        let path = path.to_path_buf();
        let page = self.page;
        let sheet = self.sheet;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = load(&path, page, sheet, text).map_err(|e| format!("{e:#}"));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    /// Returns true when the user asks to edit the source of CSV/TSV/SVG.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        path: &Path,
        text: Option<String>,
    ) -> bool {
        if self.path.as_deref() != Some(path) {
            self.path = Some(path.to_path_buf());
            self.page = 1;
            self.sheet = 0;
            self.zoom = 1.0;
            self.fit = true;
            self.filter.clear();
            self.load(ctx, path, text.clone());
        }
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx = None;
                    match result {
                        Ok(mut data) => {
                            if let Data::Image { image, .. } = &mut data {
                                self.texture = Some(ctx.load_texture(
                                    "file-preview",
                                    std::mem::replace(
                                        image,
                                        egui::ColorImage::new(
                                            [1, 1],
                                            vec![egui::Color32::TRANSPARENT],
                                        ),
                                    ),
                                    egui::TextureOptions::LINEAR,
                                ));
                            }
                            self.data = Some(data);
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    self.error = Some("Preview worker stopped".into());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let mut source = false;
        let mut reload = false;
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("PREVIEW").small().strong());
                    if ui.button("Reload").clicked() {
                        reload = true;
                    }
                    if has_source(path) && ui.button("Edit Source").clicked() {
                        source = true;
                    }
                    if ui
                        .button(if kind(path) == Some(Kind::Video) {
                            "Play in System Player"
                        } else {
                            "Open in System App"
                        })
                        .clicked()
                    {
                        if let Err(error) = open_external(path) {
                            self.error = Some(error.to_string());
                        }
                    }
                    if let Some(Data::Image { detail, pages, .. }) = &self.data {
                        ui.label(detail);
                        if *pages > 1 {
                            if ui
                                .add_enabled(self.page > 1, egui::Button::new("◀"))
                                .clicked()
                            {
                                self.page -= 1;
                                reload = true;
                            }
                            ui.label(format!("Page {} / {}", self.page, pages));
                            if ui
                                .add_enabled(self.page < *pages, egui::Button::new("▶"))
                                .clicked()
                            {
                                self.page += 1;
                                reload = true;
                            }
                        }
                        ui.checkbox(&mut self.fit, "Fit");
                        if !self.fit {
                            ui.add(egui::Slider::new(&mut self.zoom, 0.1..=4.0).text("Zoom"));
                        }
                    }
                });
            });
        ui.separator();
        if reload {
            self.load(ctx, path, text);
        }
        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        if self.rx.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading preview…");
            });
            return source;
        }
        if let Some(texture) = &self.texture {
            if kind(path) == Some(Kind::Video) {
                ui.label("Video poster. Play in System Player opens the full video with audio.");
            }
            let size = texture.size_vec2();
            let scale = if self.fit {
                (ui.available_width() / size.x)
                    .min(ui.available_height() / size.y)
                    .clamp(0.05, 1.0)
            } else {
                self.zoom
            };
            egui::ScrollArea::both()
                .id_salt(("media-preview", path))
                .show(ui, |ui| {
                    ui.add(egui::Image::new((texture.id(), size * scale)));
                });
        } else if let Some(Data::Table(table)) = &self.data {
            let mut sheet = self.sheet;
            ui.horizontal_wrapped(|ui| {
                for (index, name) in table.sheets.iter().enumerate() {
                    if ui.selectable_label(index == self.sheet, name).clicked() {
                        sheet = index;
                    }
                }
                ui.label(format!(
                    "{} rows · {} columns",
                    table.total_rows, table.columns
                ));
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .hint_text("Filter rows")
                        .desired_width(180.0),
                );
            });
            if sheet != self.sheet {
                self.sheet = sheet;
                self.load(ctx, path, None);
                return source;
            }
            if table.total_rows > MAX_ROWS || table.columns > MAX_COLS {
                ui.label(format!("Preview limited to {MAX_ROWS} rows and {MAX_COLS} columns. Open in System App for the complete workbook."));
            }
            let query = self.filter.to_lowercase();
            if self.filtered_query.as_ref() != Some(&query) {
                self.filtered_rows = table
                    .rows
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| {
                        query.is_empty()
                            || row.iter().any(|cell| cell.to_lowercase().contains(&query))
                    })
                    .map(|(index, _)| index)
                    .collect();
                self.filtered_query = Some(query);
            }
            egui::ScrollArea::horizontal()
                .id_salt(("table-x", path))
                .show(ui, |ui| {
                    let columns = table.columns.clamp(1, MAX_COLS);
                    let mut builder = egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(egui_extras::Column::exact(48.0));
                    for _ in 0..columns {
                        builder = builder.column(
                            egui_extras::Column::initial(160.0)
                                .at_least(60.0)
                                .clip(true),
                        );
                    }
                    builder
                        .header(24.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("#");
                            });
                            for col in 0..columns {
                                header.col(|ui| {
                                    ui.strong(column_name(col));
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(24.0, self.filtered_rows.len(), |mut row| {
                                let index = self.filtered_rows[row.index()];
                                let cells = &table.rows[index];
                                row.col(|ui| {
                                    ui.label((index + 1).to_string());
                                });
                                for col in 0..columns {
                                    row.col(|ui| {
                                        let cell =
                                            cells.get(col).map(String::as_str).unwrap_or_default();
                                        ui.add(egui::Label::new(cell).truncate())
                                            .on_hover_text(cell);
                                    });
                                }
                            });
                        });
                });
        } else if let Some(Data::Text(text)) = &self.data {
            ui.label(egui::RichText::new("Document text preview · use Open in System App for original layout, images, and editing").small());
            egui::ScrollArea::both()
                .id_salt(("document-preview", path))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(text).selectable(true).wrap());
                });
        } else if matches!(self.data, Some(Data::External)) {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0); ui.heading(path.file_name().unwrap_or_default().to_string_lossy());
                ui.label("Use Open in System App to view or play this file with its installed application.");
            });
        }
        source
    }
}

pub fn open_external(path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = Command::new("explorer.exe");
    command
        .arg(path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Cannot launch the system viewer")?;
    Ok(())
}

fn load(path: &Path, page: usize, sheet: usize, source: Option<String>) -> Result<Data> {
    let size = std::fs::metadata(path)?.len();
    let kind = kind(path).context("Unsupported preview format")?;
    if size > MAX_FILE && !matches!(kind, Kind::Video | Kind::External | Kind::Pdf) {
        bail!("This file exceeds the 64 MiB native preview limit. Use Open in System App.");
    }
    match kind {
        Kind::Image => {
            let bytes = std::fs::read(path)?;
            if path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("svg"))
            {
                let mut options = resvg::usvg::Options::default();
                options.fontdb_mut().load_system_fonts();
                let image = egui_extras::image::load_svg_bytes_with_size(
                    &bytes,
                    egui::SizeHint::Size {
                        width: 1600,
                        height: 1200,
                        maintain_aspect_ratio: true,
                    },
                    &options,
                )
                .map_err(anyhow::Error::msg)?;
                Ok(Data::Image {
                    detail: format!("SVG · {} × {}", image.width(), image.height()),
                    image,
                    pages: 1,
                })
            } else {
                decode_image(&bytes, "Image", 1)
            }
        }
        Kind::Table => Ok(Data::Table(load_table(path, sheet, source)?)),
        Kind::Pdf => render_pdf(path, page),
        Kind::Video => {
            let output = crate::process::capture(
                Command::new("ffmpeg")
                    .args(["-v", "error", "-nostdin", "-i"])
                    .arg(path)
                    .args([
                        "-frames:v",
                        "1",
                        "-vf",
                        "scale=1280:720:force_original_aspect_ratio=decrease",
                        "-f",
                        "image2pipe",
                        "-vcodec",
                        "png",
                        "-",
                    ]),
                crate::process::Limits {
                    timeout: std::time::Duration::from_secs(15),
                    stdout: 16 * 1024 * 1024,
                    stderr: 64 * 1024,
                },
            )
            .context("Install FFmpeg for video posters, or use Play in System Player")?;
            if !output.status.success() {
                bail!(
                    "Cannot decode video poster: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            decode_image(&output.stdout, "Video", 1)
        }
        Kind::Document => Ok(Data::Text(docx_text(path)?)),
        Kind::External => Ok(Data::External),
    }
}

fn decode_image(bytes: &[u8], label: &str, pages: usize) -> Result<Data> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let original = reader.decode()?;
    let detail = format!("{label} · {} × {}", original.width(), original.height());
    let original = if original.width() > 3840 || original.height() > 2160 {
        original.thumbnail(3840, 2160)
    } else {
        original
    };
    let rgba = original.to_rgba8();
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    );
    Ok(Data::Image {
        image,
        detail,
        pages,
    })
}

fn load_table(path: &Path, sheet: usize, source: Option<String>) -> Result<Table> {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("csv") || e.eq_ignore_ascii_case("tsv"))
    {
        let text = match source {
            Some(text) => text,
            None => std::fs::read_to_string(path)?,
        };
        let delimiter = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("tsv"))
        {
            b'\t'
        } else {
            b','
        };
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .delimiter(delimiter)
            .from_reader(text.as_bytes());
        let mut rows = Vec::new();
        let mut columns = 0;
        let mut total_rows = 0;
        for record in reader.records() {
            let record = record?;
            total_rows += 1;
            columns = columns.max(record.len());
            if rows.len() < MAX_ROWS {
                rows.push(record.iter().take(MAX_COLS).map(str::to_string).collect());
            }
        }
        return Ok(Table {
            sheets: vec!["Data".into()],
            rows,
            columns,
            total_rows,
        });
    }
    // Validate the same immutable snapshot passed to the legacy reader;
    // reopening a path after validation would allow a file-swap race.
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        bail!("Workbook exceeds the 64 MiB native preview limit");
    }
    let bytes: std::sync::Arc<[u8]> = bytes.into();
    if path.extension().is_some_and(|e| {
        e.eq_ignore_ascii_case("xlsx")
            || e.eq_ignore_ascii_case("xlsb")
            || e.eq_ignore_ascii_case("ods")
    }) {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&*bytes))?;
        if archive.len() > 10_000
            || archive.decompressed_size().unwrap_or(u128::MAX) > 128 * 1024 * 1024
        {
            bail!("Workbook exceeds the native preview expansion limit. Use Open in System App.");
        }
        let mut xml_bytes = 0;
        for index in 0..archive.len() {
            let entry = archive.by_index(index)?;
            let name = entry.name().to_ascii_lowercase();
            if name.ends_with(".xml") || name.ends_with(".rels") {
                xml_bytes += validate_workbook_xml(entry)?;
                if xml_bytes > 128 * 1024 * 1024 {
                    bail!("Workbook XML exceeds the native preview budget");
                }
            }
        }
    }
    let mut workbook = calamine::open_workbook_auto_from_rs(std::io::Cursor::new(bytes))?;
    let sheets = workbook.sheet_names().to_vec();
    let name = sheets
        .get(sheet)
        .context("Workbook has no worksheet at this index")?;
    let range = workbook.worksheet_range(name)?;
    let (total_rows, columns) = range.get_size();
    let rows = range
        .rows()
        .take(MAX_ROWS)
        .map(|row| row.iter().take(MAX_COLS).map(ToString::to_string).collect())
        .collect();
    Ok(Table {
        sheets,
        rows,
        columns,
        total_rows,
    })
}

/// Bound work before calamine's older XML attribute iterator sees input.
/// The current quick-xml dependency provides the non-quadratic duplicate check.
fn validate_workbook_xml(input: impl Read) -> Result<u64> {
    use quick_xml::events::Event;
    const LIMIT: u64 = 16 * 1024 * 1024;
    let mut reader = quick_xml::Reader::from_reader(std::io::BufReader::new(input.take(LIMIT + 1)));
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    loop {
        let event = reader.read_event_into(&mut buffer)?;
        if reader.buffer_position() > LIMIT {
            bail!("Workbook XML part exceeds 16 MiB");
        }
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                for (index, attribute) in element.attributes().enumerate() {
                    if index >= 256 {
                        bail!("Workbook XML element exceeds 256 attributes");
                    }
                    attribute?;
                }
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                    if depth > 128 {
                        bail!("Workbook XML nesting exceeds 128 levels");
                    }
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).context("Unbalanced workbook XML")?;
            }
            Event::DocType(_) => {
                bail!("Document type declarations are not supported in workbook XML")
            }
            Event::Eof => {
                if depth != 0 {
                    bail!("Unclosed workbook XML element");
                }
                return Ok(reader.buffer_position());
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn docx_text(path: &Path) -> Result<String> {
    let mut zip = zip::ZipArchive::new(File::open(path)?)?;
    let entry = zip.by_name("word/document.xml")?;
    if entry.size() > 16 * 1024 * 1024 {
        bail!("Document text is too large for native preview");
    }
    let mut xml = String::new();
    entry.take(16 * 1024 * 1024 + 1).read_to_string(&mut xml)?;
    let mut reader = quick_xml::Reader::from_str(&xml);
    let mut text = String::new();
    let mut in_text = false;
    loop {
        use quick_xml::events::Event;
        match reader.read_event()? {
            Event::Start(e) if e.local_name().as_ref() == b"t" => in_text = true,
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_text = false,
                b"p" => text.push('\n'),
                b"tc" => text.push('\t'),
                _ => {}
            },
            Event::Empty(e) if matches!(e.local_name().as_ref(), b"br" | b"cr") => text.push('\n'),
            Event::Empty(e) if e.local_name().as_ref() == b"tab" => text.push('\t'),
            Event::Text(e) if in_text => text.push_str(&quick_xml::escape::unescape(&e.decode()?)?),
            Event::GeneralRef(e) if in_text => {
                let entity = e.decode()?;
                text.push_str(&quick_xml::escape::unescape(&format!("&{entity};"))?);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(text)
}

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "oxide-preview-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn render_pdf(path: &Path, page: usize) -> Result<Data> {
    let info = crate::process::capture(
        Command::new("pdfinfo").arg(path).env("LC_ALL", "C"),
        crate::process::Limits {
            timeout: std::time::Duration::from_secs(10),
            stdout: 64 * 1024,
            stderr: 64 * 1024,
        },
    )
    .context(
        "Install Poppler (pdfinfo/pdftoppm) for inline PDF preview, or use Open in System App",
    )?;
    if !info.status.success() {
        bail!("Cannot read PDF: {}", String::from_utf8_lossy(&info.stderr));
    }
    let pages: usize = String::from_utf8_lossy(&info.stdout)
        .lines()
        .find_map(|line| {
            line.strip_prefix("Pages:")
                .and_then(|n| n.trim().parse().ok())
        })
        .unwrap_or(1);
    let temp = TempDir::new()?;
    let prefix = temp.0.join("page");
    let page = page.clamp(1, pages.max(1)).to_string();
    let output = crate::process::capture(
        Command::new("pdftoppm")
            .args([
                "-f",
                &page,
                "-l",
                &page,
                "-singlefile",
                "-scale-to",
                "1800",
                "-png",
            ])
            .arg(path)
            .arg(&prefix),
        crate::process::Limits {
            timeout: std::time::Duration::from_secs(20),
            stdout: 64 * 1024,
            stderr: 64 * 1024,
        },
    )
    .context("Install Poppler (pdftoppm) for inline PDF preview")?;
    if !output.status.success() {
        bail!(
            "Cannot render PDF: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    decode_image(&std::fs::read(prefix.with_extension("png"))?, "PDF", pages)
}

fn column_name(mut column: usize) -> String {
    let mut name = String::new();
    loop {
        name.insert(0, (b'A' + (column % 26) as u8) as char);
        if column < 26 {
            break;
        }
        column = column / 26 - 1;
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_quotes_multiline_and_uneven_rows() {
        let table = load_table(
            Path::new("fixture.csv"),
            0,
            Some("name,note\nAda,\"hello, world\"\nLin,\"two\nlines\",extra\n".into()),
        )
        .unwrap();
        assert_eq!(table.total_rows, 3);
        assert_eq!(table.columns, 3);
        assert_eq!(table.rows[1][1], "hello, world");
        assert_eq!(table.rows[2][1], "two\nlines");
        assert_eq!(column_name(0), "A");
        assert_eq!(column_name(26), "AA");
    }
    #[test]
    fn image_preview_decodes_real_pixels() {
        let image = image::RgbaImage::from_pixel(3, 2, image::Rgba([1, 2, 3, 255]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        match decode_image(bytes.get_ref(), "Image", 1).unwrap() {
            Data::Image { image, .. } => {
                assert_eq!(image.size, [3, 2]);
                assert_eq!(image.pixels[0], egui::Color32::from_rgb(1, 2, 3));
            }
            _ => panic!("expected image"),
        }
    }
    fn workbook_fixture(path: &Path, attributes: usize) {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        let mut archive = zip::ZipWriter::new(File::create(path).unwrap());
        let extra = (0..attributes)
            .map(|index| format!(" x{index}=\"v\""))
            .collect::<String>();
        let parts = [
            ("[Content_Types].xml", r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/></Types>"#.to_string()),
            ("_rels/.rels", r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#.to_string()),
            ("xl/workbook.xml", format!(r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"{extra}/></sheets></workbook>"#)),
            ("xl/_rels/workbook.xml.rels", r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#.to_string()),
            ("xl/worksheets/sheet1.xml", r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>safe</t></is></c></row></sheetData></worksheet>"#.to_string()),
        ];
        for (name, text) in parts {
            archive
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            archive.write_all(text.as_bytes()).unwrap();
        }
        archive.finish().unwrap();
    }

    #[test]
    fn sec_001_rejects_excessive_workbook_attributes() {
        let temp = TempDir::new().unwrap();
        let path = temp.0.join("attributes.xlsx");
        workbook_fixture(&path, 1_000);
        let result = load_table(&path, 0, None);
        assert!(
            result.is_err(),
            "Untrusted XML must be checked before the legacy spreadsheet parser"
        );
    }

    #[test]
    fn sec_001_accepts_normal_workbook() {
        let temp = TempDir::new().unwrap();
        let path = temp.0.join("normal.xlsx");
        workbook_fixture(&path, 0);
        let table = load_table(&path, 0, None).unwrap();
        assert_eq!(table.rows[0][0], "safe");
    }
    #[test]
    fn sec_001_workbook_xml_depth_and_doctype_are_bounded() {
        assert!(validate_workbook_xml(b"<!DOCTYPE x><x/>".as_slice()).is_err());
        let nested = format!("{}{}", "<x>".repeat(129), "</x>".repeat(129));
        assert!(validate_workbook_xml(nested.as_bytes()).is_err());
        assert!(validate_workbook_xml(b"<x><y value=\"safe\"/></x>".as_slice()).is_ok());
    }
}
