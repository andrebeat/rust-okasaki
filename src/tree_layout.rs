use std::fmt::{Debug, Display};
use std::rc::Rc;

use tree::Tree;
use tree::Tree::{Node, Tip};

// Tree layout algorithm based on the following paper:
// "FUNCTIONAL PEARLS - Drawing Trees by Andrew J. Kennedy (1996)"
// (http://research.microsoft.com/en-us/um/people/akenn/fun/DrawingTrees.pdf)

pub fn pretty_print<T: Clone + Display + Debug>(t: &Tree<(T, f64)>) {
    fn print_spaces(n: usize) {
        for _ in 0..n {
            print!(" ");
        }
    }

    fn aux<T: Clone + Display + Debug>(current: Vec<Tree<(T, f64)>>, mut next: Vec<Tree<(T, f64)>>, offset: f64) {
        // FIXME: does not take into account displacement due to the size of the printed node value
        let factor = 1.0; // FIXME: hardcoded offset factor

        match current.first() {
            None =>
                if !next.is_empty() {
                    println!("");
                    print!("   "); // FIXME: hardcoded offset
                    aux(next, vec![], 0.0);
                },
            Some(t) => {
                let o =
                    match *t {
                    Tip => offset,
                    Node(ref l, (ref v, x), ref r) => {
                        let sp = x - offset;
                        print_spaces((sp * factor) as usize);

                        print!("{}", v);
                        next.push((**l).clone());
                        next.push((**r).clone());

                        sp
                    }
                    };
                aux(current[1..].to_vec(), next, o);
            }
        }
    }

    aux(vec![], vec![(*t).clone()], 0.0);
}

pub fn move_by_leftmost<T: Clone>(t: &Tree<(T, f64)>) -> Tree<(T, f64)> {
    fn move_by_offset<T: Clone>(t: &Tree<(T, f64)>, o: f64) -> Tree<(T, f64)> {
        match *t {
            Tip => Tip,
            Node(ref l, (ref v, x), ref r) => {
                Node(Rc::new(move_by_offset(&*l, o)), (v.clone(), x + o), Rc::new(move_by_offset(&*r, o)))
            }
        }
    }

    fn find_leftmost<T>(t: &Tree<(T, f64)>, current: f64) -> f64 {
        match *t {
            Tip => current,
            Node(ref l, (_, x), ref r) => {
                let n = x.min(current);
                find_leftmost(l, n).min(find_leftmost(r, n))
            }
        }
    }

    move_by_offset(t, -find_leftmost(t, 0.0))
}

pub fn absolute_new<T: Clone + Display>(t: &Tree<(T, f64)>) -> Tree<(T, f64)> {
    fn aux<T: Clone + Display>(t: &Tree<(T, f64)>, d: f64, vd: f64) -> Tree<(T, f64)> {
        match *t {
            Tip => Tip,
            Node(ref l, (ref v, x), ref r) => {
                let a = d + x;
                let d = a + (a.signum() * vd);
                let nvd = vd + (format!("{}", v).len() as f64);
                println!("nvd: {}", nvd);
                Node(Rc::new(aux(&*l, a, nvd)), (v.clone(), d), Rc::new(aux(&*r, a, nvd)))
            }
        }
    }

    aux(t, 0.0, 0.0)
}

pub fn absolute<T: Clone + Display>(t: &Tree<(T, f64)>) -> Tree<(T, f64)> {
    fn aux<T: Clone + Display>(t: &Tree<(T, f64)>, d: f64) -> Tree<(T, f64)> {
        match *t {
            Tip => Tip,
            Node(ref l, (ref v, x), ref r) => {
                let a = d + x;
                Node(Rc::new(aux(&*l, a)), (v.clone(), a), Rc::new(aux(&*r, a)))
            }
        }
    }

    aux(t, 0.0)
}

pub fn design<T: Clone>(t: &Tree<T>) -> Tree<(T, f64)> {
    fn aux<T: Clone>(t: &Tree<T>) -> (Tree<(T, f64)>, Extent) {
        match *t {
            Tip => (Tip, vec![]),
            Node(ref l, ref v, ref r) => {
                let (trees, extents): (Vec<Tree<(T, f64)>>, Vec<Extent>) = vec![aux(l), aux(r)].into_iter().unzip();
                let positions = fit_list(extents.clone());
                let ptrees: Vec<Tree<(T, f64)>> = trees.iter().zip(positions.iter()).map(|f| {
                    let (t, x) = f;
                    move_tree(t, *x)
                }).collect();
                let pextents = extents.into_iter().zip(positions.iter()).map(|f| {
                    let (e, x) = f;
                    move_extent(e, *x)
                }).collect();

                let mut resultextent = merge_extents(pextents);
                resultextent.insert(0, (0.0, 0.0));
                let resulttree = Node(Rc::new(ptrees[0].clone()), ((*v).clone(), 0.0), Rc::new(ptrees[1].clone()));

                (resulttree, resultextent)
            }
        }

    }
    aux(t).0
}

fn move_tree<T: Clone>(t: &Tree<(T, f64)>, x1: f64) -> Tree<(T, f64)> {
    match *t {
        Tip => Tip,
        Node(ref l, (ref v, x), ref r) => Node(l.clone(), (v.clone(), x + x1), r.clone())
    }
}

type Extent = Vec<(f64, f64)>;

fn fit_list(es: Vec<Extent>) -> Vec<f64> {
    fn mean(x: f64, y: f64) -> f64 {
        (x + y) / 2.0
    }

    fit_list_left(es.clone()).iter().zip(
        fit_list_right(es.clone()).iter())
        .map(|x| mean(*x.0, *x.1)).collect()
}

fn fit_list_right(es: Vec<Extent>) -> Vec<f64> {
    fn flip_extent(e: Extent) -> Extent {
        e.iter().map(|&x| {
            let (p, q) = x;
            (-q, -p)
        }).collect()
    }

    fit_list_left(
        es.iter().rev().map(|e| flip_extent((*e).clone())).collect()).iter()
        .map(|&f| -f).rev().collect()
}

fn fit_list_left(es: Vec<Extent>) -> Vec<f64> {
    fn aux(es: Vec<Extent>, acc: Extent) -> Vec<f64> {
        match es.first() {
            Some(e) => {
                let x = fit(acc.clone(), e.clone());
                let mut r = aux(es[1..].to_vec(), merge_extent(acc.clone(), move_extent(e.clone().to_vec(), x)));
                r.insert(0, x);
                r
            },
            None => vec![]
        }
    }
    aux(es, vec![])
}

fn rmax(p: f64, q: f64) -> f64 {
    if p > q { p } else { q }
}

fn fit(e1: Extent, e2: Extent) -> f64 {
    match (e1.first(), e2.first()) {
        (Some(&(_, p)), Some(&(q, _))) =>
            rmax(fit(e1[1..].to_vec(), e2[1..].to_vec()), p - q + 1.0),
        (_, _) => 0.0
    }
}

fn merge_extent(e1: Extent, e2: Extent) -> Extent {
    match (e1.first(), e2.first()) {
        (None, _) => e2,
        (_, None) => e1,
        (Some(&(p, _)), Some(&(_, q))) => {
            let mut m = merge_extent(e1[1..].to_vec(), e2[1..].to_vec());
            m.insert(0, (p, q));
            m
        }
    }
}

fn merge_extents(es: Vec<Extent>) -> Extent {
    es.iter().fold(vec![], |acc, e| {
        merge_extent(acc, e.clone())
    })
}

fn move_extent(e: Extent, x: f64) -> Extent {
    e.iter().map(|&e| {
        let (p, q) = e;
        (p + x, q + x)
    }).collect()
}
