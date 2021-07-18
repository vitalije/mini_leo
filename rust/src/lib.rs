#[path="model.rs"]
mod model;
#[path="utils.rs"]
mod utils;
#[path="parsing.rs"]
mod parsing;
#[path="atclean.rs"]
mod atclean;
use std::collections::HashMap;
use std::sync::{Mutex};
pub use parsing::{ldf_parse,from_derived_file_content, from_derived_file,
                  from_leo_file, from_leo_content, load_with_external_files,
                  /*from_zip_archive,*/
                  };
pub use atclean::{atclean_to_string, update_atclean_tree};
pub use utils::{b64int, b64str, b64write};
pub use model::{VData, Outline, OutlineOps, LevGnx, LevGnxOps, gnx_index,
                find_derived_files, find_edit_files,
                find_auto_files, find_clean_files,
                check_levels, check_clones, check_labels,
                create_link, undo_insert_parts, redo_insert_parts,
                break_link, undo_delete_blocks, redo_delete_blocks,
                undo_shift_blocks, redo_shift_blocks,
                undo_set_nodes, redo_set_nodes,
                move_node_right, move_node_left, move_node_up, move_node_down,
                extract_subtree, INDENT};
use pyo3::prelude::*;
use pyo3::PyIterProtocol;
use pyo3::exceptions::{PyValueError, PyIOError};

//use pyo3::{wrap_pyfunction};
//use pyo3::type_object::PyTypeObject;
//use xml::reader::{ParserConfig, XmlEvent};
use std::path::{Path};
#[macro_use]
extern crate lazy_static;

#[pyfunction]
pub fn a_function_from_rust() -> PyResult<i32> {
  Ok(42)
}
pub struct Tree {
  outline: Outline,
  nodes: Vec<VData>,
}
#[pyclass]
struct TreeIterator {
  tree_id: usize,
  index: usize
}
#[pyproto]
impl PyIterProtocol for TreeIterator {
  fn __iter__(slf:PyRefMut<Self>) -> PyResult<Py<TreeIterator>> {
    Ok(slf.into())
  }
  fn __next__(mut slf:PyRefMut<Self>) -> PyResult<Option<(u8, bool, u32, VData)>> {
    let m = TREES.lock().unwrap();
    let res = m.get(&slf.tree_id).map(|t|{
      let n = t.outline.len();
      if slf.index < n {
        let levgnx = t.outline[slf.index];
        let i = levgnx.ignx() as usize;
        Some((levgnx.level(), levgnx.is_expanded(), levgnx.label(), t.nodes[i].clone()))
      } else {
        None
      }
    });
    slf.index += 1;
    match res {
      Some(Some(x)) => Ok(Some(x)),
      _ => Ok(None)
    }
  }
}
lazy_static! {
  static ref TREES:Mutex<Box<HashMap<usize,Tree>>> = Mutex::new(Box::new(HashMap::new()));
}
#[pymodule]
fn _minileo(_py: Python, m:&PyModule) -> PyResult<()> {
  /// creates outline from str
  #[pyfn(m)]
  #[pyo3(name="at_files")]
  fn at_files(_py:Python, tid:usize, folder:&str) -> PyResult<Vec<(String, usize)>> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      find_derived_files(&Path::new(folder), &t.outline, &t.nodes)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="auto_files")]
  fn auto_files(_py:Python, tid:usize, folder:&str) -> PyResult<Vec<(String, usize)>> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      find_auto_files(&Path::new(folder), &t.outline, &t.nodes)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="check_tree", text_signature="(tid)")]
  /// Returns None if the given tree is valid.
  /// In case an error has been found, throws ValueError
  ///
  fn check_tree(_py: Python, tid:usize) -> PyResult<()> {
    let res = TREES.lock().unwrap().get(&tid).and_then(|t|{
      if let Some(i) = check_levels(&t.outline) {
        let msg = format!("Level validation failed at index:{}", i);
        return Some(PyValueError::new_err(msg));
      }
      if let Some(ignx) = check_clones(&t.outline, &t.nodes) {
        let v = &t.nodes[ignx as usize];
        let msg = format!("Clones of {}[{}] are different", v.h, v.gnx);
        return Some(PyValueError::new_err(msg));
      }
      if let Some(i) = check_labels(&t.outline) {
        let msg = format!("Duplicate position value at index:{}", i);
        return Some(PyValueError::new_err(msg));
      }
      return None
    });
    if let Some(e) = res {
      return Err(e);
    }
    Ok(())
  }
  #[pyfn(m)]
  #[pyo3(name="children")]
  fn children(_py:Python, tid:usize, ni:usize) -> PyResult<Vec<VData>> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      t.outline.children(ni)
        .into_iter()
        .map(|i|t.nodes[i as usize].clone())
        .collect()
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="clean_files")]
  fn clean_files(_py:Python, tid:usize, folder:&str) -> PyResult<Vec<(String, usize)>> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      find_clean_files(&Path::new(folder), &t.outline, &t.nodes)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="collapse_node", text_signature="(tid, p)")]
  /// Collapses node at position p in the outline
  /// idetified by tid.
  /// 
  /// Returns None if the tid outline is missing or there isn't a node at 
  /// the given position p, or node was already collapsed
  fn collapse_node(_py: Python, tid:usize, label:u32) -> Option<String> {
    TREES
      .lock()
      .unwrap()
      .get_mut(&tid)
      .and_then(|t|{
        t.outline
          .iter_mut()
          .enumerate()
          .filter(|x|x.1.label() == label)
          .nth(0)
          .filter(|x|x.1.is_expanded())
          .map(|x| {
            let mut s = "collapse:".to_string();
            b64write(x.0 as u64, &mut s);
            x.1.collapse();
            s
          })
      })
  }
  #[pyfn(m)]
  #[pyo3(name="debug_str", text_signature="(tid)")]
  /// Returns debug representation of the complete outline
  ///
  /// Raises ValueError if there is no such outline
  ///
  fn to_debug_str(_py: Python, tid: usize) -> PyResult<String> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
        let mut res = String::new();
        for (i, x) in t.outline.iter().enumerate() {
          let e = if x.is_expanded() { "+" } else { "-" };
          let row = format!("{:3}{}[0x{:X}/0x{:X}]{}{}\n", i, e, x.ignx(), x.label()
                           , &INDENT[..x.level() as usize]
                           , t.nodes[x.ignx() as usize].h);
          res.push_str(&row);
        }
        Ok(res)
      }) {
      Some(t) => t,
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="drop_tree")]
  fn drop_tree(_py:Python, tid:usize) -> PyResult<bool> {
    Ok(TREES.lock().unwrap().remove(&tid).is_some())
  }
  #[pyfn(m)]
  #[pyo3(name="edit_files")]
  fn edit_files(_py:Python, tid:usize, folder:&str) -> PyResult<Vec<(String, usize)>> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      find_edit_files(&Path::new(folder), &t.outline, &t.nodes)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="expand_node", text_signature="(tid, p)")]
  /// Expands node at position p in the outline
  /// idetified by tid.
  /// 
  /// Returns None if the tid outline is missing or there isn't a node at 
  /// the given position p, or node was already expanded
  fn expand_node(_py: Python, tid:usize, label:u32) -> Option<String> {
    TREES
      .lock()
      .unwrap()
      .get_mut(&tid)
      .and_then(|t|{
        t.outline
          .iter_mut()
          .enumerate()
          .filter(|x|x.1.label() == label)
          .nth(0)
          .filter(|x|!x.1.is_expanded())
          .map(|x| {
            let mut s = "expand:".to_string();
            b64write(x.0 as u64, &mut s);
            x.1.expand();
            s
          })
      })
  }
  #[pyfn(m)]
  #[pyo3(name="extract_subtree")]
  fn pyextract_subtree(_py: Python, tid:usize, ni:usize) -> PyResult<usize> {
    let (outline, nodes) = match TREES.lock().unwrap().get(&tid) {
      Some(t) => extract_subtree(&t.outline, &t.nodes, ni),
      None => return Err(PyValueError::new_err("unknown tree id"))
    };
    let t = Tree {outline, nodes};
    let mut m = TREES.lock().unwrap();
    let ntid = m.len();
    m.insert(ntid, t);
    Ok(ntid)
  }
  #[pyfn(m)]
  #[pyo3(name="gnx_index", text_signature="(tid)")]
  /// Returns a map of gnx->ignx in the outline
  /// idetified by tid.
  /// 
  /// Returns None if the tid outline is missing
  fn gnx_index(_py: Python, tid:usize) -> PyResult<Option<HashMap<String, u32>>> {
    let res = TREES.lock().unwrap().get(&tid).map(|t|{
      let mut m = HashMap::new();
      for (i, v) in t.nodes.iter().enumerate() {
        m.insert(v.gnx.clone(), i as u32);
      }
      m
    });
    Ok(res)
  }
  #[pyfn(m)]
  #[pyo3(name="iternodes")]
  fn iternodes(_py:Python, tid:usize) -> PyResult<TreeIterator> {
    Ok(TreeIterator {tree_id:tid, index:0})
  }

  #[pyfn(m)]
  #[pyo3(name="load_leo", text_signature="(fname)")]
  /// Loads Leo document from the given filename.
  ///
  /// This function will also load all external (at-file) files
  /// found in the outline.
  ///
  /// Raises IOError if the file doesn't exists or it
  /// doesn't contain valid Leo document.
  ///
  fn load_leo(_py: Python, fname:&str) -> PyResult<usize> {
    match load_with_external_files(fname) {
      Ok((outline, nodes)) => {
        let t = Tree {outline, nodes};
        let mut m = TREES.lock().unwrap();
        let tid = m.len();
        m.insert(tid, t);
        Ok(tid)
      },
      Err(e) =>
        Err(PyIOError::new_err(e.to_string()))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="move_node_right", text_signature="(tid, label)")]
  /// Moves node right idetified by given label in the outline
  /// idetified by tid. If the movement would result in the
  /// invalid outline, returns None, without making any change.
  /// 
  /// Returns string representation of the performed changes, that
  /// can be later used for undo/redo operations.
  ///
  /// Returns None if the outline or node is missing
  ///
  fn pymove_node_right(_py: Python, tid:usize, label:u32) -> Option<String> {
    TREES
      .lock()
      .unwrap()
      .get_mut(&tid)
      .and_then(|t|{
        let i = t.outline
         .iter()
         .skip(1)
         .position(|x|x.label() == label)
         .unwrap_or(0);
        if i == 0 {return None}
        move_node_right(&mut t.outline, i+1)
      })
  }
  #[pyfn(m)]
  #[pyo3(name="move_node_left", text_signature="(tid, label)")]
  /// Moves node left idetified by given label in the outline
  /// idetified by tid. If the movement would result in the
  /// invalid outline, returns None, without making any change.
  /// 
  /// Returns string representation of the performed changes, that
  /// can be later used for undo/redo operations.
  ///
  /// Returns None if the outline or node is missing
  ///
  fn pymove_node_left(_py: Python, tid:usize, label:u32) -> Option<String> {
    TREES
      .lock()
      .unwrap()
      .get_mut(&tid)
      .and_then(|t|{
        let i = t.outline
         .iter()
         .skip(1)
         .position(|x|x.label() == label)
         .map(|x|x+1)
         .unwrap_or(0);
        if i == 0 { return None }
        move_node_left(&mut t.outline, i)
      })
  }
  #[pyfn(m)]
  #[pyo3(name="move_node_up", text_signature="(tid, label)")]
  /// Moves node up idetified by given label in the outline
  /// idetified by tid. If the movement would result in the
  /// invalid outline, returns None, without making any change.
  /// 
  /// Returns string representation of the performed changes, that
  /// can be later used for undo/redo operations.
  ///
  /// Returns None if the outline or node is missing
  ///
  fn pymove_node_up(_py: Python, tid:usize, label:u32) -> Option<String> {
    TREES
      .lock()
      .unwrap()
      .get_mut(&tid)
      .and_then(|t|{
        let i = t.outline
         .iter()
         .skip(1)
         .position(|x|x.label() == label)
         .unwrap_or(0);
        if i == 0 {return None}
        move_node_up(&mut t.outline, i+1)
      })
  }
  #[pyfn(m)]
  #[pyo3(name="move_node_down", text_signature="(tid, label)")]
  /// Moves node down idetified by given label in the outline
  /// idetified by tid. If the movement would result in the
  /// invalid outline, returns None, without making any change.
  /// 
  /// Returns string representation of the performed changes, that
  /// can be later used for undo/redo operations.
  ///
  /// Returns None if the outline or node is missing
  ///
  fn pymove_node_down(_py: Python, tid:usize, label:u32) -> Option<String> {
    TREES
      .lock()
      .unwrap()
      .get_mut(&tid)
      .and_then(|t|{
        let i = t.outline
         .iter()
         .skip(1)
         .position(|x|x.label() == label)
         .unwrap_or(0);
        if i == 0 {return None}
        move_node_down(&mut t.outline, i+1)
      })
  }
  #[pyfn(m)]
  #[pyo3(name="node_at", text_signature="(tid, i)")]
  /// Returns a copy of node located on the given index i inside the outline
  /// idetified by tid.
  /// 
  /// Returns None if the tid outline is missing or there isn't a node at 
  /// the given index i
  fn node_at(_py: Python, tid:usize, i:usize) -> PyResult<Option<(u8, VData)>> {
    let res = TREES.lock().unwrap().get(&tid).map(|t|{
      let x = t.outline[i];
      let v = t.nodes[x.ignx() as usize].clone();
      (x.level(), v)
    });
    Ok(res)
  }
  #[pyfn(m)]
  #[pyo3(name="outline_from_file", text_signature="(fname)")]
  /// Loads an outline from the given file, which should be in
  /// Leo exteranl at-file format.
  ///
  /// In case of errors the resulting outline will be empty.
  ///
  /// Raises IOError if the file doesn't exist.
  ///
  fn outline_from_file(_py: Python, fname:&str) -> PyResult<usize> {
    match from_derived_file(&Path::new(fname)) {
      Ok((outline, nodes)) => {
        let t = Tree {outline, nodes};
        let mut m = TREES.lock().unwrap();
        let tid = m.len();
        m.insert(tid, t);
        Ok(tid)
      },
      Err(e) => Err(PyIOError::new_err(e.to_string()))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="outline_from_leo_str")]
  fn outline_from_leo_str(_py: Python, txt:&str) -> PyResult<usize> {
    let (outline, nodes) = from_leo_content(txt);
    let t = Tree {outline, nodes};
    let mut m = TREES.lock().unwrap();
    let tid = m.len();
    m.insert(tid, t);
    Ok(tid)
  }
  #[pyfn(m)]
  #[pyo3(name="outline_from_leo_file")]
  fn outline_from_leo_file(_py: Python, txt:&str) -> PyResult<usize> {
    match from_leo_file(&Path::new(txt)) {
      Ok((outline, nodes)) => {
        let t = Tree {outline, nodes};
        let mut m = TREES.lock().unwrap();
        let tid = m.len();
        m.insert(tid, t);
        Ok(tid)
      },
      Err(e) => Err(PyValueError::new_err(e.to_string()))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="outline_from_str", text_signature="(txt)")]
  /// Parses given txt as Leo external at-file format,
  /// builds the outline and returns a handle to this outline.
  ///
  /// In case of errors the resulting outline will be empty.
  ///
  fn outline_from_string(_py: Python, txt:&str) -> PyResult<usize> {
    let (outline, nodes) = from_derived_file_content(txt);
    let t = Tree {outline, nodes};
    let mut m = TREES.lock().unwrap();
    let tid = m.len();
    m.insert(tid, t);
    Ok(tid)
  }
  /* // why not use Python zipfile? It would be more flexible!
  #[pyfn(m)]
  #[pyo3(name="outline_from_zipped_leo", text_signature="(arhcive, fname)")]
  fn outline_from_zipped_leo(_py: Python, arch:&str, fname:&str) -> PyResult<usize> {
    match from_zip_archive(&Path::new(arch), fname) {
      Ok((outline, nodes)) => {
        let t = Tree {outline, nodes};
        let mut m = TREES.lock().unwrap();
        let tid = m.len();
        m.insert(tid, t);
        Ok(tid)
      },
      Err(e) => Err(PyIOError::new_err(e.to_string()))
    }
  }
  */
  #[pyfn(m)]
  #[pyo3(name="parent_index")]
  fn parent_index(_py:Python, tid:usize, ni:usize) -> PyResult<usize> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      t.outline.parent_index(ni)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="parents_indexes")]
  fn parents_indexes(_py:Python, tid:usize, ni:usize) -> PyResult<Vec<usize>> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      t.outline.parents_indexes(ni)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="atclean_to_str", text_signature="(tid, ni)")]
  /// Returns `at-clean` representation of the node at the given
  /// index `ni` in the outline identified by `tid`.
  /// 
  /// Raises ValueError if there is no such outline or if there
  /// is no such node.
  ///
  fn pyatclean_to_str(_py: Python, tid: usize, ni: usize) -> PyResult<String> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      if t.outline.len() > ni {
        Ok(atclean_to_string(&t.outline, &t.nodes, ni))
      } else {
        Err(PyValueError::new_err("no such node"))
      }
    }) {
      Some(t) => t,
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="break_link", text_signature="(tid, parent, childIndex)")]
  /// Removes child at the index childIndex from parent in the outline
  /// idetified by tid. If the parent is not present in the outline, or
  /// if it doesn't have child with the childIndex, returns None
  /// else returns string description of the changes made.
  ///
  fn pybreak_link(_py: Python, tid:usize, v1:&VData, ci:usize) -> Option<String> {
    TREES.lock().unwrap().get_mut(&tid).and_then(|t|{
      return break_link( &mut t.outline
                        , v1.ignx
                        , ci
                        )
    })
  }
  #[pyfn(m)]
  #[pyo3(name="create_link", text_signature="(tid, parent, childIndex, child)")]
  /// Links child node at the childIndex to parent in the outline
  /// idetified by tid. If the link was created, returns string
  /// which can be used to undo/redo operation.
  /// 
  /// Returns None if the tid outline is missing or parent and child
  /// are not part of the outline
  ///
  fn pycreate_link(_py: Python, tid:usize, v1:&VData, ci:usize, v2:&VData) -> Option<String> {
    TREES.lock().unwrap().get_mut(&tid).and_then(|t|{
      return create_link( &mut t.outline
                        , v1.ignx as usize
                        , ci
                        , v2.ignx as usize
                        )
    })
  }
  #[pyfn(m)]
  #[pyo3(name="redo", text_signature="(tid, data)")]
  /// Redoes change described by the data parameter
  /// previously undone to the outline identified by tid.
  /// 
  fn pyredo(_py: Python, tid:usize, data:&str) {
    TREES.lock().unwrap().get_mut(&tid).map(|t|{
      for x in data.split('\n') {
        if x.starts_with("ip:") {
          redo_insert_parts(&mut t.outline, x);
        } else if x.starts_with("db:") {
          redo_delete_blocks(&mut t.outline, x);
        } else if x.starts_with("sb:") {
          redo_shift_blocks(&mut t.outline, x);
        } else if x.starts_with("sn:") {
          redo_set_nodes(&mut t.outline, x);
        }
      }
    });
  }
  #[pyfn(m)]
  #[pyo3(name="undo", text_signature="(tid, data)")]
  /// Undoes change described by the data parameter
  /// previously done to the outline identified by tid.
  /// 
  fn pyundo(_py: Python, tid:usize, data:&str) {
    TREES.lock().unwrap().get_mut(&tid).map(|t|{
      for x in data.split('\n').rev() {
        if x.starts_with("ip:") {
          undo_insert_parts(&mut t.outline, x);
        } else if x.starts_with("db:") {
          undo_delete_blocks(&mut t.outline, x);
        } else if x.starts_with("sb:") {
          undo_shift_blocks(&mut t.outline, x);
        } else if x.starts_with("sn:") {
          undo_set_nodes(&mut t.outline, x);
        }
      }
    });
  }
  #[pyfn(m)]
  #[pyo3(name="update_atclean")]
  fn pyupdate_atclean(_py: Python, tid: usize, ni:usize, cont:&str) -> PyResult<()> {
    match TREES.lock().unwrap().get_mut(&tid).map(|t|{
      update_atclean_tree(&t.outline, &mut t.nodes, ni, cont)
    }) {
      Some(()) => Ok(()),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="subtree_size")]
  fn subtree_seize(_py:Python, tid:usize, ni:usize) -> PyResult<usize> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      t.outline.subtree_size(ni)
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="tree_len", text_signature="(tid)")]
  /// Returns the number of different positions in the
  /// outline idetified by tid.
  ///
  /// Raises ValueError if there is no outline (tid).
  ///
  fn tree_len(_py:Python, tid:usize) -> PyResult<usize> {
    match TREES.lock().unwrap().get(&tid).map(|t|{
      let i = t.outline.len();
      if i > 0 {i - 1} else { 0 }
    }){
      Some(x) => Ok(x),
      None => Err(PyValueError::new_err("unknown tree id"))
    }
  }
  #[pyfn(m)]
  #[pyo3(name="update_node", text_signature="(tid, vnode)")]
  /// Updates h, b and flags in the outline
  /// idetified by tid.
  /// 
  /// Returns None if the tid outline is missing or there isn't a node at 
  /// the given index i
  fn update_node(_py: Python, tid:usize, v:&VData) -> PyResult<bool> {
    let res = TREES.lock().unwrap().get_mut(&tid).map(|t|{
      if let Some(mut v0) = t.nodes.get_mut(v.ignx as usize) {
        if v0.gnx == v.gnx {
          v0.h.replace_range(.., &v.h);
          v0.b.replace_range(.., &v.b);
          v0.flags = v.flags;
          return true;
        }
      }
      false
    });
    Ok(res.unwrap_or(false))
  }
  //m.add_wrapped(wrap_pyfunction!(a_function_from_rust))?;
  m.add("VData", _py.get_type::<VData>())?;
  Ok(())
}
