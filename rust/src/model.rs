#[path="utils.rs"]
mod utils;
use utils::{b64str, b64int, partition, extract_section_ref, has_others};
use std::collections::HashMap;
use pyo3::prelude::*;
use std::path::{ Path};
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

  /// creates object from level and ignx values
  fn make(lev:u8, ignx:u32) -> Self;
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
    *self = (*self & 0x3ffff) | if lev <= 0 { 0 } else { (lev as u32) << 18};
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

  /// creates object from level and ignx values
  fn make(lev:u8, ignx:u32) -> Self {
    ((lev as u32) << 18) | (ignx & 0x3ffff)
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

  /// returns list of all parent indexes for the child node at index i
  fn parents_indexes(&self, i:usize) -> Vec<usize>;

  /// returns list of children ignxes
  fn children(&self, i:usize) -> Vec<u32>;
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
  /// returns list of all parent indexes for the child node at index i
  fn parents_indexes(&self, i:usize) -> Vec<usize> {
    let ignx = self[i].ignx();
    let mut pstack = vec![0usize];
    let mut res = Vec::new();
    for (j, x) in self.into_iter().enumerate() {
      if j == 0 {continue}
      pstack.truncate(x.level() as usize);
      if x.ignx() == ignx {
        res.push(*pstack.last().unwrap())
      }
      pstack.push(j);
    }
    res
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
  /// returns list of children ignxes
  fn children(&self, i:usize) -> Vec<u32> {
    let plev = self[i].level();
    self[i+1..].iter()
      .take_while(|x|x.level() > plev)
      .filter(|x|x.level() == plev + 1)
      .map(|x|x.ignx())
      .collect()
  }
}
/// returns a map of ignx -> gnx
pub fn gnx_index(nodes:&Vec<VData>) -> HashMap<&str, u32> {
  let mut res = HashMap::new();
  for (i, x) in nodes.iter().enumerate() {
    res.insert(x.gnx.as_str(), i as u32);
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
  pub fn clone(&self) -> VData {
    VData {
      gnx: self.gnx.clone(),
      h: self.h.clone(),
      b: self.b.clone(),
      flags: self.flags
    }
  }
  pub fn is_expanded(&self) -> bool {self.flags & 1 == 1}
  pub fn section_ref(&self) -> Option<&str> {
    extract_section_ref(self.h.as_str())
  }
}
pub fn find_any_file_nodes(
    folder:&Path,
    outline:&Outline,
    nodes:&Vec<VData>,
    kind:&str)
     ->
     Vec<(String, usize)> {
  let mut stack = vec![folder.to_path_buf()];
  let mut res = Vec::new();
  for (i, x) in outline.iter().enumerate() {
    let lev = x.level() as usize;
    let v = &nodes[x.ignx() as usize];
    let np = if v.h.starts_with("@path ") {
      &v.h[6..].trim()
    } else if v.b.starts_with("@path ") {
      partition(&v.b[6..], "\n").0.trim()
    } else if let Some(i) = v.b.find("\n@path ") {
      partition(&v.b[i+7..], "\n").0.trim()
    } else {""};

    if lev < stack.len() { stack.drain(lev+1..); }

    let mut nf = stack[stack.len() - 1].clone();
    if np.len() > 0 {
      for x in np.split(|a|a == '/' || a == '\\') {
        nf.push(x);
      }
    }
    stack.push(nf);
    if v.h.starts_with(kind) {
      let fname = v.h[6..].trim();
      let mut p = stack[stack.len() - 1].clone();
      for f in fname.split(|x|x == '\\'  || x == '/') {
        if f == ".." {
          p.pop();
        } else {
          p.push(f);
        }
      }
      res.push((p.to_str().unwrap().to_string(), i));
    }
  }
  res
}
pub fn find_derived_files(folder:&Path, outline:&Outline, nodes:&Vec<VData>) -> Vec<(String, usize)> {
  find_any_file_nodes(folder, outline, nodes, "@file ")
}
pub fn find_clean_files(folder:&Path, outline:&Outline, nodes:&Vec<VData>) -> Vec<(String, usize)> {
  find_any_file_nodes(folder, outline, nodes, "@clean ")
}
pub fn find_auto_files(folder:&Path, outline:&Outline, nodes:&Vec<VData>) -> Vec<(String, usize)> {
  find_any_file_nodes(folder, outline, nodes, "@auto ")
}
pub fn find_edit_files(folder:&Path, outline:&Outline, nodes:&Vec<VData>) -> Vec<(String, usize)> {
  find_any_file_nodes(folder, outline, nodes, "@edit ")
}
pub fn combine_trees(trees:&Vec<(Outline, Vec<VData>)>) -> (Outline, Vec<VData>) {
  let mut catalog:HashMap<&str, (usize, usize)> = HashMap::new();
  for (i, x) in trees.iter().enumerate() {
    let (o, n) = x;
    for y in o.iter() {
      let ignx = y.ignx() as usize;
      let gnx = n[ignx].gnx.as_str();
      catalog.insert(gnx, (i, ignx));
    }
  }
  let vclone = |v:&VData| {
    let gnx = v.gnx.as_str();
    let (i, j) = catalog.get(gnx).unwrap();
    trees[*i].1[*j].clone()
  };
  let (o1, v1) = &trees[0];
  let mut outline = vec![o1[0]];
  let mut vnodes:Vec<VData> = v1.iter().map(vclone).collect();
  let mut real_start = 0;
  let mut i = trees.len();
  let mut ignxes = gnx_index(&v1);
  for (o, n) in trees.iter().rev() {
    i -= 1;
    if i == 0 {
      real_start = outline.len();
      for x in o.iter().skip(1) {
        let gnx = n[x.ignx() as usize].gnx.as_str();
        let ii = ignxes.get(gnx).unwrap();
        outline.add_node(x.level(), *ii);
      }
    } else {
      for v in n.iter() {
        let gnx = v.gnx.as_str();
        if !ignxes.contains_key(gnx) {
          let ii = ignxes.len() as u32;
          ignxes.insert(gnx, ii);
          vnodes.push(vclone(v));
        }
      }
      for x in o {
        let gnx = n[x.ignx() as usize].gnx.as_str();
        let ii = ignxes.get(gnx).unwrap();
        outline.add_node(x.level(), *ii);
      }
    }
  }
  outline.drain(1..real_start);
  (outline, vnodes)
}
pub fn extract_subtree(
    outline:&Outline,
    nodes:&Vec<VData>,
    ni:usize) -> (Outline, Vec<VData>) {
    let mut o = Vec::new();
    let mut ind:LevGnx = 0;
    let mut vs = Vec::new();
    let zlev = outline[ni].level();
    for lg in &outline[ni..] {
      let lev = lg.level();
      if ind == 0 || lev > zlev {
        let ignx = lg.ignx();
        let v = nodes[ignx as usize].clone();
        o.push(LevGnx::make(lev-zlev, ind));
        vs.push(v);
        ind += 1;
      } else {
        break
      }
    }
    (o, vs)
}
pub struct Tree<'a> {
  outline: &'a Outline,
  nodes: &'a Vec<VData>,
  predicat: fn(u8, &VData) -> bool,
  index: usize
}
#[allow(dead_code)]
impl<'a> Tree<'a> {
  pub fn new(outline:&'a Outline, nodes:&'a Vec<VData>) -> Self {
    Tree {
      outline,
      nodes,
      predicat: |_, _| {true},
      index: 0
    }
  }
  pub fn roots(outline:&'a Outline, nodes:&'a Vec<VData>, predicat:fn(u8, &VData) -> bool) -> Self {
    Tree {
      outline, nodes, predicat, index: 0
    }
  }
  pub fn skip(&mut self, i:usize) {self.index = i }
  pub fn skip_sections(mut self, start:usize) -> impl Iterator<Item=(u8, &'a VData)> {
    self.predicat = |_, v| v.section_ref().is_none();
    self.index = start;
    self.into_iter()
  }
  pub fn skip_sections_and_nodes_with_others(mut self, start:usize) -> impl Iterator<Item=(u8, &'a VData)> {
    self.predicat = |_, v| {
      v.section_ref().is_none() && !has_others(v.b.as_str())
    };
    self.index = start;
    self.into_iter()
  }
  pub fn find_section(&self, p:&str, start:usize) -> Option<usize> {
    let o = self.outline;
    let vs = self.nodes;
    let mut i = start;
    let zlev = o[i].level();
    if let Some(x) = vs[o[i].ignx() as usize].section_ref() {
      if x == p {return Some(i)}
    }
    i += 1;
    while i < o.len() && o[i].level() > zlev {
       if let Some(x) = vs[o[i].ignx() as usize].section_ref() {
        if x == p {return Some(i)}
       }
       i += 1;
    }
    None
  }
  pub fn others(self, start:usize) -> impl Iterator<Item=(u8, &'a VData)> {
    self.skip_sections_and_nodes_with_others(start)
    .filter(|(_, v)|v.section_ref().is_none())
  }
}
impl<'a> Iterator for Tree<'a> {
  type Item = (u8, &'a VData);
  fn next(&mut self) -> Option<Self::Item> {
    let i = &mut self.index;
    let o = &self.outline;
    let vs = &self.nodes;
    if *i < o.len() {
      let lev = o[*i].level();
      let v = &vs[o[*i].ignx() as usize];
      *i += 1;
      if !(self.predicat)(lev, v) {
        while *i < o.len() && o[*i].level() > lev {
          *i += 1;
        }
      }
      Some((lev, v))
    } else {
      None
    }
  }
}
