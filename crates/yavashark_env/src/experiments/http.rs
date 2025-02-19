mod status;

use crate::experiments::http::status::status_code_to_reason;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Result};
use std::cell::RefCell;
use std::fmt::Write;
use std::io::{BufRead, BufReader, Read, Write as _};
use std::{io, mem};
use yavashark_macro::{object, properties_new};
use yavashark_value::{IntoValue, Obj};

#[object]
#[derive(Debug)]
pub struct Http {}

impl Http {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableHttp {
                object: MutObject::new(realm),
            }),
        };

        this.initialize(realm.intrinsics.func.clone().into())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Http {
    fn get(_url: String) -> crate::Result<String> {
        Ok(String::new()) //TODO
    }

    fn server(
        ip: String,
        port: u16,
        callback: ObjectHandle,
        #[realm] realm: &mut Realm,
    ) -> crate::Result<()> {
        let mut server = SimpleHttpServer::new(ip, port, move |realm, request, response| {
            let Ok(obj) = request.into_object(realm) else {
                return;
            };

            let res = response.into_object();

            let res = callback.call(
                realm,
                vec![obj.into(), res.into()],
                callback.clone().into_value(),
            );

            if let Err(err) = res {
                eprintln!("Error in callback: {err:?}");
            }
        });

        server.start(realm)?;

        Ok(())
    }
}

struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
}

#[object]
#[derive(Debug)]
struct HttpResponseWriter {
    #[mutable]
    stream: std::net::TcpStream,
    #[mutable]
    status: u16,
    #[mutable]
    headers: Vec<(String, String)>,
    #[mutable]
    body: Vec<u8>,
}

impl HttpResponseWriter {
    fn new(stream: std::net::TcpStream, realm: &Realm) -> Result<Self> {
        let mut this = Self {
            inner: RefCell::new(MutableHttpResponseWriter {
                object: MutObject::new(realm),
                stream,
                status: 200,
                headers: Vec::new(),
                body: Vec::new(),
            }),
        };

        this.initialize(realm.intrinsics.func.clone().into())?;

        Ok(this)
    }
}

#[properties_new(raw)]
impl HttpResponseWriter {
    fn set_body(&self, body: String) {
        let inner = &mut self.inner.borrow_mut();

        inner.body = body.into_bytes();
    }

    fn set_header(&self, key: String, value: String) {
        let inner = &mut self.inner.borrow_mut();

        inner.headers.push((key, value));
    }

    fn set_status(&self, status: u16) {
        let inner = &mut self.inner.borrow_mut();

        inner.status = status;
    }

    fn finish(&self) -> Res {
        let mut inner = self.inner.borrow_mut();

        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            inner.status,
            status_code_to_reason(inner.status)
        );

        for (key, value) in &inner.headers {
            write!(response, "{key}: {value}\r\n")?;
        }

        response.push_str("\r\n");

        inner.stream.write_all(response.as_bytes())?;

        println!("Writing response: {response:?}");

        let body = mem::take(&mut inner.body);

        inner.stream.write_all(&body)?;

        let _ = mem::replace(&mut inner.body, body);

        Ok(())
    }
}

impl HttpRequest {
    fn into_object(self, realm: &Realm) -> crate::Result<ObjectHandle> {
        let obj = Object::new(realm);
        obj.define_property("method".into(), self.method.into_value())?;
        obj.define_property("url".into(), self.url.into_value())?;

        let headers = self
            .headers
            .into_iter()
            .map(|(key, value)| (key.into_value(), value.into_value()))
            .collect::<Vec<_>>();

        let headers = Object::from_values(headers, realm)?;

        obj.define_property("headers".into(), headers.into())?;
        obj.define_property("body".into(), self.body.into_value())?;

        Ok(obj)
    }
}

struct SimpleHttpServer<C: FnMut(&mut Realm, HttpRequest, HttpResponseWriter)> {
    ip: String,
    port: u16,
    callback: C,
}

impl<C: FnMut(&mut Realm, HttpRequest, HttpResponseWriter)> SimpleHttpServer<C> {
    const fn new(ip: String, port: u16, callback: C) -> Self {
        Self { ip, port, callback }
    }

    fn start(&mut self, realm: &mut Realm) -> io::Result<()> {
        println!("Starting server on {}:{}", self.ip, self.port);
        let listener = std::net::TcpListener::bind(format!("{}:{}", self.ip, self.port))?;

        for stream in listener.incoming() {
            let stream = stream?;
            let mut buf_reader = BufReader::new(&stream);

            let mut header_lines = Vec::new();
            loop {
                let mut line = String::new();
                let bytes_read = buf_reader.read_line(&mut line)?;
                if bytes_read == 0 || line.trim().is_empty() {
                    break;
                }
                header_lines.push(line.trim_end().to_string());
            }

            let mut header_iter = header_lines.iter();
            let Some(request_line) = header_iter.next() else { continue };

            let mut parts = request_line.split_whitespace();
            let method = match parts.next() {
                Some(m) => m.to_string(),
                None => continue,
            };
            let url = match parts.next() {
                Some(u) => u.to_string(),
                None => continue,
            };

            let headers: Vec<(String, String)> = header_iter
                .filter_map(|line| {
                    let mut parts = line.splitn(2, ':');
                    let key = parts.next()?.trim().to_string();
                    let value = parts.next()?.trim().to_string();
                    Some((key, value))
                })
                .collect();

            let mut content_length: usize = 0;
            for (key, value) in &headers {
                if key.eq_ignore_ascii_case("Content-Length") {
                    if let Ok(val) = value.parse() {
                        content_length = val;
                    }
                }
            }

            let mut body = String::new();
            if content_length > 0 {
                let mut limited_reader = buf_reader.take(content_length as u64);
                limited_reader.read_to_string(&mut body)?;
            }

            let request = HttpRequest {
                method,
                url,
                headers,
                body,
            };

            let Ok(response) = HttpResponseWriter::new(stream, realm) else {
                continue;
            };

            (self.callback)(realm, request, response);
        }

        Ok(())
    }
}
