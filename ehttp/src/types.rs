use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Write};

#[cfg(feature = "multipart")]
use crate::multipart::MultipartBuilder;

/// A simple HTTP request.
#[derive(Clone, Debug)]
pub struct Request {
    /// "GET", "POST", …
    pub method: String,

    /// https://…
    pub url: String,

    /// The data you send with e.g. "POST".
    pub body: Vec<u8>,

    /// ("Accept", "*/*"), …
    pub headers: BTreeMap<String, String>,
}

/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
pub enum Method {
    ///The GET method requests a representation of the specified resource. Requests using GET should only retrieve data.
    Get,
    /// The HEAD method asks for a response identical to a GET request, but without the response body.
    Head,
    /// The POST method submits an entity to the specified resource, often causing a change in state or side effects on the server.
    Post,
    /// The PUT method replaces all current representations of the target resource with the request payload.
    Put,
    /// The DELETE method deletes the specified resource.
    Delete,
    /// The CONNECT method establishes a tunnel to the server identified by the target resource.
    Connect,
    /// The OPTIONS method describes the communication options for the target resource.
    Options,
    /// The TRACE method performs a message loop-back test along the path to the target resource.
    Trace,
    /// The PATCH method applies partial modifications to a resource.
    Patch,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Head => write!(f, "HEAD"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Connect => write!(f, "CONNECT"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Trace => write!(f, "TRACE"),
            Method::Patch => write!(f, "PATCH"),
        }
    }
}

impl Request {
    /// Create a `GET` request with the given url.
    #[allow(clippy::needless_pass_by_value)]
    pub fn get(url: impl ToString) -> Self {
        Self {
            method: Method::Get.to_string(),
            url: url.to_string(),
            body: vec![],
            headers: crate::headers(&[("Accept", "*/*")]),
        }
    }

    #[cfg(feature = "multipart")]
    /// Creates a `POST` mutlipart request withen given url and builder
    pub fn multipart(url: impl ToString, builder: MultipartBuilder) -> std::io::Result<Self> {
        let (content_type, data) = builder.finish()?;
        Ok(Self {
            method: Method::Post.to_string(),
            url: url.to_string(),
            body: data,
            headers: crate::headers(&[
                ("Accept", "*/*"),
                ("Content-Type", &*content_type),
            ]),
        })
    }

    /// Create a `POST` request with the given url and body.
    #[allow(clippy::needless_pass_by_value)]
    pub fn post(url: impl ToString, body: Vec<u8>) -> Self {
        Self {
            method: Method::Post.to_string(),
            url: url.to_string(),
            body,
            headers: crate::headers(&[
                ("Accept", "*/*"),
                ("Content-Type", "text/plain; charset=utf-8"),
            ]),
        }
    }

    /// Allows to change used method
    pub fn method(mut self, method: Method) -> Self {
        self.method = method.to_string();
        self
    }
}

/// Response from a completed HTTP request.
#[derive(Clone, Eq, PartialEq)]
pub struct Response {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,

    /// Did we get a 2xx response code?
    pub ok: bool,

    /// Status code (e.g. `404` for "File not found").
    pub status: u16,

    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,

    /// The returned headers. All header names are lower-case.
    pub headers: BTreeMap<String, String>,

    /// The raw bytes of the response body.
    pub bytes: Vec<u8>,
}

impl Response {
    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Response")
            .field("url", &self.url)
            .field("ok", &self.ok)
            .field("status", &self.status)
            .field("status_text", &self.status_text)
            //    .field("bytes", &self.bytes)
            .field("headers", &self.headers)
            .finish_non_exhaustive()
    }
}

/// An HTTP response status line and headers used for the [`streaming`](crate::streaming) API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialResponse {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,

    /// Did we get a 2xx response code?
    pub ok: bool,

    /// Status code (e.g. `404` for "File not found").
    pub status: u16,

    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,

    /// The returned headers. All header names are lower-case.
    pub headers: BTreeMap<String, String>,
}

impl PartialResponse {
    pub fn complete(self, bytes: Vec<u8>) -> Response {
        let Self {
            url,
            ok,
            status,
            status_text,
            headers,
        } = self;
        Response {
            url,
            ok,
            status,
            status_text,
            headers,
            bytes,
        }
    }
}

/// A description of an error.
///
/// This is only used when we fail to make a request.
/// Any response results in `Ok`, including things like 404 (file not found).
pub type Error = String;

/// A type-alias for `Result<T, ehttp::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
