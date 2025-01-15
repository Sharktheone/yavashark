use crate::{MutObject, Object, ObjectHandle, Realm};
use std::cell::RefCell;
use std::io;
use std::io::{BufRead, BufReader};
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
    fn get(url: String) -> crate::Result<String> {
        Ok(String::new()) //TODO
    }

    fn server(
        ip: String,
        port: u16,
        callback: ObjectHandle,
        #[realm] realm: &mut Realm,
    ) -> crate::Result<()> {
        let mut server = SimpleHttpServer::new(ip, port, move |request| {
            let Ok(obj) = request.into_object(realm) else {
                return;
            };

            _ = callback.call(realm, vec![obj.into()], callback.clone().into_value());
        });

        server.start()?;

        Ok(())
    }
}

struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
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

struct SimpleHttpServer<C: FnMut(HttpRequest)> {
    ip: String,
    port: u16,
    callback: C,
}

impl<C: FnMut(HttpRequest)> SimpleHttpServer<C> {
    const fn new(ip: String, port: u16, callback: C) -> Self {
        Self { ip, port, callback }
    }

    fn start(&mut self) -> io::Result<()> {
        println!("Starting server on {}:{}", self.ip, self.port);
        let listener = std::net::TcpListener::bind(format!("{}:{}", self.ip, self.port))?;

        for stream in listener.incoming() {
            let stream = stream?;
            let buf_reader = BufReader::new(&stream);
            let http_request = buf_reader
                .lines()
                .take_while(|line| line.as_ref().map(|l| !l.is_empty()).unwrap_or(false))
                .collect::<Result<Vec<_>, _>>()?;

            println!("{}", http_request.join("\n"));

            let Some(mut request_line) = http_request.first().map(|x| x.split_whitespace()) else {
                continue;
            };

            let Some(method) = request_line.next() else {
                continue;
            };
            let Some(url) = request_line.next() else {
                continue;
            };
            let headers = http_request
                .iter()
                .skip(1)
                .map(|x| {
                    let mut parts = x.split(':');
                    let key = parts.next().unwrap_or("").trim().to_string();
                    let value = parts.next().unwrap_or("").trim().to_string();
                    (key, value)
                })
                .collect::<Vec<_>>();

            //TODO: Parse body

            let request = HttpRequest {
                method: method.to_string(),
                url: url.to_string(),
                headers,
                body: String::new(),
            };

            (self.callback)(request);
        }

        Ok(())
    }
}
