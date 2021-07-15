#[path="utils.rs"]
mod utils;
use utils::{b64str, b64int, b64write, partition, extract_section_ref, has_others,
            insert_parts, /*make_gaps,*/ delete_blocks};
use std::collections::{HashMap, HashSet};
use pyo3::prelude::*;
use std::path::{ Path};
use std::error::Error;
use std::fmt;
pub type LevGnx = u64;
pub trait LevGnxOps {
  /// returns level of this node
  fn level(&self) -> u8;

  /// returns ignx of this node
  fn ignx(&self) -> u32;

  /// returns label of this node
  fn label(&self) -> u32;

  /// returns true if this node should be expanded
  fn is_expanded(&self) -> bool;

  /// increments level of this node
  fn inc(&mut self);

  /// decrements level of this node
  fn dec(&mut self);

  /// changes the level of this node for given delta d
  fn shift(&mut self, d: i8);

  /// sets level of this node to given value
  fn set_level(&mut self, lev: u8);

  /// sets ignx of this node to given value
  fn set_ignx(&mut self, ignx:u32);

  /// sets label for this node to given value
  fn set_label(&mut self, label:u32);

  /// expands this node
  fn expand(&mut self);

  /// collapses this node
  fn collapse(&mut self);

  /// converts this object into ascii representation (11 ascii letters)
  fn to_str(&self) -> String;

  /// creates object from its String representation
  fn from_str(a:&str) -> LevGnx;

  /// creates object from level and ignx values
  fn make(lev:u8, ignx:u32, label:u32) -> Self;

  /// appends String representation of this node to the given buffer
  fn write(&self, buf:&mut String);
}
const LEVEL_ONE:u64 =             0x100_0000;
const LEVEL_MASK:u64 =           0xFF00_0000;
const IGNX_MASK:u64 =              0xFF_FFFF;
const LABEL_MASK:u64 = 0x7FFF_FFFF_0000_0000;
const EXPANDED:u64 =   0x8000_0000_0000_0000;
impl LevGnxOps for LevGnx {

  /// returns level of this node
  fn level(&self) -> u8 {(((*self) >> 24) & 255) as u8}

  /// sets level of this node to given value
  fn set_level(&mut self, lev: u8) {
    *self = (*self & !LEVEL_MASK) | ((lev as u64) << 24)
  }

  /// returns ignx of this node
  fn ignx(&self) -> u32 {((*self) & IGNX_MASK) as u32}

  /// increments level of this node
  fn inc(&mut self) {*self += LEVEL_ONE;}

  /// decrements level of this node
  fn dec(&mut self) {
    if *self & LEVEL_MASK > 0 {
      *self -= LEVEL_ONE;
    }
  }

  /// changes the level of this node for given delta d
  fn shift(&mut self, d: i8) {
    if d < 0 {
      let d1 = ((-d) as u64) << 24;
      if *self & LEVEL_MASK >= d1 {
        *self -= d1;
      }
    } else {
      let d1 = (d as u64) << 24;
      *self += d1;
    }
  }

  /// returns label of this node
  fn label(&self) -> u32 {((*self & LABEL_MASK) >> 32) as u32}

  /// sets label of this node to given value
  fn set_label(&mut self, label:u32) {
    let mlab:u64 = ((label as u64) << 32) & LABEL_MASK;
    *self = (*self & !LABEL_MASK) | mlab;
  }

  /// expands this node
  fn expand(&mut self) {*self |= EXPANDED}

  /// collapses this node
  fn collapse(&mut self) {*self &=!EXPANDED}

  /// returns true if this node should be expanded
  fn is_expanded(&self) -> bool {*self & EXPANDED != 0}

  /// sets ignx of this node to given value
  fn set_ignx(&mut self, ignx:u32) {
    *self = (*self & !IGNX_MASK) | (ignx as u64) & IGNX_MASK;
  }

  /// converts this node into ascii representation (4 ascii letters)
  fn to_str(&self) -> String {
    let mut res = b64str(*self);
    while res.len() < 11 {res.insert(0, '0');}
    res
  }

  /// creates object from its String representation
  fn from_str(a:&str) -> LevGnx {
    b64int(&a[..4]) as LevGnx
  }
  /// appends String representation of this node to the given buffer
  fn write(&self, buf:&mut String) {
    b64write(*self, buf);
  }
  /// creates object from level and ignx values
  fn make(lev:u8, ignx:u32, label:u32) -> Self {
    (((label as u64) << 32) & LABEL_MASK) |
    ((lev as u64) << 24) |
    (ignx as u64 & IGNX_MASK)
  }
}
#[test]
fn test_make_levgnx() {
  let i = LevGnx::make(3, 5, 7);
  assert_eq!(i, 0x0000_0007_03_000005);
}
#[test]
fn test_getters_levgnx() {
  let i = LevGnx::make(3, 5, 7);
  assert_eq!(i, 0x0000_0007_03_000005);  assert_eq!(i.ignx(), 5);
  assert_eq!(i.level(), 3);
  assert_eq!(i.label(), 7);
}
#[test]
fn test_setters_levgnx() {
  let mut i = 0u64;
  i.set_label(8);
  assert_eq!(i.label(), 8);
  i.set_level(3);
  assert_eq!(i.level(), 3);
  i.set_ignx(5);
  assert_eq!(i.ignx(), 5);
}


#[test]
fn test_shift_levgnx() {
  let mut i = LevGnx::make(3, 5, 7);
  i.shift(3);
  assert_eq!(i, 0x0000_0007_06_000005);
  i.shift(-2);
  assert_eq!(i, 0x0000_0007_04_000005);
}

#[test]
fn test_incdec_levgnx() {
  let mut i = LevGnx::make(3, 5, 7);
  i.inc();
  assert_eq!(i, 0x0000_0007_04_000005);
  i.dec();
  assert_eq!(i, 0x0000_0007_03_000005);
}
pub type Outline = Vec<u64>;

pub trait OutlineOps {
  /// returns true if this outline conains a node with given ignx
  fn has(&self, ignx:u32) -> bool;

  /// returns the index of the first node with the given ignx
  /// if the node can't be found returns None
  fn find(&self, ignx:u32) -> Option<usize>;

  /// returns suboutline of the node with the given ignx
  /// if such a node dosen't exist returns an empty outline
  fn subtree(&self, ignx:u32) -> Outline;

  /// appends a node with the given ignx at the given level.
  /// Returns true if the node is clone and its subtree has been added too;
  /// otherwise returns false.
  fn add_node(&mut self, level: u8, ignx: u32) -> Result<bool, TreeError>;

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
  fn has(&self, ignx:u32) -> bool { self.find(ignx).is_some()}
  /// returns the index of the first node with the given ignx
  /// if the node can't be found returns None
  fn find(&self, ignx:u32) -> Option<usize> {
    let mut i = 0usize;
    for x in self {
      if x.ignx() == ignx {return Some(i)}
      i += 1;
    }
    None
  }
  /// returns suboutline of the node with the given ignx
  /// if such a node dosen't exist returns an empty outline
  fn subtree(&self, ignx:u32) -> Outline {
    let mut res:Outline = Vec::new();
    if let Some(j) = self.find(ignx) {
      let mut i = j;
      let z = self[i];
      let zlev:u8 = z.level();
      let delta:i8 = -(zlev as i8);
      res.push(z & !LEVEL_MASK);
      let n = self.len();
      while i + 1 < n {
        i += 1;
        let mut z = self[i];
        if z.level() <= zlev { break };
        z.shift(delta);
        res.push(z);
      }
    }
    res
  }
  /// appends a node with the given ignx at the given level.
  /// Returns true if the node is clone and its subtree has been added too;
  /// otherwise returns false.
  fn add_node(&mut self, level: u8, ignx: u32) -> Result<bool, TreeError> {
    let mut label = if self.is_empty() {0} else {self[0].label()};
    let max_level:u8 = match self.last() {
      Some(z) => z.level() + 1,
      None => 0
    };
    if level > max_level {
      let e = TreeError(format!(
        "trying to add a node to level {} when max_level is {}"
        , level
        , max_level
        )); 
      return Err(e);
    }
    let st = self.subtree(ignx);
    if st.is_empty() {
      self.push(LevGnxOps::make(level, ignx, label+1));
      self[0].set_label(label + 1);
      Ok(false)
    } else {
      for mut z in st {
        z.shift(level as i8);
        z.set_label(label + 1);
        label += 1;
        self.push(z);
      }
      Ok(true)
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
    let z = self[i] & LEVEL_MASK;
    self[i+1..]
      .iter()
      .position(|x|(*x & LEVEL_MASK) <= z)
      .unwrap_or(self.len() - i - 1) + 1
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
  #[pyo3(get)]
  pub gnx: String,
  #[pyo3(get)]
  pub ignx: u32,
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
  fn pynew(_gnx:&str) -> Self {
    VData::new(_gnx)
  }
}
impl VData {
  pub fn new(_gnx:&str) -> VData {
    VData {
      gnx: String::from(_gnx),
      ignx:0,
      h:String::new(),
      b:String::new(),
      flags:1,
    }
  }
  pub fn clone(&self) -> VData {
    VData {
      gnx: self.gnx.clone(),
      h: self.h.clone(),
      b: self.b.clone(),
      flags: self.flags,
      ignx: self.ignx
    }
  }
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
      if ignx > n.len() {
        println!("{:x} nepostojeci index: ignx={:x}", y, ignx);
        for v in n.iter() {
          println!("{}:{}", v.ignx, v.h);
        }
        println!("{}", INDENT);
        for v in o.iter() {
          println!("{}:{:08x}:{:08x} {:016x}",&INDENT[0..v.level() as usize], v.label(), v.ignx(), v);
        }
      }
      let gnx = n[ignx].gnx.as_str();
      catalog.insert(gnx, (i, ignx));
    }
  }
  let vclone = |(k,v):(usize, &VData)| {
    let gnx = v.gnx.as_str();
    let (i, j) = catalog.get(gnx).unwrap();
    let mut v = trees[*i].1[*j].clone();
    v.ignx = k as u32;
    v
  };
  let (o1, v1) = &trees[0];
  let mut outline = vec![o1[0]];
  outline[0].set_label(0);
  let mut vnodes:Vec<VData> = v1.iter().enumerate().map(vclone).collect();
  let mut real_start = 0;
  let mut i = trees.len();
  let mut ignxes = gnx_index(&v1);
  for (o, n) in trees.iter().rev() {
    if o.len() < 1 { continue }
    i -= 1;
    if i == 0 {
      real_start = outline.len();
      let mut skip_level = 255u8;
      for x in o.iter().skip(1) {
        if x.level() <= skip_level {
          let gnx = n[x.ignx() as usize].gnx.as_str();
          let ii = ignxes.get(gnx).unwrap();
          skip_level = if outline.add_node(x.level(), *ii).unwrap() {
                          x.level()
                       } else {
                          255u8
                       };
        }
      }
    } else {
      for v in n.iter() {
        let gnx = v.gnx.as_str();
        if !ignxes.contains_key(gnx) {
          let ii = ignxes.len() as u32;
          ignxes.insert(gnx, ii);
          vnodes.push(vclone((ii as usize, v)));
        }
      }
      let mut skip_level = 255u8;
      for x in o {
        if x.level() <= skip_level {
          let gnx = n[x.ignx() as usize].gnx.as_str();
          let ii = ignxes.get(gnx).unwrap();
          skip_level = if outline.add_node(x.level(), *ii).unwrap() {
                          x.level()
                       } else {
                          255u8
                       };
        }
      }
    }
  }
  outline.drain(1..real_start);
  (outline, vnodes)
}
pub const INDENT:&str="...........................................................................................................................................................................................................................................................................................................................................";

#[allow(dead_code)]
fn dump_outline(o:&Outline, n:&Vec<VData>, s:&str){
  println!("DUMPING START: {}", s);
  for (i, x) in o[..].iter().enumerate() {
    let v = &n[x.ignx() as usize];
    println!("{}{}{}", i, &INDENT[..x.level() as usize], v.h);
  }
  println!("DUMPING END: {}", s);
  println!("");
}
pub fn extract_subtree(
    outline:&Outline,
    nodes:&Vec<VData>,
    ni:usize) -> (Outline, Vec<VData>) {
    let rawnodes = &nodes[..];
    let mut o = Vec::new();
    let mut label = 0;
    let mut ind:LevGnx = 0;
    let mut vs = Vec::new();
    let mut gnx2i:HashMap<&str, usize>=HashMap::new();
    let mut gnxcount:usize = 0;
    let zlev = outline[ni].level();
    for lg in &outline[ni..] {
      let lev = lg.level();
      if ind == 0 || lev > zlev {
        let ignx = lg.ignx();
        let v = &rawnodes[ignx as usize];
        let ignx2 = gnx2i.entry(&v.gnx).or_insert(gnxcount);
        if *ignx2 == gnxcount {
          vs.push(v.clone());
          gnxcount +=1;
        }
        o.push(LevGnx::make(lev-zlev, (*ignx2) as u32, label));
        label += 1;
        ind += 1;
      } else {
        break
      }
    }
    o[0].set_label(label);
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
#[allow(dead_code)]
pub fn all_parents(o: &Outline, n:&Vec<VData>) -> Vec<HashSet<u64>> {
  let mut res:Vec<HashSet<u64>> = n.iter().map(|_|HashSet::new()).collect();
  let mut stack = vec![0u64];
  for (i, x) in o.iter().enumerate().skip(1) {
    let lev = x.level() as usize;
    let ignx = x.ignx() as u64;
    let pignx = (stack[lev-1] >> 32) as u64;
    let pind = (stack[lev-1] & 0xFFFF_FFFF) as u64;
    stack.truncate(lev);
    stack.push(((ignx << 32) | (i as u64)) as u64);
    res[ignx as usize].insert((pignx << 32) | (i as u64 - pind));
  }
  res
}
#[allow(dead_code)]
pub fn clonned_nodes(o:&Outline, n:&Vec<VData>) -> Vec<u32> {
  all_parents(o, n)
    .iter()
    .enumerate()
    .filter(|x|x.1.len()>1)
    .map(|x|(x.0 as u32))
    .collect()
}
pub fn check_levels(o:&Outline) -> Option<usize> {
  o.iter()
   .enumerate()
   .skip(1)
   .filter(|(i,x)|x.level() > o[i-1].level()+1)
   .map(|x|x.0)
   .find(|_|true)
}

pub fn check_labels(o:&Outline) -> Option<usize> {
  let mut seen:HashSet<u32> = HashSet::new();
  for (i, x) in o.iter().enumerate().skip(1) {
    let lab = x.label();
    if seen.contains(&lab) { return Some(i) }
    seen.insert(lab);
  }
  None
}

pub fn check_clones(o:&Outline, n:&Vec<VData>) -> Option<u32> {
  let clones = clonned_nodes(o, n);
  for cignx in clones.iter() {
    let t1 = o.subtree(*cignx);
    let it = o.iter()
              .enumerate()
              .filter(|x|x.1.ignx() == *cignx)
              .skip(1);
    for (i, x) in it {
      let zlev = x.level();
      for (j, y) in t1.iter().enumerate() {
        let z = o[i + j];
        if y.ignx() != z.ignx() || y.level() + zlev != z.level() {
          return Some(*cignx)
        }
      }
    }
  }
  None
}
pub fn create_link(o:&mut Outline, pignx:usize, child_index:usize, cignx:usize) -> Option<String> {
  let mut tch = o.subtree(cignx as u32);
  if tch.len() == 0 {return None}
  let pu32 = pignx as u32;
  for x in tch.iter() {
    if x.ignx() == pu32 { return None }
  }
  let mut tp = o.subtree(pu32);
  tp.push(LevGnx::make(1, 0, 0));
  if tp.len() == 0 {return None}
  if let Some((j, _)) = tp.iter()
                          .enumerate()
                          .filter(|x|x.1.level()==1)
                          .skip(child_index)
                          .next() {
    let mut label = o[0].label()+1;
    let marks:Vec<usize> = 
      o.iter()
       .enumerate()
       .skip(1)
       .filter(|x|x.1.ignx() == pu32)
       .map(|(i,_)|i+j)
       .collect();
    let sz = tch.len();
    for (i, m) in marks.iter().enumerate() {
      if i > 0 {
        tch.extend_from_within(0..sz);
      }
      let zlev = (o[*m as usize - j].level() + 1) as i8 - tch[0].level() as i8;
      for x in tch.iter_mut().skip(i*sz) {
        x.shift(zlev as i8);
        x.set_label(label); label += 1;
      }
    }
    let mut res = String::new();
    encode_insert_parts(o, &marks, &tch, &mut res);
    insert_parts(o, &marks, &tch);
    o[0].set_label(label-1);
    return Some(res)
  }
  None
}
#[allow(dead_code)]
fn log<T>(x:T) -> T where T: std::fmt::Debug {
  println!("{:?}", x);
  x
}
pub fn break_link(o:&mut Outline, pignx:u32, child_index:usize) -> Option<String> {
  if let Some(pi) = o.find(pignx) {
    let zlev = o[pi].level() + 1;
    if let Some(ci) = o.iter()
                       .skip(pi+1)
                       .enumerate()
                       .take_while(|x|x.1.level() >= zlev)
                       .filter(|x|x.1.level() == zlev)
                       .skip(child_index)
                       .map(|x|x.0+1)
                       .next() {
      // now parent is at index pi, and child starts at the index ci
      let sz = o.subtree_size(ci+pi);
      let delta = ci;
      let mut marks:Vec<usize> =
        o.iter()
         .enumerate()
         .skip(pi+ci+sz)
         .filter(|x|x.1.ignx() == pignx)
         .map(|x|x.0+delta)
         .collect();
      marks.insert(0, pi+ci);
      let mut buf = String::new();
      encode_delete_blocks(o, &marks, sz, &mut buf);
      delete_blocks(o, &marks, sz);
      return Some(buf);
    }
  };
  None
}
fn encode_insert_parts(o:&Outline, marks:&Vec<usize>, data:&Vec<u64>, mut buf:&mut String) {
    // TODO: check if it is necessary to have o[0].label() after parts have been
    //       inserted. This information might be necessary for redo!
    buf.push_str("ip:");
    b64write(o[0].label() as u64, &mut buf);
    buf.push_str(",(");
    for m in marks.iter() {
      b64write(*m as u64, &mut buf);
      buf.push(',');
    }
    buf.pop();
    buf.push_str("),(");
    for m in data.iter() {
      if m.is_expanded() {
        buf.push('+');
      } else {
        buf.push('-');
      }
      b64write(m.level() as u64, &mut buf);buf.push('.');
      b64write(m.ignx() as u64, &mut buf);buf.push('.');
      b64write(m.label() as u64, &mut buf);buf.push(',');
    }
    buf.pop();
    buf.push(')');
}
fn encode_delete_blocks(o:&Outline, marks:&Vec<usize>, sz:usize, mut buf:&mut String) {
    buf.push_str("db:");
    b64write(sz as u64, &mut buf);
    buf.push_str(",(");
    let mut data:Vec<u64> = Vec::new();
    for m in marks.iter() {
      b64write(*m as u64, &mut buf);
      buf.push(',');
      for i in o.iter().skip(*m).take(sz) {
        data.push(*i);
      }
    }
    buf.pop();
    buf.push_str("),(");
    for m in data.iter() {
      if m.is_expanded() {
        buf.push('+');
      } else {
        buf.push('-');
      }
      b64write(m.level() as u64, &mut buf);buf.push('.');
      b64write(m.ignx() as u64, &mut buf);buf.push('.');
      b64write(m.label() as u64, &mut buf);buf.push(',');
    }
    buf.pop();
    buf.push(')');
}
pub fn undo_delete_blocks(o:&mut Outline, enc:&str) {
  // expects enc string to look like
  // db:xxx,(m1,m2,m3,...),(_l1.ignx1.lab1,_l2.ignx2.lab2,_l3.ignx3.lab3,...)
  // where xxx is size of the deleted block
  // m1,m2,m3,... are old marks before deletion
  //_l1.ignx1.lab1,... are nodes that were deleted
  let size = b64int(&enc[3..]) as usize;
  let s2 = partition(&enc, ",(").2;
  let (s2,_,s3) = partition(&s2, "),(");
  let pmarks:Vec<usize> =
    s2.split(',')
     .enumerate()
     .map(|x|b64int(x.1) as usize - size*x.0)
     .collect();
  let data:Vec<u64> =
      s3.split(",")
        .map(|x|{
          let mut it = x[1..].split('.');
          let lev = b64int(it.next().unwrap()) as u8;
          let ignx = b64int(it.next().unwrap()) as u32;
          let lab = b64int(it.next().unwrap()) as u32;
          let mut y = LevGnx::make(lev, ignx, lab);
          if x.starts_with('-') {
            y.expand();
          } else {
            y.collapse();
          }
          y
        }).collect();
  insert_parts(o, &pmarks, &data);
}
pub fn redo_delete_blocks(o:&mut Outline, enc:&str) {
  // expects enc string to look like
  // db:xxx,(m1,m2,m3,...),(_l1.ignx1.lab1,_l2.ignx2.lab2,_l3.ignx3.lab3,...)
  // where xxx is size of the deleted block
  // m1,m2,m3,... are old marks before deletion
  //_l1.ignx1.lab1,... are nodes that were deleted
  let size = b64int(&enc[3..]) as usize;
  let s2 = partition(&enc, ",(").2;
  let s2 = partition(&s2, "),(").0;
  let pmarks:Vec<usize> =
    s2.split(',')
     .enumerate()
     .map(|x|b64int(x.1) as usize)
     .collect();
  delete_blocks(o, &pmarks, size);
}
pub fn undo_insert_parts(o:&mut Outline, enc:&str) {
    // expects enc string to look like
    // ip:xxx,(m1,m2,m3,...),(_l1.ignx1.lab1,_l2.ignx2.lab2,_l3.ignx3.lab3,...)
    // where xxx is old label of o[0]
    // m1,m2,m3,... are old marks
    //_l1.ignx1.lab1,... are nodes that were inserted
    let old_zlabel = b64int(&enc[3..]);
    let s2 = partition(&enc, ",(").2;
    let (s2,_,s3) = partition(&s2, "),(");
    let pmarks:Vec<usize> =
      s2.split(',')
       .map(|x|b64int(x) as usize)
       .collect();
    let size = s3.split(",").count()/pmarks.len();
    let nmarks:Vec<usize> =
      pmarks
        .iter()
        .enumerate()
        .map(|(i,j)|j+i*size)
        .collect();
    delete_blocks(o, &nmarks, size);
    o[0].set_label(old_zlabel as u32);
}
pub fn redo_insert_parts(o:&mut Outline, enc:&str) {
    // expects enc string to look like
    // ip:xxx,(m1,m2,m3,...),(_l1.ignx1.lab1,_l2.ignx2.lab2,_l3.ignx3.lab3,...)
    // where xxx is old label of o[0]
    // m1,m2,m3,... are old marks
    //_l1.ignx1.lab1,... are nodes that were inserted
    let s2 = partition(&enc, ",(").2;
    let (s2,_,s3) = partition(&s2, "),(");
    let pmarks:Vec<usize> =
      s2.split(',')
       .map(|x|b64int(x) as usize)
       .collect();
    let data:Vec<u64> =
      s3.split(",")
        .map(|x|{
          let mut it = x[1..].split('.');
          let lev = b64int(it.next().unwrap()) as u8;
          let ignx = b64int(it.next().unwrap()) as u32;
          let lab = b64int(it.next().unwrap()) as u32;
          let mut y = LevGnx::make(lev, ignx, lab);
          if x.starts_with('-') {
            y.expand();
          } else {
            y.collapse();
          }
          y
        }).collect();
    insert_parts(o, &pmarks, &data);
    let z = &mut o[0];
    z.set_label(z.label() + data.len() as u32);
}


#[derive(Debug)]
pub struct TreeError(String);
impl Error for TreeError{}
impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TreeError:{}", self.0)
    }
}
