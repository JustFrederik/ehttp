/// Taken from ureq_multipart 1.1.1
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use mime::Mime;
use rand::Rng;

const BOUNDARY_LEN: usize = 29;

fn opt_filename(path: &Path) -> Option<&str> {
    path.file_name().and_then(|filename| filename.to_str())
}

fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Uniform::from(0..=9))
        .take(len)
        .map(|num| num.to_string())
        .collect()
}

fn mime_filename(path: &Path) -> (Mime, Option<&str>) {
    let content_type = mime_guess::from_path(path);
    let filename = opt_filename(path);
    (content_type.first_or_octet_stream(), filename)
}

#[derive(Debug)]
pub struct MultipartBuilder {
    boundary: String,
    inner: Vec<u8>,
    data_written: bool,
}

impl Default for MultipartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl MultipartBuilder {
    pub fn new() -> Self {
        Self {
            boundary: random_alphanumeric(BOUNDARY_LEN),
            inner: Vec::new(),
            data_written: false,
        }
    }
    /// add text field
    ///
    /// * name field name
    /// * text field text value
    pub fn add_text(mut self, name: &str, text: &str) -> io::Result<Self> {
        self.write_field_headers(name, None, None)?;
        self.inner.write_all(text.as_bytes())?;
        Ok(self)
    }
    /// add file
    ///
    /// * name file field name
    /// * path the sending file path
    pub fn add_file<P: AsRef<Path>>(self, name: &str, path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let (content_type, filename) = mime_filename(path);
        let mut file = File::open(path)?;
        self.add_stream(&mut file, name, filename, Some(content_type))
    }
    /// add some stream
    pub fn add_stream<S: Read>(
        mut self,
        stream: &mut S,
        name: &str,
        filename: Option<&str>,
        content_type: Option<Mime>,
    ) -> io::Result<Self> {
        // This is necessary to make sure it is interpreted as a file on the server end.
        let content_type = Some(content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM));
        self.write_field_headers(name, filename, content_type)?;
        io::copy(stream, &mut self.inner)?;
        Ok(self)
    }
    fn write_boundary(&mut self) -> io::Result<()> {
        if self.data_written {
            self.inner.write_all(b"\r\n")?;
        }

        write!(
            self.inner,
            "-----------------------------{}\r\n",
            self.boundary
        )
    }
    fn write_field_headers(
        &mut self,
        name: &str,
        filename: Option<&str>,
        content_type: Option<Mime>,
    ) -> io::Result<()> {
        self.write_boundary()?;
        if !self.data_written {
            self.data_written = true;
        }
        write!(
            self.inner,
            "Content-Disposition: form-data; name=\"{name}\""
        )?;
        if let Some(filename) = filename {
            write!(self.inner, "; filename=\"{filename}\"")?;
        }
        if let Some(content_type) = content_type {
            write!(self.inner, "\r\nContent-Type: {content_type}")?;
        }
        self.inner.write_all(b"\r\n\r\n")
    }
    /// general multipart data
    ///
    /// # Return
    /// * (content_type,post_data)
    ///    * content_type http header content type
    ///    * post_data ureq.req.send_send_bytes(&post_data)
    ///
    pub fn finish(mut self) -> io::Result<(String, Vec<u8>)> {
        if self.data_written {
            self.inner.write_all(b"\r\n")?;
        }

        // always write the closing boundary, even for empty bodies
        write!(
            self.inner,
            "-----------------------------{}--\r\n",
            self.boundary
        )?;
        Ok((
            format!(
                "multipart/form-data; boundary=---------------------------{}",
                self.boundary
            ),
            self.inner,
        ))
    }
}