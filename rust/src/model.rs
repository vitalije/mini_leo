#[path="utils.rs"]
mod utils;
use self::utils::{b64str, b64int};
use std::collections::HashMap;
use pyo3::prelude::*;
pub type LevGnx = u32;
pub trait LevGnxOps {
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
  fn from_str(a:&str) -> LevGnx;
}
impl LevGnxOps for LevGnx {

  /// returns level of this object
  fn level(&self) -> u8 {(((*self) >> 18) & 63) as u8}

  /// returns ignx of this object
  fn ignx(&self) -> u32 {(*self) & 0x3ffffu32}

  /// increments level of this object
  fn inc(&mut self) {*self += 0x4ffffu32;}

  /// decrements level of this object
  fn dec(&mut self) {if *self > 0x3ffff {*self -= 0x4ffffu32;}}

  /// changes the level of this object for given delta d
  fn shift(&mut self, d: i8) {
    let lev = ((*self >> 18) & 63) as i8 + d;
    *self = (*self & 0x3ffff) | if lev <= 0 { 0 } else { ((lev as u32) << 18)};
  }

  /// sets ignx of this object to given value
  fn set_ignx(&mut self, ignx:u32) {
    *self = (*self & 0xfc0000) | ignx;
  }

  /// converts this object into ascii representation (4 ascii letters)
  fn to_str(&self) -> String {
    let mut res = b64str(*self);
    while res.len() < 4 {res.insert(0, '0');}
    res
  }

  /// creates object from its String representation
  fn from_str(a:&str) -> LevGnx {
    b64int(&a[..4]) as LevGnx
  }
}
pub type Outline = Vec<u32>;

pub trait OutlineOps {
  /// returns true if this outline conains a node with given ignx
  fn has(&self, ignx:u32) -> bool;

  /// returns the index of the first node with the given ignx
  /// if the node can't be found returns -1
  fn find(&self, ignx:u32) -> i64;

  /// returns suboutline of the node with the given ignx
  /// if such a node dosen't exist returns an empty outline
  fn subtree(&self, ignx:u32) -> Outline;

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
  /// returns true if this outline conains a node with given ignx
  fn has(&self, ignx:u32) -> bool { self.find(ignx) > -1}
  /// returns the index of the first node with the given ignx
  /// if the node can't be found returns -1
  fn find(&self, ignx:u32) -> i64 {
    let mut i = 0i64;
    for x in self {
      if (x & 0x3ffff) == ignx {return i}
      i += 1;
    }
    -1
  }
  /// returns suboutline of the node with the given ignx
  /// if such a node dosen't exist returns an empty outline
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
  /// appends a node with the given ignx at the given level.
  /// Returns true if the node is clone and its subtree has been added too;
  /// otherwise returns false.
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
  /// returns the index of the parent node
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
  /// returns the size of the subtree starting at given index
  fn subtree_size(&self, i:usize) -> usize {
    let z:u32 = self[i] & 0xffc_0000;
    for (j, x) in self[i..].iter().enumerate() {
      if z > *x {return j + 1}
    }
    self.len() - i
  }
  /// Given the node located at the given outline index i
  /// this method returns the child index that this node
  /// has in its parent's list of children
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
/// returns a map of ignx -> gnx
fn gnx_index(nodes:&Vec<VData>) -> HashMap<u32, String> {
  let mut res = HashMap::new();
  for (i, x) in nodes.iter().enumerate() {
    res.insert(i as u32, x.gnx.clone());
  }
  res
}
#[pyclass]
pub struct VData {
  #[pyo3(get, set)]
  pub gnx: String,
  #[pyo3(get, set)]
  pub h: String,
  #[pyo3(get, set)]
  pub b: String,
  #[pyo3(get, set)]
  pub flags: u16
}

#[pymethods]
impl VData {
  #[new]
  fn pynew(obj: &PyRawObject, _gnx:&str) {
    obj.init(VData::new(_gnx));
  }
}
impl VData {
  pub fn new(_gnx:&str) -> VData {
    VData {
      gnx: String::from(_gnx),
      h:String::new(),
      b:String::new(),
      flags:1
    }
  }
  pub fn is_expanded(&self) -> bool {self.flags & 1 == 1}
}
