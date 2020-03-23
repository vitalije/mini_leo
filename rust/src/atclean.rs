use crate::model::{VData, Outline}; //, OutlineOps, LevGnx, LevGnxOps};
pub fn update_atclean_tree(outline:&Outline, nodes:&mut Vec<VData>, cont:&str) {
  let _v = nodes.get_mut(0);
  if cont.len() == 0 || outline.len() == 0 {
    for v in _v {
      v.b.clear();
    }
  }
}
