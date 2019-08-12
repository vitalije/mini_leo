
#[path="../model.rs"]
mod model;

extern crate xml;
extern crate clap;
#[macro_use]
extern crate error_type;
use crate::model::{VData, LevGnx, LevGnxOps, Outline, OutlineOps};
use std::collections::HashMap;
use clap::App;
use xml::reader::{ParserConfig, XmlEvent};

fn parser_config() -> ParserConfig {
  ParserConfig::new()
      .cdata_to_characters(true)
      .whitespace_to_characters(true)
}

use std::{
  error::Error as StdError,
  io,
  io::{Read, BufReader},
  fs::File,
  path::{PathBuf}
};
fn parse_config_from_cmdline() -> Result<PathBuf, Error> {
  let matches = App::new("Leo server")
    .version(env!("CARGO_PKG_VERSION"))
    .about("Server that serves Leo outline and its external files")
    .args_from_usage(
      "<FILE> 'leo outline file'"
    ).get_matches();
  Ok(PathBuf::from(matches.value_of("FILE").unwrap()))
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
fn _test_levgnx() {
  let mut a:LevGnx = 23;
  println!("Ignx {}", a.ignx());
  println!("level {}", a.level());
  a.inc();
  println!("level {}", a.level());
  a.shift(5i8);
  println!("level {}", a.level());
  a.dec();
  println!("level {}", a.level());
  println!("levgnx {}", LevGnx::from_str("01234").to_str());
}
fn _test_parse_leoxml(fname:PathBuf) -> (Outline, Vec<VData>) {
  let f = File::open(&fname).unwrap();
  let mut buf_reader = BufReader::new(f);
  let mut buf = String::new();
  buf_reader.read_to_string(&mut buf).unwrap();
  let config = parser_config();
  let reader = config.create_reader(buf.as_bytes());
  let mut nodes:Vec<VData> = Vec::new();
  nodes.push(VData::new("hidden-root-vnode-gnx"));
  let mut gnx2i:HashMap<String, usize> = HashMap::new();
  let mut names:Vec<String> = Vec::new();
  let mut last_gnx = String::new();
  let mut txt = String::new();
  let mut lev = 0u8;
  let mut gnxcount:usize = 1;
  let mut outline:Outline = vec![0u32];
  for xe in reader.into_iter() {
    match xe {
      Ok(XmlEvent::StartElement { name, attributes, ..}) => {
        let n = name.local_name;
        names.push(n.clone());
        if n == "v" {
          last_gnx.clear();
          last_gnx.push_str(&attributes[0].value);
          let v = VData::new(&last_gnx);
          let ignx = gnx2i.entry(v.gnx.clone()).or_insert(gnxcount);
          lev += 1u8;
          outline.add_node(lev, *ignx as u32);
          nodes.push(v);
          gnxcount += 1;
        } else if n == "vnodes" {
          lev=0;
        } else if n == "t" {
          last_gnx.clear();
          last_gnx.push_str(&attributes[0].value);
        }
        txt.clear();
      },
      Ok(XmlEvent::Characters(t)) => txt.push_str(&t),
      Ok(XmlEvent::EndElement{..}) => {
        let n = names.pop().unwrap();
        if n == "vh" {
          if let Some(i) = gnx2i.get(&last_gnx) {
            nodes[*i].h.push_str(&txt)
          }
        } else if n == "v" {
          lev -= 1;
        } else if n == "t" {
          if let Some(i) = gnx2i.get(&last_gnx) {
            nodes[*i].b.push_str(&txt);
          }
        }
      },
      _ => ()
    }
  }
  (outline, nodes)
}

fn main() {
  let fname = parse_config_from_cmdline().unwrap();
  let (o, nodes) = _test_parse_leoxml(fname);
  let s_lev = ".............................................................";
  for x in o {
    let lev = 2 * x.level() as usize;
    let ignx = x.ignx() as usize;
    let h = &nodes[ignx].h;
    let sz = nodes[ignx].b.len();
    println!("{}{}:[{}]", &s_lev[..lev], h, sz);
  }
  _test_levgnx();
}
