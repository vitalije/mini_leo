extern crate nom;
mod parsing;
mod model;
use std::env::args;
use parsing::{ldf_parse,from_derived_file_content};
use std::io;
use std::io::prelude::*;
use std::fs::File;
type BufNode = (usize, usize, usize, usize, usize);
type BufLine = (usize, usize, usize, Option<(&'static str, &'static str)>);

fn pr_bufline(s:&BufLine) {
  let (ni, a, b, op) = *s;
  let pref_suf = match op {
    Some((pref, suf)) =>  format!("\"{}\", \"{}\"", pref.escape_debug(), suf.escape_debug()),
    _ => String::from("null, null")
  };
  print!("[{}, {}, {}, {}]", ni, a, b, &pref_suf);
}
fn pr_bufnode(s:&BufNode) {
  let (a, b, c, d, e) = *s;
  print!("[{}, {}, {}, {}, {}]", a, b, c, d, e);
}
fn pr_ldf(datastr:&str) {
  let (nodes, lines, _first_end, _last_start) = ldf_parse(datastr);
  println!("{}\n \"nodes\": [", "{");
  let n = nodes.len() - 1;
  for (i, x) in nodes.iter().enumerate() {
    print!("    ");
    pr_bufnode(x);
    if i != n {println!(",")}
  }
  println!("],\n  \"lines\": [");
  let n = lines.len() - 1;
  for (i, x) in lines.iter().enumerate() {
    print!("    ");
    pr_bufline(x);
    if i != n {println!(",")}
  }
  println!("]\n{}", "}");
}

fn main() -> io::Result<()>{
  let mut buffer = String::new();
  let mut kind:usize = 0;
  for (i, a) in args().enumerate() {
    if i == 1 {
      let f = File::open(a)?;
      let mut reader = io::BufReader::new(f);
      buffer.clear();
      reader.read_to_string(&mut buffer)?;
    } else if i == 2 {
      kind = a.parse().expect("Not a number");
    }
  }
  if kind == 0 {
    pr_ldf(&buffer);
  } else {
    let (o, nodes) = from_derived_file_content(&buffer);
    if let Some(v) = nodes.get(kind) {
      println!("head:{}\ngnx:{}\nbody:{}", v.h, v.gnx, v.b);
    }
  }
  Ok(())
}
