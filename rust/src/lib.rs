mod model;
mod utils;
mod parsing;
use std::collections::HashMap;
use std::sync::{Mutex};
pub use crate::parsing::{ldf_parse,from_derived_file_content, from_derived_file,
                         from_leo_file, from_leo_content, load_with_external_files};
pub use crate::utils::{b64int, b64str};
pub use crate::model::{VData, Outline, LevGnx, LevGnxOps, gnx_index, find_derived_files};
use pyo3::prelude::*;
use pyo3::PyIterProtocol;
use pyo3::{wrap_pyfunction};
use pyo3::type_object::PyTypeObject;
//use xml::reader::{ParserConfig, XmlEvent};
use std::path::{PathBuf};
#[macro_use]
extern crate lazy_static;

#[pyfunction]
pub fn a_function_from_rust() -> PyResult<i32> {
    Ok(42)
}
struct Tree {
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
    fn __next__(mut slf:PyRefMut<Self>) -> PyResult<Option<(u8, VData)>> {
        let m = TREES.lock().unwrap();
        let res = m.get(&slf.tree_id).map(|t|{
            let n = t.outline.len();
            if slf.index < n {
                let levgnx = t.outline[slf.index];
                let i = levgnx.ignx() as usize;
                Some((levgnx.level(), t.nodes[i].clone()))
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
    #[pyfn(m, "outline_from_str")]
    fn outline_from_string(_py: Python, txt:&str) -> PyResult<usize> {
        let (outline, nodes) = from_derived_file_content(txt);
        let t = Tree {outline, nodes};
        let mut m = TREES.lock().unwrap();
        let tid = m.len();
        m.insert(tid, t);
        Ok(tid)
    }
    #[pyfn(m, "outline_from_file")]
    fn outline_from_file(_py: Python, txt:&str) -> PyResult<usize> {
        match from_derived_file(PathBuf::from(txt)) {
            Ok((outline, nodes)) => {
                let t = Tree {outline, nodes};
                let mut m = TREES.lock().unwrap();
                let tid = m.len();
                m.insert(tid, t);
                Ok(tid)
            },
            Err(e) => Err(pyo3::exceptions::IOError::py_err(e.to_string()))
        }
    }
    #[pyfn(m, "outline_from_leo_file")]
    fn outline_from_leo_file(_py: Python, txt:&str) -> PyResult<usize> {
        match from_leo_file(PathBuf::from(txt)) {
            Ok((outline, nodes)) => {
                let t = Tree {outline, nodes};
                let mut m = TREES.lock().unwrap();
                let tid = m.len();
                m.insert(tid, t);
                Ok(tid)
            },
            Err(e) => Err(pyo3::exceptions::IOError::py_err(e.to_string()))
        }
    }
    #[pyfn(m, "outline_from_leo_str")]
    fn outline_from_leo_str(_py: Python, txt:&str) -> PyResult<usize> {
        let (outline, nodes) = from_leo_content(txt);
        let t = Tree {outline, nodes};
        let mut m = TREES.lock().unwrap();
        let tid = m.len();
        m.insert(tid, t);
        Ok(tid)
    }
    #[pyfn(m, "load_leo")]
    fn load_leo(_py: Python, fname:&str) -> PyResult<usize> {
        let tid = match load_with_external_files(fname) {
            Ok((outline, nodes)) => {
                let t = Tree {outline, nodes};
                let mut m = TREES.lock().unwrap();
                let tid = m.len();
                m.insert(tid, t);
                tid
            },
            Err(e) => {
                println!("{}", e);
                0
            }
        };
        Ok(tid)
    }
    #[pyfn(m, "node_at")]
    fn node_at(_py: Python, tid:usize, i:usize) -> PyResult<Option<(u8, VData)>> {
        let res = TREES.lock().unwrap().get(&tid).map(|t|{
            let x = t.outline[i];
            let v = t.nodes[x.ignx() as usize].clone();
            (x.level(), v)
        });
        Ok(res)
    }
    #[pyfn(m, "iternodes")]
    fn iternodes(_py:Python, tid:usize) -> PyResult<TreeIterator> {
        Ok(TreeIterator {tree_id:tid, index:0})
    }
    #[pyfn(m, "drop_tree")]
    fn drop_tree(_py:Python, tid:usize) -> PyResult<bool> {
        Ok(TREES.lock().unwrap().remove(&tid).is_some())
    }
    #[pyfn(m, "at_files")]
    fn at_files(_py:Python, tid:usize, folder:&str) -> PyResult<Vec<(String, usize)>> {
        match TREES.lock().unwrap().get(&tid).map(|t|{
            find_derived_files(folder, &t.outline, &t.nodes)
        }){
            Some(x) => Ok(x),
            None => Err(pyo3::exceptions::ValueError::py_err("unknown tree id"))
        }
    }
    #[pyfn(m, "tree_len")]
    fn tree_len(_py:Python, tid:usize) -> PyResult<usize> {
        match TREES.lock().unwrap().get(&tid).map(|t|{
            t.outline.len() - 1
        }){
            Some(x) => Ok(x),
            None => Err(pyo3::exceptions::ValueError::py_err("unknown tree id"))
        }
    }
    m.add_wrapped(wrap_pyfunction!(a_function_from_rust))?;
    m.add("VData", <VData as PyTypeObject>::type_object())?;
    Ok(())
}
