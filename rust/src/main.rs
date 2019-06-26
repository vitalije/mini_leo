extern crate parsing;
use parsing::{ldf_header, rpartition};

fn main() {

  let data = include_bytes!("/tmp/leoGlobals.py"); // or take any other Leo external file
  let datastr = std::str::from_utf8(data).unwrap();
  let res = ldf_header(datastr);
  match res {
    Ok((_, (flines, st, en))) => {
      println!("flines[{}]\nstart=[{}]\nend=[{}]", flines, st, en);
    },
    Err(_) => println!("greska u parsiranju")
  }
  let (a, b, c) = rpartition("proba", "c");
  println!("a:[{}], b:[{}], c:[{}]", a, b, c);
}
