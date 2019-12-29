use super::model::{VData, Outline, OutlineOps, LevGnx, LevGnxOps};
//use xml::reader::{ParserConfig, XmlEvent};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use quick_xml::events::attributes::Attributes;

use std::{
  error::Error as StdError,
  io,
  io::{Read, BufReader},
  fs::File,
  path::{PathBuf},
  collections::{HashMap}
};

#[cfg(test)]
mod tests {
  #[test]
  fn test_handle_level_stars() {
    let s = b"#@+node:ekr.20050208101229: ** << imports  (leoGlobals)";
    assert_eq!(super::handle_level_stars(s, 28), (2, 31));
    let s = b"#@+node:ekr.20050208101229: *4* << imports  (leoGlobals)";
    assert_eq!(super::handle_level_stars(s, 28), (4, 32));
    let s = b"#@+node:ekr.20050208101229: *14* << imports  (leoGlobals)";
    assert_eq!(super::handle_level_stars(s, 28), (14, 33));
    let s = b"#@+node:ekr.20050208101229: * << imports  (leoGlobals)";
    assert_eq!(super::handle_level_stars(s, 28), (1, 30))
  }
}
struct LdfParseState<'a> {
  ind:usize,
  st:&'a str,
  en:&'a str,
  buf:&'a [u8],
  indents: Vec<usize>,
  mark:usize,
  in_doc:bool,
  path: Vec<usize>,
  first_start: usize,
  first_end: usize,
  last_start: usize,
  // in_raw:bool,
  in_all:bool
}
type NodesBuf = Vec<(usize, usize, usize, usize, usize)>;
type LinesBuf = Vec<(usize, usize, usize, Option<(&'static str, &'static str)>)>;

/// handles a line in derived file
/// if it is ordinary line appends its start, end to the
/// current body
/// else if it is special leo sentinel
/// handles it by changing state or adding a node
fn handle_line<'a>(state:&mut LdfParseState<'a>,
                       nodes:&mut NodesBuf,
                       lines:&mut LinesBuf){
  let i0 = state.ind;
  let a = afterws(state.buf, i0);

  if state.buf[a..].starts_with(state.st.as_bytes()) &&
    state.buf[(a + state.st.len())] == b'@' {
      // it is a leo sentinel
      state.ind = a + state.st.len() + 1;
      if state.in_all {
        handle_leo_line_in_all(state, nodes, lines)
      } else {
        handle_leo_line(state, a - i0, nodes, lines)
      }
  } else {
    state.ind = afternl(state.buf, a);
    if state.in_doc {
      let a = i0 + state.st.len() + 1;
      let b = state.ind - 1 - state.en.len();
      push_body_line(state, a, b, Some(("", "\n")), lines);
    } else {
      let a = state.ind;
      push_body_line(state, i0, a, None, lines);
    }
  }
}
fn afterws(buf:&[u8], i:usize) -> usize {
  let mut j = i;
  let n = buf.len() - 1;
  while j < n && buf[j] == b' ' {
    j += 1;
  }
  j
}
fn tonl(buf:&[u8], i:usize) -> usize {
  let mut j = i;
  let n = buf.len() - 1;
  while j < n && buf[j] != b'\n' {
    j += 1;
  }
  j
}
fn tocolon(buf:&[u8], i:usize) -> usize {
  let mut j = i;
  let n = buf.len() - 1;
  while j < n && buf[j] != b':' {
    j += 1;
  }
  j
}
// fn aftercolon(buf:&[u8], i:usize) -> usize { tocolon(buf, i) + 1 }
fn tocloseref(buf:&[u8], i:usize) -> usize {
  let mut j = i;
  let n = buf.len() - 1;
  while j < n && !buf[j..].starts_with(b">>") {
    j += 1;
  }
  j
}
// fn aftercloseref(buf:&[u8], i:usize) -> usize { tocloseref(buf, i) + 2 }
/* fn to_start_of_bytes(buf:&[u8], i:usize, s:&[u8]) -> usize {
  let mut j = i;
  let n = buf.len() - 1;
  while j < n && !buf[j..].starts_with(s) {
    j += 1;
  }
  j
}
*/
fn afternl(buf:&[u8], i:usize) -> usize { tonl(buf, i) + 1}
fn push_body_line<'a>(state:&mut LdfParseState<'a>,
  a: usize,
  b: usize,
  e: Option<(&'static str, &'static str)>,
  lines:&mut LinesBuf) {
  if a > b {panic!("push_body_line a > b {} > {}", a, b);}
  let ni = *state.path.last().unwrap();
  let wi = *state.indents.last().unwrap();
  if b - a == 1 {
    lines.push((ni, a, b, e))
  } else {
    lines.push((ni, a + wi, b, e))
  }
}
fn handle_leo_line<'a>(state:&mut LdfParseState<'a>,
                       ws:usize,
                       nodes:&mut NodesBuf,
                       lines:&mut LinesBuf){
  let _ =  check_at_plus_node(state, nodes, lines)
        || check_at_plus_others(state, ws, nodes, lines)
        || check_at_plus_ref(state, ws, nodes, lines)
        || check_at_minus_others(state, nodes, lines)
        || check_at_minus_ref(state, nodes, lines)
        || check_at_plus_at(state, lines)
        || check_at_plus_all(state, ws, nodes, lines)
        || check_at_first(state, lines)
        || check_at_verbatim(state, lines)
        || check_at_raw(state, lines)
        || check_leo_directives(state, ws, lines)
        || check_ignored_sentinels(state)
        || unknown_leo_sentinel(state);
}
fn handle_leo_line_in_all<'a>(state:&mut LdfParseState<'a>,
                       nodes:&mut NodesBuf,
                       lines:&mut LinesBuf){
   let _ = false
        || check_at_verbatim(state, lines)
        || check_at_plus_node(state, nodes, lines)
        || check_at_minus_all(state, nodes, lines);
}
fn unknown_leo_sentinel(state:&LdfParseState) -> bool {
  let a = state.ind - 2;
  let b = tonl(state.buf, a);
  let _s = std::str::from_utf8(&state.buf[a..b]).unwrap();
  //println!("unknown leo sentinel:[{}] at index:{}", _s, state.ind);
  false //panic!("error");
}
fn check_at_plus_node<'a>(state:&mut LdfParseState<'a>,
                       nodes:&mut NodesBuf,
                       _lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"+node:") {
    let a = i0 + 6;
    let b = tocolon(state.buf, a);
    let (lev, c) = handle_level_stars(state.buf, b + 2);
    let d = tonl(state.buf, c);
    nodes.push((lev, a, b, c, d - state.en.len()));
    if let Some(ni) = state.path.last_mut() {
      *ni = nodes.len();
    }
    state.ind = d + 1;
    state.in_doc = false;
    true
  } else {
    false
  }
}
fn handle_level_stars(buf:&[u8], i:usize) -> (usize, usize) {
  if buf[i + 1] == b'*' {
    // i - star; i + 1 star; i + 2 space
    (2, i + 3)
  } else if buf[i + 1] == b' ' {
    (1, i + 2)
  } else {
    let mut j = i + 1;
    let mut k = 0;
    while buf[j] != b'*' {
      k = 10 * k + ((buf[j] - b'0') as usize);
      j += 1;
    }
    (k, j + 2)
  }
}
fn check_at_minus_others<'a>(state:&mut LdfParseState<'a>,
                       _nodes:&mut NodesBuf,
                       _lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"-others") {
    state.ind = afternl(state.buf, i0 + 7);
    state.path.pop();
    state.indents.pop();
    state.in_doc = false;
    true
  } else {
    false
  }
}
fn check_at_minus_ref<'a>(state:&mut LdfParseState<'a>,
                       _nodes:&mut NodesBuf,
                       lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"-<<") {
    let i1 = afternl(state.buf, i0 + 4);
    let i2 = afterws(state.buf, i1);
    let i3 = i2 + state.st.len();
    state.path.pop();
    state.indents.pop();
    let ni = state.path.last().unwrap();
    if state.buf[i2..].starts_with(state.st.as_bytes()) &&
      state.buf[i3..].starts_with(b"@afterref") {
      let i4 = afternl(state.buf, i3 + 9); // after sentinel
      let i5 = afternl(state.buf, i4); // line of text
      lines.push((*ni, i4, i5, None));
      state.ind = i5;
    } else {
      lines.push((*ni, i1 - 1,  i1, None));
      state.ind = i1;
    }
    state.in_doc = false;
    true
  } else {
    false
  }
}
fn check_at_plus_ref<'a>(state:&mut LdfParseState<'a>,
                       ws:usize,
                       nodes:&mut NodesBuf,
                       lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"+<<") {
    let a = i0 + 1;
    let b = tocloseref(state.buf, a + 2);
    let ni = *(state.path.last().unwrap());
    let wi = *(state.indents.last().unwrap());
    let d = a - 2 - state.st.len();
    let c = d - ws + wi;
    lines.push((ni, c, d, None));
    lines.push((ni, a, b + 2, None));
    state.indents.push(ws);
    state.path.push(0);
    state.ind = afterws(state.buf, afternl(state.buf, state.ind));
    check_at_plus_node(state, nodes, lines);
    state.in_doc = false;
    true
  } else {
    false
  }
}
fn check_at_plus_others<'a>(state:&mut LdfParseState<'a>,
                       ws:usize,
                       nodes:&mut NodesBuf,
                       lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"+others") {
    let i1 = i0 - state.st.len() - 1 - ws;
    push_body_line(state, i1, i1 + ws, Some(("", "@others\n")), lines);
    state.indents.push(ws);
    state.path.push(0);
    state.ind = afterws(state.buf, afternl(state.buf, state.ind));
    check_at_plus_node(state, nodes, lines);
    state.in_doc = false;
    true
  } else {
    false
  }
}
fn check_at_plus_all<'a>(state:&mut LdfParseState<'a>,
                       ws:usize,
                       nodes:&mut NodesBuf,
                       lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"+all") {
    let i1 = i0 - state.st.len() - 1 - ws;
    push_body_line(state, i1, i1 + ws, Some(("", "@all\n")), lines);
    state.indents.push(ws);
    state.path.push(0);
    state.ind = afterws(state.buf, afternl(state.buf, state.ind));
    check_at_plus_node(state, nodes, lines);
    state.in_doc = false;
    state.in_all = true;
    true
  } else {
    false
  }
}
fn check_at_minus_all<'a>(state:&mut LdfParseState<'a>,
                       _nodes:&mut NodesBuf,
                       _lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"-all") {
    state.ind = afternl(state.buf, i0 + 4);
    state.path.pop();
    state.indents.pop();
    state.in_doc = false;
    state.in_all = false;
    true
  } else {
    false
  }
}
fn check_leo_directives(state:&mut LdfParseState, ws:usize, lines:&mut LinesBuf) -> bool {
  if state.buf[state.ind] == b'@' {
    if state.buf[state.ind..].starts_with(b"@c") || state.buf[state.ind..].starts_with(b"@code") {
      state.in_doc = false;
    }
    let c = tonl(state.buf, state.ind);
    let a = state.ind - ws;
    let b = c - state.en.len();
    push_body_line(state, a, b, Some(("", "\n")), lines);
    state.ind = c + 1;
    true
  } else { false }
}
fn check_at_verbatim(state:&mut LdfParseState, lines:&mut LinesBuf) -> bool {
  if state.buf[state.ind..].starts_with(b"verbatim") {
    let a = afternl(state.buf, state.ind + 8);
    let b = afternl(state.buf, a);
    let ni = *(state.path.last().unwrap());
    lines.push((ni, a, b, None));
    state.ind = b;
    true
  } else { false }
}
fn check_at_raw(state:&mut LdfParseState, lines:&mut LinesBuf) -> bool {
  if state.buf[state.ind..].starts_with(b"verbatim") {
    let a = afternl(state.buf, state.ind + 9);
    let b = afternl(state.buf, a);
    let ni = *(state.path.last().unwrap());
    lines.push((ni, a, b, None));
    state.ind = b;
    true
  } else { false }
}
fn check_ignored_sentinels(state:&mut LdfParseState) -> bool {
  let a = state.buf[state.ind..].starts_with(b"@first")
      ||  state.buf[state.ind..].starts_with(b"@first")
      ||  state.buf[state.ind..].starts_with(b"@first")
      ||  state.buf[state.ind..].starts_with(b"@first");
  if a {
    state.ind = afternl(state.buf, state.ind);
  }
  a
}
fn check_at_first<'a>(state:&mut LdfParseState<'a>, lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  if state.buf[i0..].starts_with(b"@first"){
    let i = state.first_start;
    let j = afternl(state.buf, i);
    state.first_start = j;
    lines.push((2, i, j, Some(("@first ", ""))));
    state.ind = afternl(state.buf, i0 + 6);
    true
  } else {
    false
  }
}
fn check_at_plus_at<'a>(state:&mut LdfParseState<'a>, lines:&mut LinesBuf) -> bool {
  let i0 = state.ind;
  let f1 = state.buf[i0..].starts_with(b"+at");
  let f2 = state.buf[i0..].starts_with(b"+doc");
  if f1 || f2 {
    let a = if f1 { i0 + 3} else {i0 + 4};
    let f3 = state.en.len() > 0 && state.buf[a..].starts_with(state.en.as_bytes());
    let f4 = state.en.len() == 0 && state.buf[a] == b'\n';
    let ni = *(state.path.last().unwrap());
    if f3 || f4 {
      let opt = if f1 { Some(("@", "")) } else { Some(("@doc", "")) };
      let b = a + state.en.len();
      push_body_line(state, b, b + 1, opt, lines);
      state.ind = b + 1;
    } else {
      let opt = if f1 { Some(("@ ", "\n")) } else { Some(("@doc ", "\n")) };
      let b = tonl(state.buf, a) - state.en.len();
      lines.push((ni, a + 1, b, opt));
      state.ind = b + 1;
    }
    if state.en.len() != 0 {
      let wi = state.indents.last().unwrap();
      let mut a = afternl(state.buf, state.ind) + wi;
      let mut i = a;
      while !state.buf[i..].starts_with(state.en.as_bytes()) {
        if state.buf[i] == b'\n' {
          lines.push((ni, a, i+1, None));
          i += wi + 1;
          a = i;
        } else {
          i += 1;
        }
      }
      state.ind = afternl(state.buf, i);
      state.in_doc = false;
    } else {
      state.in_doc = true;
    }
    true
  } else {
    false
  }
}
fn handle_leo_header<'a>(txt:&'a str) -> LdfParseState<'a> {
  let mut state = LdfParseState {
    ind: 0,
    st: "",
    en: "",
    buf: txt.as_bytes(),
    indents: vec![0],
    mark:0,
    in_doc: false,
    path: vec![0],
    first_start:0,
    first_end: 0,
    last_start:txt.len(),
    in_all:false,
    // in_raw:false
  };
  let n = txt.len();
  while state.ind < n {
    if state.buf[state.ind] == b'\n' {
      state.ind += 1;
      state.mark = state.ind;
    } else if state.buf[state.ind..].starts_with(b"@+leo-ver=5-thin") {
      state.first_end = state.mark;
      state.st = &txt[state.mark..state.ind];
      let a = state.ind + 16;
      state.ind = tonl(state.buf, a);
      state.en = &txt[a..state.ind];
      state.ind += 1;
      break;
    } else {
      state.ind += 1;
    }
  }
  state
}
pub fn ldf_parse<'a>(txt:&'a str) -> (NodesBuf, LinesBuf, usize, usize) {
  let mut nodes:NodesBuf = vec![(0,0,0,0,0)];
  let mut lines:LinesBuf = Vec::new();
  let mut state = handle_leo_header(txt);
  let n = txt.len();
  while state.ind < n {
    if is_at_minus_leo(&mut state) && !state.in_all{ break }
    handle_line(&mut state, &mut nodes, &mut lines)
  }
  (nodes, lines, state.first_end, state.last_start)
}
fn is_at_minus_leo<'a>(state:&mut LdfParseState) -> bool {
  let a = state.ind;
  let b = state.ind + state.st.len();
  if state.buf[a..].starts_with(state.st.as_bytes()) && state.buf[b..].starts_with(b"@-leo") {
    state.last_start = afternl(state.buf, b + 5);
    state.ind = state.last_start;
    true
  } else { false}
}
pub fn from_derived_file(fname:PathBuf) -> Result<(Outline, Vec<VData>), io::Error> {
  let f = File::open(&fname)?;
  let mut buf_reader = BufReader::new(f);
  let mut buf = String::new();
  buf_reader.read_to_string(&mut buf)?;
  Ok(from_derived_file_content(&buf))
}
pub fn from_derived_file_content(content:&str) -> (Outline, Vec<VData>) {
  let mut nodes = Vec::new();
  let mut outline = Vec::new();
  let (vnodes, lines, _, _) = ldf_parse(content);
  let mut i:u32 = 0;
  // TODO: consider changing ldf_parse to skip root node in its output nodes
  // if it skips root node, in the following loop we won't have to check if lev > 0
  // and root node can be inserted in nodes before loop
  for (lev, a, b, c, d) in vnodes {
    if lev > 0 {
      let mut v = VData::new(&content[a..b]);
      v.h.push_str(&content[c..d]);
      nodes.push(v);
      let mut x:LevGnx = i;
      x.shift(lev as i8);
      outline.push(x);
    } else {
      let mut v = VData::new("hidden-root-vnode-gnx");
      v.h.push_str("<hidden root vnode>");
      nodes.push(v)
    }
    i += 1;
  }
  for (i, a, b, op) in lines {
    let v = nodes.get_mut(i-1).unwrap();
    match op {
      Some((pref, suf)) => {
        v.b.push_str(pref);
        v.b.push_str(&content[a..b]);
        v.b.push_str(suf);
      },
      _ => v.b.push_str(&content[a..b])
    }
  }
  (outline, nodes)
}
/*
fn parser_config() -> ParserConfig {
  ParserConfig::new()
      .cdata_to_characters(true)
      .whitespace_to_characters(true)
}
*/
pub fn from_leo_file(fname:PathBuf) -> Result<(Outline, Vec<VData>), io::Error> {
  let f = File::open(&fname)?;
  let mut buf_reader = BufReader::new(f);
  let mut buf = String::new();
  buf_reader.read_to_string(&mut buf)?;
  Ok(from_leo_content(&buf))
}
pub fn from_leo_content(buf:&str) -> (Outline, Vec<VData>) {
  //let config = parser_config();
  //let reader = config.create_reader(buf.as_bytes());
  let mut reader = XmlReader::from_str(buf);
  let mut nodes:Vec<VData> = Vec::new();
  nodes.push(VData::new("hidden-root-vnode-gnx"));
  let mut gnx2i:HashMap<String, usize> = HashMap::new();
  let mut last_gnx = String::new();
  let mut txt = String::new();
  let mut lev = 0u8;
  let mut gnxcount:usize = 1;
  let mut outline:Outline = vec![0u32];
  loop {
    let mut xmlbuf = Vec::new();
    let getattr = |k:&[u8], attrs:Attributes, rr| {
      for x in attrs {
        let a = x.unwrap();
        if a.key == k {
          return a.unescape_and_decode_value(rr).unwrap();
        }
      }
      panic!("missing attribute:{:?}", k);
    };
    match reader.read_event(&mut xmlbuf) {
      Ok(Event::Start(ref e)) => {
        let n = e.local_name();
        if n == b"v" {
          last_gnx.clear();
          last_gnx.push_str(&getattr(b"t", e.attributes(), &reader));
          let v = VData::new(&last_gnx);
          let ignx = gnx2i.entry(v.gnx.clone()).or_insert(gnxcount);
          lev += 1u8;
          outline.add_node(lev, *ignx as u32);
          nodes.push(v);
          gnxcount += 1;
        } else if n == b"vnodes" {
          lev=0;
        } else if n == b"t" {
          last_gnx.clear();
          last_gnx.push_str(&getattr(b"tx", e.attributes(), &reader));
        }
        txt.clear();
      },
      Ok(Event::Text(e)) => txt.push_str(&e.unescape_and_decode(&reader).unwrap()),
      Ok(Event::End(ref e)) => {
        let n = e.local_name();
        if n == b"vh" {
          if let Some(i) = gnx2i.get(&last_gnx) {
            nodes[*i].h.push_str(&txt)
          }
        } else if n == b"v" {
          lev -= 1;
        } else if n == b"t" {
          if let Some(i) = gnx2i.get(&last_gnx) {
            nodes[*i].b.push_str(&txt);
          }
        }
      },
      Ok(Event::Eof) => break,
      _ => ()
    }
  }
  (outline, nodes)
}
