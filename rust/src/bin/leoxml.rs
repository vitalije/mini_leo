extern crate xml;
// extern crate sxd_document;
extern crate clap;
#[macro_use]
extern crate error_type;
use std::collections::HashMap;
use clap::App;
//use sxd_document::parser::parse;
//use sxd_document::dom::{Document, Element, Text, ChildOfElement, ChildOfRoot};
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
/*
enum XmlSadrzaj<'a> {
  E(Element<'a>),
  T(Text<'a>)
}
fn iter_root<'a>(doc:&'a Document) -> Vec<XmlSadrzaj<'a>> {
  let mut res:Vec<XmlSadrzaj> = Vec::new();
  fn dump_e<'a>(e:&Element<'a>, res:&mut Vec<XmlSadrzaj<'a>>) {
    res.push(XmlSadrzaj::E(*e));
    for e1 in e.children().iter() {
        match e1 {
          ChildOfElement::Element(e2) => dump_e(e2, res),
          ChildOfElement::Text(t) => res.push(XmlSadrzaj::T(*t)),
          _ => ()
        }
    }
  };
  for e in doc.root().children().iter() {
    match e {
      ChildOfRoot::Element(e1) => dump_e(&e1, &mut res),
      _ => ()
    }
  }
  res
}
fn get_bodies<'a>(doc:&'a Document) -> HashMap<String, String> {
  let mut h:HashMap<String, String> = HashMap::new();
  let mut k = String::new();
  let mut b = String::new();
  for v in iter_root(doc) {
    match v {
      XmlSadrzaj::E(e) if e.name().local_part() == "t" => {
        if !k.is_empty() {
          h.insert(k.clone(), b.clone());
        }
        b.clear();
        k.clear();
        if let Some(a) = e.attribute("tx") {
          k.push_str(a.value());
        }
      },
      XmlSadrzaj::T(t) if !k.is_empty() => {
        b.push_str(t.text());
      },
      _ => if !k.is_empty() {
        h.insert(k.clone(), b.clone());
        k.clear();
      }
    }
  }
  h
}
fn v_gnx_h_children<'a>(e:&'a Element) -> (String, String, String) {
  let gnx = if let Some(x) = e.attribute("t") {
      String::from(x.value())
    } else {
      String::new()
  };
  let mut name = String::new();
  let mut children = String::new();
  let mut h = String::new();
  for e1 in e.children().iter() {
    match e1 {
      ChildOfElement::Element(e2) => {
        name.clear();
        name.push_str(e2.name().local_part());
        if name == "v" {
          if let Some(x) = e2.attribute("t") {
            children.push_str(x.value());
            children.push(' ');
          }
        }
      },
      ChildOfElement::Text(t) if name == "vh" => {
        h.push_str(t.text())
      },
      _ => ()
    }
  }
  children.pop();
  (gnx, h, children)
}
fn main2() {
  let fname = parse_config_from_cmdline().unwrap();
  let f = File::open(&fname).unwrap();
  let mut buf_reader = BufReader::new(f);
  let mut buf = String::new();
  buf_reader.read_to_string(&mut buf).unwrap();
  match parse(&buf) {
    Ok(doc) => {
      println!("uspesno parsiran xml");
      let document = doc.as_document();
      let bodies = get_bodies(&document);
      for (k, v) in bodies {
        println!("------------{}--{}chars-----------------", k, v.len());
        println!("{}======================================", v);
      }
    },
    _ => println!("greska u parsiranju")
  }
}
*/
const B64DIGITS:[char;64] = [
  '0', '1', '2', '3', '4', '5', '6', '7',
  '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
  'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
  'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
  'W', 'X', 'Y', 'Z', '_', 'a', 'b', 'c',
  'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
  'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
  't', 'u', 'v', 'w', 'x', 'y', 'z', '~'
];
const B64VALUES:[u8; 128] = [
  255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
  255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
  255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
  255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
  255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
  255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
    0u8,   1u8,   2u8,   3u8,   4u8,   5u8,   6u8,   7u8,
    8u8,   9u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
  255u8,  10u8,  11u8,  12u8,  13u8,  14u8,  15u8,  16u8,
   17u8,  18u8,  19u8,  20u8,  21u8,  22u8,  23u8,  24u8,
   25u8,  26u8,  27u8,  28u8,  29u8,  30u8,  31u8,  32u8,
   33u8,  34u8,  35u8, 255u8, 255u8, 255u8, 255u8,  36u8,
  255u8,  37u8,  38u8,  39u8,  40u8,  41u8,  42u8,  43u8,
   44u8,  45u8,  46u8,  47u8,  48u8,  49u8,  50u8,  51u8,
   52u8,  53u8,  54u8,  55u8,  56u8,  57u8,  58u8,  59u8,
   60u8,  61u8,  62u8, 255u8, 255u8, 255u8,  63u8, 255u8
];
fn b64str(n:u32) -> String {
  if n == 0 {
    String::from("0")
  } else {
    let mut res = String::new();
    let mut _n = n;
    while _n > 0 {
      res.insert(0, B64DIGITS[(_n & 63) as usize]);
      _n = _n >> 6;
    }
    res
  }
}
fn b64int(a:&str) -> u32 {
  let mut res = 0_u32;
  for i in a.bytes() {
    let k = B64VALUES[(i & 127) as usize];
    if k == 255 { break }
    res = (res << 6) + (k as u32);
  }
  res
}
type LevGnx = u32;
pub trait LevGnxUtils {
  fn level(&self) -> u8;
  fn ignx(&self) -> u32;
  fn inc(&mut self);
  fn dec(&mut self);
  fn shift(&mut self, d: i8);
  fn set_ignx(&mut self, ignx:u32);
  fn to_str(&self) -> String;
  fn from_str(a:&str) -> Self;
}

impl LevGnxUtils for LevGnx {
  fn level(&self) -> u8 {(((*self) >> 18) & 63) as u8}
  fn ignx(&self) -> u32 {(*self) & 0x3ffffu32}
  fn inc(&mut self) {*self += 0x4ffffu32;}
  fn dec(&mut self) {if *self > 0x3ffff {*self -= 0x4ffffu32;}}
  fn shift(&mut self, d: i8) {
    let lev = ((*self >> 18) & 63) as i8 + d;
    *self = (*self & 0x3ffff) | if lev <= 0 { 0 } else { ((lev as u32) << 18)};
  }
  fn set_ignx(&mut self, ignx:u32) {
    *self = (*self & 0xfc0000) | ignx;
  }
  fn to_str(&self) -> String {
    let mut res = b64str(*self);
    while res.len() < 4 {res.insert(0, '0');}
    res
  }
  fn from_str(a:&str) -> LevGnx {
    b64int(&a[..4]) as LevGnx
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
fn _test_parse_leoxml(fname:PathBuf) {
  let f = File::open(&fname).unwrap();
  let mut buf_reader = BufReader::new(f);
  let mut buf = String::new();
  buf_reader.read_to_string(&mut buf).unwrap();
  let config = parser_config();
  let reader = config.create_reader(buf.as_bytes());
  let mut gnx2h:HashMap<String, String> = HashMap::new();
  let mut names:Vec<String> = Vec::new();
  let mut gnxes:Vec<String> = Vec::new();
  let mut txt = String::new();
  let mut lev = 0;
  let s_lev = String::from("...............................................................");
  for xe in reader.into_iter() {
    match xe {
      Ok(XmlEvent::StartElement { name, attributes, ..}) => {
        let n = name.local_name;
        names.push(n.clone());
        if n == "v" {
          lev += 1;
          gnxes.push(attributes[0].value.clone());
        } else if n == "vnodes" {
          lev=0;
        }
        txt.clear();
      },
      Ok(XmlEvent::Characters(t)) => txt.push_str(&t),
      Ok(XmlEvent::EndElement{..}) => {
        let n = names.pop().unwrap();
        if n == "vh" {
          let x = gnxes.pop().unwrap();
          println!("{}{}:{}", s_lev.get(0..lev * 2).unwrap(), txt, x);
          gnx2h.insert(x , txt.clone());
        } else if n == "v" {
          lev -= 1;
        }
      },
      _ => ()
    }
  }
}
/*
struct Vnode {
  gnx: String,
  h: String,
  b: String,
  flags: u16
}
*/

fn main() {
  let fname = parse_config_from_cmdline().unwrap();
  _test_parse_leoxml(fname);
  _test_levgnx()
}
