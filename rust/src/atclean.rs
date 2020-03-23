use crate::model::{VData, Outline}; //, OutlineOps, LevGnx, LevGnxOps};
pub fn atclean_to_string(outline:&Outline, nodes:&Vec<VData>, ni:usize) -> String {
  if nodes.len() > 0 && ni < outline.len() {
    String::new()
  } else {
    String::new()
  }
}
pub fn update_atclean_tree(outline:&Outline, nodes:&mut Vec<VData>, cont:&str) {
  let oldtxt = atclean_to_string(outline, nodes, 0);
  let v = nodes.get_mut(0).unwrap();
  if cont.len() == 0 {
    v.b.push_str(cont);
  } else {
    v.b.push_str(oldtxt.as_str());
  }
}
