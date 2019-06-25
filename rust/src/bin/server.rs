extern crate hyper;
#[macro_use]
extern crate error_type;

extern crate futures;
extern crate clap;
// extern crate tokio_fs;
use clap::App;

use futures::{future::Either};

use hyper::{header, Body, Request, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn;

use hyper::http::status::StatusCode;

use std::{
  error::Error as StdError,
  io,
  net::SocketAddr,
  path::{Path, PathBuf}
};
fn main() {
  if let Err(e) = run() {
    println!("Error: {}", e.description());
  }
}
fn run() -> Result<(), Error> {
  let config = parse_config_from_cmdline()?;
  let Config {addr, root_dir, .. } = config;
  let server = Server::bind(&addr)
    .serve(move || {
      let root_dir = root_dir.clone();
      service_fn(move |req| serve(req, &root_dir.clone()))
    })
    .map_err(|e| {
      println!("There was an error: {}", e);
    });

  hyper::rt::run(server);
  Ok(())
}
#[derive(Clone)]
struct Config {
  addr: SocketAddr,
  root_dir: PathBuf,
  file_name: PathBuf
}
fn parse_config_from_cmdline() -> Result<Config, Error> {
  let matches = App::new("Leo server")
    .version(env!("CARGO_PKG_VERSION"))
    .about("Server that serves Leo outline and its external files")
    .args_from_usage(
      "<FILE> 'leo outline file'
       [ADDR] -a --addr=[ADDR] 'sets the IP:PORT combination (default \"127.0.0.1:4000\")'"
    ).get_matches();
  let addr = matches.value_of("ADDR").unwrap_or("127.0.0.1:4000");
  let fname_arg = matches.value_of("FILE").unwrap();
  let fname_a = std::env::current_dir()?.with_file_name(fname_arg);
  let fname_b = Path::new(fname_arg);

  let root_dir = if fname_b.is_relative() {
    fname_a.parent().unwrap()
  } else {
    fname_b.parent().unwrap()
  };

  let file_name = root_dir.join(fname_b.file_name().unwrap());
  println!("file: [{}]", file_name.display());
  println!("root_dir: [{}]", root_dir.display());
  println!("addr: [{}]", addr);
  Ok(Config {
    addr: addr.parse()?,
    root_dir: PathBuf::from(root_dir),
    file_name: file_name
  })
}
// The function that returns a future of http responses for each hyper Request
// that is received. Errors are turned into an Error response (404 or 500).
fn serve(
    req: Request<Body>,
    root_dir: &PathBuf,
) -> impl Future<Item = Response<Body>, Error = Error> {
    let uri_path = req.uri().path();
    if let Some(path) = local_path_for_request(&uri_path, root_dir) {
        Either::A(tokio_fs::file::File::open(path.clone()).then(
            move |open_result| match open_result {
                Ok(file) => Either::A(read_file(file, path)),
                Err(e) => Either::B(handle_io_error(e)),
            },
        ))
    } else {
        Either::B(internal_server_error())
    }
}
// Read the file completely and construct a 200 response with that file as
// the body of the response.
fn read_file<'a>(
    file: tokio_fs::File,
    path: PathBuf,
) -> impl Future<Item = Response<Body>, Error = Error> {
    let buf: Vec<u8> = Vec::new();
    tokio_io::io::read_to_end(file, buf)
        .map_err(Error::Io)
        .and_then(move |(_, buf)| {
            let mime_type = file_path_mime(&path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_LENGTH, buf.len() as u64)
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .body(Body::from(buf))
                .map_err(Error::from)
        })
}
// Handle the one special io error (file not found) by returning a 404, otherwise
// return a 500
fn handle_io_error(error: io::Error) -> impl Future<Item = Response<Body>, Error = Error> {
    match error.kind() {
        io::ErrorKind::NotFound => Either::A(futures::future::result(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .map_err(Error::from),
        )),
        _ => Either::B(internal_server_error()),
    }
}
fn file_path_mime(file_path: &Path) -> mime::Mime {
    let mime_type = match file_path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => mime::TEXT_HTML,
        Some("css") => mime::TEXT_CSS,
        Some("js") => mime::TEXT_JAVASCRIPT,
        Some("jpg") => mime::IMAGE_JPEG,
        Some("png") => mime::IMAGE_PNG,
        Some("svg") => mime::IMAGE_SVG,
        Some("wasm") => "application/wasm".parse::<mime::Mime>().unwrap(),
        _ => mime::TEXT_PLAIN,
    };
    mime_type
}
fn local_path_for_request(request_path: &str, root_dir: &Path) -> Option<PathBuf> {
    // This is equivalent to checking for hyper::RequestUri::AbsoluteUri
    if !request_path.starts_with("/") {
        return None;
    }
    // Trim off the url parameters starting with '?'
    let end = request_path.find('?').unwrap_or(request_path.len());
    let request_path = &request_path[0..end];

    // Append the requested path to the root directory
    let mut path = root_dir.to_owned();
    if request_path.starts_with('/') {
        path.push(&request_path[1..]);
    } else {
        return None;
    }

    // Maybe turn directory requests into index.html requests
    if request_path.ends_with('/') {
        path.push("index.html");
    }

    Some(path)
}
fn internal_server_error() -> impl Future<Item = Response<Body>, Error = Error> {
    futures::future::result(
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_LENGTH, 0)
            .body(Body::empty()),
    )
    .map_err(Error::from)
}
// The custom Error type that encapsulates all the possible errors
// that can occur in this crate. This macro defines it and
// automatically creates Display, Error, and From implementations for
// all the variants.
error_type! {
    #[derive(Debug)]
    enum Error {
        Io(io::Error) { },
        HttpError(http::Error) { },
        AddrParse(std::net::AddrParseError) { },
        Std(Box<StdError + Send + Sync>) {
            desc (e) e.description();
        },
        ParseInt(std::num::ParseIntError) { },
    }
}
