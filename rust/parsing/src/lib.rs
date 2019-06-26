extern crate nom;
use nom::{
  IResult,
  bytes::complete::take_until, //{is_not, is_a, tag, take, take_until, take_while},
  // character::complete::{none_of, one_of, line_ending, char as pchar},
  sequence::pair, //{tuple, pair, preceded},
  //branch::alt,
  // multi::{many_till},
  combinator::map,
  //error::ErrorKind

};
pub fn rpartition<'a>(input:&'a str, sep:&str) -> (&'a str, &'a str, &'a str) {
  match input.rfind(sep) {
    Some(i) => {
      let j = i + sep.len();
      (&input[..i], &input[i..j], &input[j..])
    },
    None => {
      let i = input.len() - 1;
      (&input, &input[i..i], &input[i..i])
    }
  }
}
pub fn ldf_header(input:&str) ->  IResult<&str, (&str, &str, &str)> {
  let w1 = take_until("@+leo-ver=5-thin");
  let w2 = pair(w1, take_until("\n"));
  let w3 = map(w2, |(a, b)| {
    let (f, _, st) = rpartition(a, "\n");
    (f, st, &b[16..])
  });
  w3(input)
}
