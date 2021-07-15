/// converts integer to String in base 64
pub fn b64str(n:u64) -> String {
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
#[allow(dead_code)]
/// appends textual representation of u64 number to
/// the given string buffer
pub fn b64write(n:u64, buf:&mut String) {
  if n == 0 {
    buf.push('0');
  } else {
    let mut _n = n;
    let i = buf.len();
    while _n > 0 {
      buf.insert(i, B64DIGITS[(_n & 63) as usize]);
      _n = _n >> 6;
    }
  }
}

/// converts base 64 str to u64
pub fn b64int(a:&str) -> u64 {
  let mut res = 0_u64;
  for i in a.bytes() {
    let k = B64VALUES[(i & 127) as usize];
    if k == 255 { break }
    res = (res << 6) + (k as u64);
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
#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn partition<'a>(input:&'a str, sep:&str) -> (&'a str, &'a str, &'a str) {
  match input.find(sep) {
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
#[allow(dead_code)]
pub fn extract_section_ref(x:&str) -> Option<&str> {
    x.find("<<")
      .map(|i|{
        x.find(">>").map(|j|{(i, j)})
      }).flatten()
      .filter(|(i, j)|i + 3 < *j)
      .map(|(i, j)| &x[i..j+2])
}
#[allow(dead_code)]
pub fn is_directive(t:&str) -> bool {
  t.starts_with("@language") ||
  t.starts_with("@nocolor") ||
  t.starts_with("@killcolor") ||
  t.starts_with("@tabwidth") ||
  t.starts_with("@beautify") ||
  t.starts_with("@nobeautify") ||
  t.starts_with("@killbeautify") ||
  t.starts_with("@nopyflakes") ||
  t.starts_with("@linending") ||
  t.starts_with("@wrap") ||
  t.starts_with("@nowrap") ||
  t.starts_with("@encoding")
}
#[allow(dead_code)]
pub fn is_special(t:&str) -> bool {
  t.trim_start().starts_with("@others") || extract_section_ref(t).is_some()
}
#[allow(dead_code)]
pub fn others_index(t:&str) -> usize {
  if t.starts_with("@others") { return 0 }
  for (i, _) in t.match_indices("@others") {
    if let Some(j) = t[0..i].rfind('\n') {
      return j;
    }
  }
  t.len() + 100
}
#[allow(dead_code)]
pub fn has_others(t:&str) -> bool {
  others_index(t) < t.len()
}
#[allow(dead_code)]
pub fn insert_parts(inp:&mut Vec<u64>, marks:&Vec<usize>, data:&Vec<u64>) {
  let size = data.len()/marks.len();
  make_gaps(inp, marks, size);
  for (i, j) in marks.iter().enumerate() {
    let mut a = (*j as usize) + i * size;
    for k in &data[i*size..(i+1)*size] {
      inp[a] = *k;
      a += 1;
    }
  }
}
#[allow(dead_code)]
pub fn make_gaps(inp:&mut Vec<u64>, marks:&Vec<usize>, sz:usize) {
    let mut space = marks.len() * sz;
    inp.reserve(space);
    let mut count = inp.len();
    unsafe { inp.set_len(space + count); }
    for i in marks.iter().rev() {
        if count != *i {
          unsafe {
              let src = inp.as_mut_ptr().add(*i);
              let dst = src.add(space);
              std::ptr::copy(src, dst, count - *i);
          }
        }
        count = *i;
        space -= sz;
    }
}
#[allow(dead_code)]
pub fn delete_blocks(inp:&mut Vec<u64>, marks:&Vec<usize>, sz:usize) {
    let count = inp.len();
    let mut delta = 0;
    for (j, i) in marks.iter().enumerate() {
      unsafe {
        let src = inp.as_mut_ptr().add(*i + sz);
        let dst = inp.as_mut_ptr().add(*i - delta);
        let cnt = *marks.get(j+1).unwrap_or(&count) - *i;
        std::ptr::copy(src, dst, cnt);
        delta += sz;
      }
    }
    inp.truncate(count-delta);
}
#[cfg(test)]
mod tests {
  #[test]
  fn test_rpartition() {
    let s = "asepbsepc";
    let res = super::rpartition(s, "sep");
    assert_eq!(res, ("asepb", "sep", "c"));
    let res2 = super::rpartition(s, "d");
    assert_eq!(res2, ("asepbsepc", "", ""));
    let res3 = super::rpartition(s, "a");
    assert_eq!(res3, ("", "a", "sepbsepc"));
  }

  #[test]
  fn test_make_gaps() {
    let mut v1:Vec<u64> = vec![1, 3, 5, 7, 9];
    let marks = vec![1usize,2, 3, 4, 5];
    super::make_gaps(&mut v1, &marks, 1);
    for (i, j) in marks.iter().enumerate() {
      v1[i+(*j as usize)] = (i * 2 + 2) as u64;
    }
    assert_eq!(v1, [1,2,3,4,5,6,7,8,9,10]);
  }
  #[test]
  fn test_insert_parts() {
    let mut v1:Vec<u64> = vec![1, 5, 9, 13];
    let marks = vec![1usize,2, 3, 4];
    let data = vec![2u64, 3, 4, 6, 7, 8, 10, 11, 12, 14, 15, 16];
    super::insert_parts(&mut v1, &marks, &data);
    assert_eq!(v1, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
  }
  #[test]
  fn test_delete_blocks() {
    let mut v1:Vec<u64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let marks = vec![1usize,5, 9, 13];
    super::delete_blocks(&mut v1, &marks, 3);
    assert_eq!(v1, [1, 5, 9, 13]);
  }
}
