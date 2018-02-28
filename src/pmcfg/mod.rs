use std::collections::BTreeMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::slice;
use std::vec;

use util::tree::GornTree;

mod from_str;
pub mod negra;

/// Variable or terminal symbol in an MCFG.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum VarT<T> {
    /// `Var(i, j)` represents the `j`th component of the `i`th successor.
    /// Indexing starts from `0`.
    Var(usize, usize),
    T(T),
}

/// Composition function in an MCFG.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Composition<T> {
    pub composition: Vec<Vec<VarT<T>>>,
}

impl<T> From<Vec<Vec<VarT<T>>>> for Composition<T> {
    fn from(encapsulated_value: Vec<Vec<VarT<T>>>) -> Self {
        Composition { composition: encapsulated_value }
    }
}

impl<T> IntoIterator for Composition<T> {
    type Item = Vec<VarT<T>>;
    type IntoIter = vec::IntoIter<Vec<VarT<T>>>;

    fn into_iter(self) -> vec::IntoIter<Vec<VarT<T>>> {
        self.composition.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Composition<T> {
    type Item = &'a Vec<VarT<T>>;
    type IntoIter = slice::Iter<'a, Vec<VarT<T>>>;

    fn into_iter(self) -> slice::Iter<'a, Vec<VarT<T>>> {
        (&self.composition).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Composition<T> {
    type Item = &'a mut Vec<VarT<T>>;
    type IntoIter = slice::IterMut<'a, Vec<VarT<T>>>;

    fn into_iter(self) -> slice::IterMut<'a, Vec<VarT<T>>> {
        (&mut self.composition).into_iter()
    }
}

/// Rule of a weighted MCFG.
#[derive(Debug, PartialOrd, Ord, Clone)]
pub struct PMCFGRule<N, T, W> {
    pub head: N,
    pub tail: Vec<N>,
    pub composition: Composition<T>,
    pub weight: W,
}

impl<N, T, W> PMCFGRule<N, T, W>
    where T: Clone,
          W: Clone,
{
    pub fn map_nonterminals<F, M>(&self, f: F) -> PMCFGRule<M, T, W>
        where F: Fn(&N) -> M,
    {
        PMCFGRule {
            head: f(&self.head),
            tail: self.tail.iter().map(f).collect(),
            composition: self.composition.clone(),
            weight: self.weight.clone()
        }
    }
}

/// A weighted MCFG.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PMCFG<N, T, W> {
    pub initial: Vec<N>,
    pub rules: Vec<PMCFGRule<N, T, W>>,
}

impl<N: Hash, T: Hash, W> Hash for PMCFGRule<N, T, W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.head.hash(state);
        self.tail.hash(state);
        self.composition.hash(state);
    }
}

impl<N: PartialEq, T: PartialEq, W> PartialEq for PMCFGRule<N, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head && self.tail == other.tail && self.composition == other.composition
    }
}

impl<N: Eq, T: Eq, W> Eq for PMCFGRule<N, T, W> {}

impl<T: fmt::Display> fmt::Display for Composition<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buffer = "".to_string();

        let mut iter0 = self.composition.iter().peekable();
        let mut iter1;

        buffer.push_str("[");
        while let Some(proj) = iter0.next() {
            iter1 = proj.into_iter().peekable();
            buffer.push_str("[");
            while let Some(vart) = iter1.next() {
                buffer.push_str(format!("{}", vart).as_str());
                if iter1.peek().is_some() {
                    buffer.push_str(", ");
                }
            }
            buffer.push_str("]");
            if iter0.peek().is_some() {
                buffer.push_str(", ");
            }
        }
        buffer.push_str("]");

        write!(f, "{}", buffer)
    }
}

impl<T: fmt::Display> fmt::Display for VarT<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VarT::Var(i, j) => {
                write!(f, "Var {} {}", i, j)
            },
            VarT::T(ref x) => {
                write!(f, "T \"{}\"", x)
            },
        }
    }
}

impl<N: fmt::Display, T: fmt::Display, W: fmt::Display> fmt::Display for PMCFGRule<N, T, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
           let mut buffer = "".to_string();

           let mut iter = self.tail.iter().peekable();

           buffer.push_str("(");
           while let Some(nt) = iter.next() {
               buffer.push_str(format!("\"{}\"", nt).as_str());
               if iter.peek().is_some() {
                   buffer.push_str(", ");
               }
           }
           buffer.push_str(")");

        write!(f, "\"{}\" → {} {}  # {}", self.head, self.composition, buffer, self.weight)
        // write!(f, "\"{}\" → {}", self.head, self.composition)
    }
}

impl<N: fmt::Display, T: fmt::Display, W: fmt::Display> fmt::Display for PMCFG<N, T, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buffer = "".to_string();

        let mut iter = self.initial.iter().peekable();

        buffer.push_str("initial: [");
        while let Some(nt) = iter.next() {
            buffer.push_str(format!("\"{}\"", nt).as_str());
            if iter.peek().is_some() {
                buffer.push_str(", ");
            }
        }
        buffer.push_str("]\n\n");

        for r in &self.rules {
            buffer.push_str(format!("{}\n", r).as_str());
        }

        write!(f, "{}", buffer)
    }
}

pub fn evaluate<T: Clone + fmt::Debug>(term_map: &GornTree<Composition<T>>)
        -> Composition<T>
{
    evaluate_pos(term_map, vec![])
}

pub fn evaluate_pos<T>(term_map: &GornTree<Composition<T>>, address: Vec<usize>)
        -> Composition<T>
    where T: Clone + fmt::Debug,
{
    let unexpanded_composition = term_map.get(&address).unwrap();
    let mut expanded_nonterminals: BTreeMap<_, Vec<Vec<VarT<T>>>> = BTreeMap::new();
    let mut expanded_composition = Vec::new();

    for component in unexpanded_composition {
        let mut expanded_component = Vec::new();

        for variable in component {
            match variable {
                &VarT::Var(num_nonter, num_compon) => {
                    if !expanded_nonterminals.contains_key(&num_nonter) {
                        let mut child_address = address.clone();
                        child_address.push(num_nonter);
                        let nonter_compos = evaluate_pos(term_map, child_address).composition;
                        expanded_nonterminals.insert(num_nonter, nonter_compos);
                    }

                    let nonter_compos = expanded_nonterminals.get(&num_nonter).unwrap();

                    if let Some(compon) = nonter_compos.get(num_compon) {
                        for terminal in compon {
                            expanded_component.push(terminal.clone());
                        }
                    } else {
                        panic!("{:?}: use of {}-th component of nonterminal {} that has only {} components!",
                               unexpanded_composition, num_compon, num_nonter, nonter_compos.len());
                    }
                },
                &VarT::T(ref terminal) => {
                    expanded_component.push(VarT::T(terminal.clone()));
                },
            }
        }

        expanded_composition.push(expanded_component);
    }

    Composition::from(expanded_composition)
}

pub fn to_term<H, T, W>(tree_map: &GornTree<PMCFGRule<H, T, W>>)
        -> (GornTree<Composition<T>>, GornTree<H>)
    where H: Clone,
          T: Clone,
{
    let mut term_map = GornTree::new();
    let mut head_map = GornTree::new();

    for (address, &PMCFGRule { ref head, tail: _, ref composition, weight: _ }) in tree_map {
        term_map.insert(address.clone(), composition.clone());
        head_map.insert(address.clone(), head.clone());
    }

    (term_map, head_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use self::VarT::{Var, T};
    use std::str::FromStr;

    pub fn example_tree_map() -> GornTree<PMCFGRule<String, char, usize>> {
        let mut tree_map = GornTree::new();

        tree_map.insert(vec![], PMCFGRule::from_str(
            "S -> [[Var 0 0, Var 1 0, Var 0 1, Var 1 1]] (A, B) # 1"
        ).unwrap());
        tree_map.insert(vec![0], PMCFGRule::from_str(
            "A -> [[Var 1 0, Var 0 0], [Var 2 0, Var 0 1]] (A, a, c) # 1"
        ).unwrap());
        tree_map.insert(vec![0, 0], PMCFGRule::from_str(
            "A -> [[Var 1 0, Var 0 0], [Var 2 0, Var 0 1]] (A, a, c) # 1"
        ).unwrap());
        tree_map.insert(vec![0, 0, 0], PMCFGRule::from_str(
            "A -> [[], []] () # 1"
        ).unwrap());
        tree_map.insert(vec![0, 0, 1], PMCFGRule::from_str(
            "a -> [[T a]] () # 1"
        ).unwrap());
        tree_map.insert(vec![0, 0, 2], PMCFGRule::from_str(
            "c -> [[T c]] () # 1"
        ).unwrap());
        tree_map.insert(vec![0, 1], PMCFGRule::from_str(
            "a -> [[T a]] () # 1"
        ).unwrap());
        tree_map.insert(vec![0, 2], PMCFGRule::from_str(
            "c -> [[T c]] () # 1"
        ).unwrap());
        tree_map.insert(vec![1], PMCFGRule::from_str(
            "B -> [[Var 1 0, Var 0 0], [Var 2 0, Var 0 1]] (B, b, d) # 1"
        ).unwrap());
        tree_map.insert(vec![1, 0], PMCFGRule::from_str(
            "B -> [[], []] () # 1"
        ).unwrap());
        tree_map.insert(vec![1, 1], PMCFGRule::from_str(
            "b -> [[T b]] () # 1"
        ).unwrap());
        tree_map.insert(vec![1, 2], PMCFGRule::from_str(
            "d -> [[T d]] () # 1"
        ).unwrap());

        tree_map
    }

    #[test]
    fn test_evaluate() {
        let tree_map = example_tree_map();
        let mut term_map = GornTree::new();

        for (address, PMCFGRule { head: _, tail: _, composition, weight: _ }) in tree_map {
            term_map.insert(address, composition);
        }

        let expanded_compos = Composition::from(vec![
            vec![T('a'), T('a'), T('b'), T('c'), T('c'), T('d')]
        ]);

        assert_eq!(expanded_compos, evaluate(&term_map));
    }

    #[test]
    #[should_panic(expected =
        "[[Var(0, 0), Var(0, 1)]]: use of 1-th component of nonterminal 0 that has only 1 components!"
    )]
    fn test_evaluate_invalid_composition() {
        let mut term_map = GornTree::new();
        term_map.insert(vec![], Composition::from(vec![
            vec![Var(0, 0), Var(0, 1)]
        ]));
        term_map.insert(vec![0], Composition::from(vec![
            vec![T('a')]
        ]));

        evaluate(&term_map);
    }

    #[test]
    fn test_to_term() {
        let mut tree_map: GornTree<PMCFGRule<String, char, usize>> = GornTree::new();

        tree_map.insert(vec![], PMCFGRule::from_str(
            "A -> [[Var 0 0, T a, Var 0 1, T b]] (B) # 1"
        ).unwrap());
        tree_map.insert(vec![0], PMCFGRule::from_str(
            "B -> [[Var 1 0], [T c]] (C) # 1"
        ).unwrap());
        tree_map.insert(vec![0, 1], PMCFGRule::from_str(
            "C -> [[], []] () # 1"
        ).unwrap());

        let mut term_map = GornTree::new();
        term_map.insert(vec![], Composition::from(vec![
            vec![Var(0, 0), T('a'), Var(0, 1), T('b')]
        ]));
        term_map.insert(vec![0], Composition::from(vec![
            vec![Var(1, 0)],
            vec![T('c')]
        ]));
        term_map.insert(vec![0, 1], Composition::from(vec![
            vec![],
            vec![]
        ]));

        let mut head_map = GornTree::new();
        head_map.insert(vec![], String::from("A"));
        head_map.insert(vec![0], String::from("B"));
        head_map.insert(vec![0, 1], String::from("C"));

        assert_eq!((term_map, head_map), to_term(&tree_map));
    }

    #[test]
    fn test_to_term_inverse() {
        let tree_map = example_tree_map();
        let (term_map, head_map) = to_term(&tree_map);
        let mut reconstructed_tree_map = GornTree::new();

        for (address, composition) in term_map {
            let head = head_map.get(&address).unwrap().clone();
            reconstructed_tree_map.insert(address, PMCFGRule {
                head, tail: vec![], composition, weight: 0
            });
        }

        for (address, rule) in tree_map {
            let PMCFGRule { head: ref orig_head, tail: _, composition: ref orig_composition, weight: _ } =
                rule;
            let &PMCFGRule { ref head, tail: _, ref composition, weight: _ } =
                reconstructed_tree_map.get(&address).unwrap();
            assert_eq!(orig_head, head);
            assert_eq!(orig_composition, composition);
        }
    }
}
