#[path="../parsing.rs"]
mod parsing;
use std::env::args;
use parsing::{ldf_parse,from_derived_file_content};
use std::io;
use std::io::prelude::*;
use std::fs::File;
fn benchmark(datastr:&str) {
  let mut n = 0;
  let t1 = Instant::now();
  for i in 0..100 {
     let (o, nodes) = from_derived_file_content(datastr);
     n += o.len() + nodes.len();
  }
  if n < 100 {println!("less than 100 lines and nodes")}
  println!("{} ms", t1.elapsed().as_millis());
}
use std::time::{Instant};
fn main() -> io::Result<()> {
  let mut buffer = String::new();
  for (i, a) in args().enumerate() {
    if i == 1 {
      let f = File::open(a)?;
      let mut reader = io::BufReader::new(f);
      buffer.clear();
      reader.read_to_string(&mut buffer)?;
    }
  }
  benchmark(&buffer);
  Ok(())
}
