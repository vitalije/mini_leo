extern crate xml;
extern crate clap;
#[macro_use]
extern crate error_type;
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
/// converts integer to String in base 64
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

/// converts base 64 str to u32
fn b64int(a:&str) -> u32 {
  let mut res = 0_u32;
  for i in a.bytes() {
    let k = B64VALUES[(i & 127) as usize];
    if k == 255 { break }
    res = (res << 6) + (k as u32);
  }
  res
}
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
type LevGnx = u32;
pub trait LevGnxUtils {

  /// returns level of this object
  fn level(&self) -> u8;

  /// returns ignx of this object
  fn ignx(&self) -> u32;

  /// increments level of this object
  fn inc(&mut self);

  /// decrements level of this object
  fn dec(&mut self);

  /// changes the level of this object for given delta d
  fn shift(&mut self, d: i8);

  /// sets ignx of this object to given value
  fn set_ignx(&mut self, ignx:u32);

  /// converts this object into ascii representation (4 ascii letters)
  fn to_str(&self) -> String;

  /// creates object from its String representation
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
type Outline = Vec<LevGnx>;

pub trait OutlineOps {
  /// returns true if this outline conains a node with given ignx
  fn has(&self, ignx:u32) -> bool;

  /// returns the index of the first node with the given ignx
  /// if the node can't be found returns -1
  fn find(&self, ignx:u32) -> i64;

  /// returns suboutline of the node with the given ignx
  /// if such a node dosen't exist returns an empty outline
  fn subtree(&self, ignx:u32) -> Self;

  /// appends a node with the given ignx at the given level.
  /// Returns true if the node is clone and its subtree has been added too;
  /// otherwise returns false.
  fn add_node(&mut self, level: u8, ignx: u32) -> bool;

  /// returns the index of the parent node
  fn parent_index(&self, i: usize) -> usize;

  /// returns the size of the subtree starting at given index
  fn subtree_size(&self, i:usize) -> usize;

  /// Given the node located at the given outline index i
  /// this method returns the child index that this node
  /// has in its parent's list of children
  fn child_index(&self, i:usize) -> usize;

}
impl OutlineOps for Outline {
  fn has(&self, ignx:u32) -> bool { self.find(ignx) > -1}
  fn find(&self, ignx:u32) -> i64 {
    let mut i = 0i64;
    for x in self {
      if (x & 0x3ffff) == ignx {return i}
      i += 1;
    }
    -1
  }
  fn subtree(&self, ignx:u32) -> Outline {
    let mut res:Outline = Vec::new();
    let j = self.find(ignx);
    if j < 0 { return res }
    let mut i = j as usize;
    let z = self[i];
    let zlev:u8 = z.level();
    let delta:i8 = -(zlev as i8);
    res.push(z.ignx());
    let n = self.len();
    while i + 1 < n {
      i += 1;
      let mut z = self[i];
      if z.level() <= zlev { break };
      z.shift(delta);
      res.push(z);
    }
    res
  }
  fn add_node(&mut self, level: u8, ignx: u32) -> bool {
    let max_level:u8 = match self.last() {
      Some(z) => z.level() + 1,
      None => 0
    };
    if level > max_level {
      panic!("trying to add a node to level {} when max_level is {}", level, max_level);
    }
    let st = self.subtree(ignx);
    if st.is_empty() {
      self.push(((level as u32) << 18) | ignx);
      false
    } else {
      for x in st {
        let mut z = x;
        z.shift(level as i8);
        self.push(z);
      }
      true
    }
  }
  fn parent_index(&self, i: usize) -> usize {
    return match self.get(i) {
      Some(z) => if z.level() < 2 {
          0
        } else {
          let mut j = i - 1;
          let lev = z.level() - 1;
          while self[j].level() != lev {j -= 1}
          j
        },
      None => 0
    }
  }
  fn subtree_size(&self, i:usize) -> usize {
    let z:u32 = self[i] & 0xffc_0000;
    for (j, x) in self[i..].iter().enumerate() {
      if z > *x {return j + 1}
    }
    self.len() - i
  }
  fn child_index(&self, i:usize) -> usize {
    let pi = self.parent_index(i);
    let mut n:usize = 0;
    let lev = self[i].level();
    for z in &self[pi..i] {
      if z.level() == lev {n += 1}
    }
    n
  }
}
struct VData {
  gnx: String,
  h: String,
  b: String,
  flags: u16
}
pub trait VDataOps {
  fn new(gnx:&str) -> Self;
  fn is_expanded(&self) -> bool;
}
impl VDataOps for VData {
  fn new(_gnx:&str) -> VData {
    VData {
      gnx: String::from(_gnx),
      h:String::new(),
      b:String::new(),
      flags:0
    }
  }
  fn is_expanded(&self) -> bool {self.flags & 1 == 1}
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
