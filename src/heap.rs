use std::collections::VecDeque;
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;

pub trait Heap<T: Ord> {
    fn empty() -> Self;
    fn is_empty(&self) -> bool;

    fn merge(&self, &Self) -> Self;
    fn insert(&self, T) -> Self;

    fn find_min(&self) -> T;
    fn delete_min(&self) -> Self;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeftistHeap<T> {
    Tip,
    Node(usize, T, Rc<LeftistHeap<T>>, Rc<LeftistHeap<T>>),
}

use heap::LeftistHeap::{Tip, Node};

impl<T: Ord + Clone> Heap<T> for LeftistHeap<T> {
    fn empty() -> LeftistHeap<T> {
        Tip
    }

    fn is_empty(&self) -> bool {
        match *self {
            Tip => true,
            _ => false
        }
    }

    fn merge(&self, h: &LeftistHeap<T>) -> LeftistHeap<T> {
        fn rank<T>(h: &LeftistHeap<T>) -> usize {
            match *h {
                Tip => 0,
                Node(r, _, _, _) => r
            }
        }

        fn make_node<T: Clone>(x: T, l: Rc<LeftistHeap<T>>, r: Rc<LeftistHeap<T>>) -> LeftistHeap<T> {
            if rank(&l) >= rank(&r) { Node(rank(&r) + 1, x.clone(), l.clone(), r.clone()) }
            else { Node(rank(&l) + 1, x.clone(), r.clone(), l.clone()) }
        }

        match (self, h) {
            (e, &Tip) => e.clone(),
            (&Tip, e) => e.clone(),
            (&Node(_, ref x, ref l1, ref r1), &Node(_, ref y, ref l2, ref r2)) => {
                if *x <= *y {
                    make_node(x.clone(), l1.clone(), Rc::new(r1.merge(h)))
                } else {
                    make_node(y.clone(), l2.clone(), Rc::new(self.merge(r2)))
                }
            }
        }
    }

    fn insert(&self, x: T) -> LeftistHeap<T> {
        let h = Node(1, x, Rc::new(Tip), Rc::new(Tip));
        h.merge(self)
    }

    fn find_min(&self) -> T {
        match *self {
            Tip => panic!("empty heap"),
            Node(_, ref x, _, _) => x.clone()
        }
    }

    fn delete_min(&self) -> LeftistHeap<T> {
        match *self {
            Tip => panic!("empty heap"),
            Node(_, _, ref l, ref r) => l.merge(r)
        }
    }
}

impl<T: Display> Display for LeftistHeap<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        fn aux<T: Display>(f: &mut Formatter, t: &LeftistHeap<T>, right: bool, indent: &str) -> Result<(), Error> {
            match *t {
                Node(ref rank, ref x, ref l, ref r) => {
                    try!(aux(f, r, true, &(indent.to_string() + if right { "        " } else { " |      " })));

                    try!(write!(f, "{}", indent));
                    try!(if right { write!(f, "{}", " /") } else { write!(f, "{}", " \\") });
                    try!(write!(f, "{}", "----- "));

                    try!(writeln!(f, "(#{}, {})", rank, x));

                    aux(f, l, false, &(indent.to_string() + if right { " |      " } else { "        " }))
                },
                Tip => {
                    try!(write!(f, "{}", indent));
                    try!(if right { write!(f, "{}", " /") } else { write!(f, "{}", " \\") });
                    try!(write!(f, "{}", "----- "));

                    writeln!(f, "{}", "()")
                }
            }
        }

        match *self {
            Node(ref rank, ref x, ref l, ref r) => {
                try!(aux(f, r, true, ""));
                try!(writeln!(f, "(#{}, {})", rank, x));
                aux(f, l, false, "")
            },
            Tip => Result::Ok(())
        }
    }
}

macro_rules! vecdeque {
    ($( $v: expr ),*) => {{
         let vec = ::std::collections::VecDeque::new();
         $( vec.push_back($v); )*
         vec
    }}
}

macro_rules! vecdeque_mut {
    ($( $v: expr ),*) => {{
         let mut vec = ::std::collections::VecDeque::new();
         $( vec.push_back($v); )*
         vec
    }}
}

pub type BinomialHeap<T> = VecDeque<Rc<BinomialTree<T>>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BinomialTree<T>(usize, T, BinomialHeap<T>);

fn link<T: Clone + Ord>(t1: &BinomialTree<T>, t2: &BinomialTree<T>) -> BinomialTree<T> {
    let BinomialTree(r, ref x1, ref c1) = *t1;
    let BinomialTree(_, ref x2, ref c2) = *t2;

    if x1 <= x2 {
        let mut c = c1.clone();
        c.push_front(Rc::new(t2.clone()));
        BinomialTree(r + 1, x1.clone(), c)
    } else {
        let mut c = c2.clone();
        c.push_front(Rc::new(t1.clone()));
        BinomialTree(r + 1, x2.clone(), c)
    }
}

fn rank<T>(t: &BinomialTree<T>) -> usize {
    let BinomialTree(r, _, _) = *t;
    r
}

fn root<T: Clone>(t: &BinomialTree<T>) -> T {
    let BinomialTree(_, ref x, _) = *t;
    x.clone()
}

fn insert_tree<T: Clone + Ord>(h: &BinomialHeap<T>, t: &BinomialTree<T>) -> BinomialHeap<T> {
    match h.front() {
        Some(t2) => {
            let mut h2 = h.clone();

            if rank(t) < rank(t2) {
                h2.push_front(Rc::new(t.clone()));
                h2
            } else {
                insert_tree(&h2.split_off(1), &link(t, t2))
            }
        },
        _ => vecdeque_mut![Rc::new(t.clone())],
    }
}

fn remove_min_tree<T: Clone + Ord>(h: &BinomialHeap<T>) -> (BinomialTree<T>, BinomialHeap<T>) {
    match h.len() {
        0 => panic!("remove tree from empty heap"),
        1 => {
            ((**h.front().unwrap()).clone(), vecdeque![])
        },
        _ => {
            let t = h.front().unwrap();
            let ts = h.clone().split_off(1);

            let (t1, mut ts1) = remove_min_tree(&ts);

            if root(t) < root(&t1) {
                ((**t).clone(), ts)
            } else {
                ts1.push_front(t.clone());
                (t1, ts1)
            }
        }
    }
}

impl<T: Ord + Clone> Heap<T> for BinomialHeap<T> {
    fn empty() -> BinomialHeap<T> {
        vecdeque![]
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn merge(&self, h: &BinomialHeap<T>) -> BinomialHeap<T> {
        match (self.front(), h.front()) {
            (_, None) => self.clone(),
            (None, _) => h.clone(),
            (Some(t1), Some(t2)) => {
                if rank(t1) < rank(t2) {

                    let mut h = self.clone().split_off(1).merge(h);
                    h.push_front(t1.clone());
                    h
                } else if rank(t2) < rank(t1) {

                    let mut h = self.merge(&h.clone().split_off(1));
                    h.push_front(t2.clone());
                    h
                } else {
                    insert_tree(
                        &self.clone().split_off(1).merge(&h.clone().split_off(2)),
                        &link(t1, t2))
                }
            }
        }
    }

    fn insert(&self, x: T) -> BinomialHeap<T> {
        insert_tree(self, &BinomialTree(0, x, vecdeque![]))
    }

    fn find_min(&self) -> T {
        let (t, _) = remove_min_tree(self);
        root(&t)
    }

    fn delete_min(&self) -> BinomialHeap<T> {
        let (BinomialTree(_, _, ts1), ts2) = remove_min_tree(self);
        let ts1: BinomialHeap<T> = ts1.into_iter().rev().collect();
        ts1.merge(&ts2)
    }
}

#[test]
fn leftistheap() {
    let h: LeftistHeap<usize> = Heap::empty();
    let h2: LeftistHeap<usize> = h.insert(10).insert(9).insert(8).insert(11).insert(1).insert(4);

    assert!(h.is_empty());
    assert!(!h2.is_empty());

    assert_eq!(h2.find_min(), 1);
    assert_eq!(h2.delete_min(), h.insert(10).insert(9).insert(8).insert(11).insert(4));
}

#[test]
fn binomialheap() {
    let h: BinomialHeap<usize> = Heap::empty();
    let h2: BinomialHeap<usize> = h.insert(10).insert(9).insert(8).insert(11).insert(1).insert(4);

    assert!(h.is_empty());
    assert!(!h2.is_empty());

    assert_eq!(h2.find_min(), 1);
    assert_eq!(h2.delete_min(), h.insert(10).insert(9).insert(8).insert(11).insert(4));
}
