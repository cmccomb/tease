//! Bounded raster-image and CSV table values.
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageFormat, ImageReader, Limits};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
const IMAGE_BYTES: usize = 8 * 1024 * 1024;
/// A PNG or JPEG, encoded for transport to a local webview.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageData {
    /// Exactly `image/png` or `image/jpeg`, matching the decoded bytes.
    pub mime: String,
    /// Standard padded base64 data, without a data-URL prefix.
    pub base64: String,
}
impl ImageData {
    /// Validate and encode raster bytes. Limit: 8 MiB, 4096×4096, 128 MiB decoder allocation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mime = validate_image(bytes)?;
        Ok(Self {
            mime: mime.into(),
            base64: STANDARD.encode(bytes),
        })
    }
    /// Decode and validate the image bytes for a model.
    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        if self.base64.len() > IMAGE_BYTES.div_ceil(3) * 4 {
            return Err("Image exceeds 8 MiB.".into());
        }
        let bytes = STANDARD
            .decode(&self.base64)
            .map_err(|_| "Invalid image encoding.")?;
        if validate_image(&bytes)? != self.mime {
            return Err("Image media type does not match its content.".into());
        }
        Ok(bytes)
    }
}
fn validate_image(bytes: &[u8]) -> Result<&'static str, String> {
    if bytes.len() > IMAGE_BYTES {
        return Err("Image exceeds 8 MiB.".into());
    }
    let format = image::guess_format(bytes).map_err(|_| "Select a PNG or JPEG image.")?;
    let mime = match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        _ => return Err("Only PNG and JPEG images are supported.".into()),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|e| format!("Image cannot be decoded within the size limits: {e}"))?;
    Ok(mime)
}
/// A rectangular string table. Columns must be nonempty and unique.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    /// Up to 64 column names.
    pub columns: Vec<String>,
    /// Up to 1,000 rows, with at most 8,192 bytes per cell and 2 MiB total text.
    pub rows: Vec<Vec<String>>,
}
impl Table {
    /// Parse UTF-8 comma-separated data with a header row and standard CSV quoting.
    pub fn from_csv(text: &str) -> Result<Self, String> {
        if text.len() > 2 * 1024 * 1024 {
            return Err("CSV exceeds 2 MiB.".into());
        }
        let mut reader =
            csv::ReaderBuilder::new().from_reader(text.trim_start_matches('\u{feff}').as_bytes());
        let columns = reader
            .headers()
            .map_err(|e| e.to_string())?
            .iter()
            .map(str::to_owned)
            .collect();
        let mut rows = Vec::new();
        for row in reader.records() {
            if rows.len() >= 1000 {
                return Err("CSV exceeds 1,000 rows.".into());
            }
            rows.push(
                row.map_err(|e| format!("Invalid CSV row: {e}"))?
                    .iter()
                    .map(str::to_owned)
                    .collect(),
            );
        }
        let table = Self { columns, rows };
        table.validate()?;
        Ok(table)
    }
    /// Serialize exact cell contents with CSV quoting, without spreadsheet-specific escaping.
    pub fn to_csv(&self) -> Result<String, String> {
        self.csv(false)
    }
    pub(crate) fn export_csv(&self) -> Result<String, String> {
        self.csv(true)
    }
    fn csv(&self, safe: bool) -> Result<String, String> {
        self.validate()?;
        let mut writer = csv::Writer::from_writer(Vec::new());
        for row in std::iter::once(&self.columns).chain(&self.rows) {
            let cells: Vec<_> = row
                .iter()
                .map(|cell| {
                    let trimmed = cell.trim_start();
                    if safe
                        && (cell.starts_with(['\t', '\r', '\n'])
                            || trimmed.starts_with(['=', '+', '-', '@']))
                        && !cell.parse::<f64>().is_ok_and(f64::is_finite)
                    {
                        format!("'{cell}")
                    } else {
                        cell.clone()
                    }
                })
                .collect();
            writer.write_record(cells).map_err(|e| e.to_string())?;
        }
        String::from_utf8(writer.into_inner().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.columns.is_empty() || self.columns.len() > 64 || self.rows.len() > 1000 {
            return Err("Tables need 1–64 columns and at most 1,000 rows.".into());
        }
        if self
            .columns
            .iter()
            .enumerate()
            .any(|(i, c)| c.trim().is_empty() || self.columns[..i].contains(c))
        {
            return Err("Table headers must be nonempty and unique.".into());
        }
        if self.rows.iter().any(|r| r.len() != self.columns.len()) {
            return Err("Every table row must match the header width.".into());
        }
        let cells = self.columns.iter().chain(self.rows.iter().flatten());
        if cells.clone().any(|c| c.len() > 8192)
            || cells.map(String::len).sum::<usize>() > 2 * 1024 * 1024
        {
            return Err("Table exceeds its text-size limits.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_export_neutralizes_formulas_but_preserves_numbers() {
        let table = Table {
            columns: vec!["value".into()],
            rows: vec![
                vec!["=1+1".into()],
                vec!["-3.5".into()],
                vec!["\t=1+1".into()],
            ],
        };
        let exported = Table::from_csv(&table.export_csv().unwrap()).unwrap();
        assert_eq!(exported.rows[0][0], "'=1+1");
        assert_eq!(exported.rows[1][0], "-3.5");
        assert_eq!(exported.rows[2][0], "'\t=1+1");
    }
}
