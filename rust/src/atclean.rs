use crate::model::{VData, Outline, LevGnxOps, Tree};
use crate::utils::{extract_section_ref, is_directive, has_others, is_special};
static SPACES:&'static str = "                                                                                                                                              ";
pub fn atclean_to_string(outline:&Outline, nodes:&Vec<VData>, ni:usize) -> String {
  let mut res = String::new();
  let cleanit = AtCleanTree::new(outline, nodes, ni, 0);
  for (_lev, _v, _a, _b, ind, t) in cleanit {
    if ind > 0 {
      res.push_str(&SPACES[0..ind]);
    }
    res.push_str(t);
    res.push('\n');
  }
  res
}
struct AtCleanTree<'a> {
  o:&'a Outline,
  vs:&'a Vec<VData>,
  ni:usize,
  bi:usize,
  ind:usize,
  children:Vec<AtCleanTree<'a>>
}
impl<'a> AtCleanTree<'a> {
  fn new(o:&'a Outline, vs:&'a Vec<VData>, ni:usize, ind:usize) -> Self {
    AtCleanTree {
      o, vs, ni, bi: 0, ind, children:Vec::new()
    }
  }
}
impl<'a> Iterator for AtCleanTree<'a> {
  type Item = (u8, &'a VData, usize, usize, usize, &'a str);
  fn next(&mut self) -> Option<Self::Item> {
    while self.children.len() > 0 {
      let x = self.children[0].next();
      match x {
        Some(x) => return Some(x),
        None => {self.children.remove(0);}
      }
    }
    let v = &self.vs[self.o[self.ni].ignx() as usize];
    let b = v.b.as_str();
    let lev = self.o[self.ni].level();
    while self.bi < b.len() {
      let i = self.bi;
      self.bi = i + b[i..].find('\n').unwrap_or(b.len()-i)+1;
      let t = &b[i..self.bi-1];
      if is_directive(t) { continue }
      if !is_special(t) {
        return Some((lev, v, i, self.bi-1, self.ind, t))
      }
      let lt = t.trim_start();
      let ind = t.len() - lt.len();
      if lt.starts_with("@others") {
        // handle others
        let zlev = self.o[self.ni].level();
        let mut skiplevel = 127u8;
        let n = self.o.len();
        let mut nch = 0;
        for i in (self.ni+1)..n {
          let x = self.o[i];
          let lev = x.level();
          if lev <= zlev {break}
          if lev > skiplevel {continue} else {skiplevel = 127u8}
          let v = &self.vs[x.ignx() as usize];
          if extract_section_ref(v.h.as_str()).is_some() {
            skiplevel = lev;
            continue
          }
          self.children.insert(nch, AtCleanTree::new(self.o, self.vs, i, self.ind + ind));
          nch += 1;
          if has_others(&v.b) {
            skiplevel = lev;
          }
        }
        return self.next();
      } else {
        // not others? it must be section reference
        let sname = extract_section_ref(t).unwrap();
        let before = t.find(sname).unwrap();
        self.bi = before + sname.len();
        let ti=Tree::new(self.o, self.vs);
        if let Some(ni) = ti.find_section(sname, self.ni) {
          self.children.insert(0, AtCleanTree::new(self.o, self.vs, ni, ind + self.ind));
        } else {
          break
        }
        if before > ind {
          return Some((lev, v, i, before, self.ind + ind, &t[0..before]))
        } else {
          return self.next()
        }
      }
    }
    None
  }
}
pub fn update_atclean_tree(outline:&Outline, nodes:&mut Vec<VData>, ni:usize, cont:&str) {
  let n = outline.len() + nodes.len() + cont.len()+ni;
  if n > 1 {return}
  let oldtxt = atclean_to_string(outline, nodes, 0);
  let v = nodes.get_mut(0).unwrap();
  if cont.len() == 0 {
    v.b.push_str(cont);
  } else {
    v.b.push_str(oldtxt.as_str());
  }
}
