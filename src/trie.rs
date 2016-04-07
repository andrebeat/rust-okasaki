use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;

use map::Map;

#[derive(Clone, Debug)]
pub enum PatriciaTrie<T> {
    Tip,
    Node { key: String, value: Option<T>, children: HashMap<char, Rc<PatriciaTrie<T>>> }
}

use trie::PatriciaTrie::{Tip, Node};

fn longest_common_prefix(s1: &str, s2: &str) -> usize {
    s1.chars().zip(s2.chars()).take_while(|t| t.0 == t.1).count()
}

fn first_char_unwrap(s: &String) -> char {
    s.chars().nth(0).unwrap()
}

// taken from: http://stackoverflow.com/questions/28392008/more-concise-hashmap-initialization
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}
macro_rules! hashmap_mut {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

impl<T: Clone> Map<String, T> for PatriciaTrie<T> {
    fn empty() -> PatriciaTrie<T> {
        Tip
    }

    fn bind(&self, k: String, v: T) -> PatriciaTrie<T> {
        fn add_children<T: Clone>(t: &PatriciaTrie<T>, k: String, v: T) -> PatriciaTrie<T> {
            match *t {
                Tip => panic!("undefined"),
                Node { ref key, ref value, ref children } =>
                    match children.get(&first_char_unwrap(&k)) {
                        Some(n) => n.bind(k, v),
                        None => Node { key: k, value: Some(v), children: hashmap![] }
                    }
            }
        }

        match *self {
            Tip => Node { key: k, value: Some(v), children: hashmap![] },
            Node { ref key, ref value, ref children } => {
                let i = longest_common_prefix(&k, &key);

                // update an already existing key
                if i == k.len() && i == key.len() {
                    Node { key: k, value: Some(v), children: children.clone() }
                }
                // the existing key is contained in the new key
                else if i == key.len() {
                    let k1 = k[i..].to_string();

                    let mut children = children.clone();
                    children.insert(first_char_unwrap(&k1), Rc::new(add_children(self, k1, v)));

                    Node { key: key.clone(), value: value.clone(), children: children }
                }
                // the new key is contained in the existing key
                else if i == k.len() {
                    let k1 = key[i..].to_string();
                    let children = hashmap_mut![
                        first_char_unwrap(&k1) => Rc::new(Node { key: k1, value: value.clone(), children: children.clone() })];
                    Node { key: k, value: Some(v), children: children }
                }
                // split at longest common prefix
                else {
                    let common = &k[..i];

                    let k1 = key[i..].to_string();
                    let k2 = k[i..].to_string();

                    let children = hashmap_mut![
                        first_char_unwrap(&k1) => Rc::new(Node { key: k1, value: value.clone(), children: children.clone() }),
                        first_char_unwrap(&k2) => Rc::new(Node { key: k2, value: Some(v), children: hashmap![] })];

                    Node { key: common.to_string(), value: None, children: children }
                }
            }
        }
    }

    fn lookup(&self, k: String) -> T {
        match *self {
            Tip => panic!("lookup on empty tree."),
            Node { ref key, ref value, ref children } => {
                if k.starts_with(key) {
                    let k2 = k[key.len()..].to_string();
                    if k2 == "" {
                        match *value {
                            Some(ref v) => v.clone(),
                            None => panic!("element does not exist"),
                        }
                    } else {
                        match children.get(&first_char_unwrap(&k2)) {
                            Some(t) => t.lookup(k2),
                            None => panic!("element does not exist"),
                        }
                    }
                } else {
                    panic!("element does not exist")
                }
            }
        }
    }
}

impl<T: Display> Display for PatriciaTrie<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        fn aux<T: Display>(t: &PatriciaTrie<T>, mut indent: String, last: bool, f: &mut Formatter) -> Result<(), Error> {
            match *t {
                Tip => writeln!(f, "()"),
                Node { ref key, ref value, ref children } => {
                    try!(write!(f, "{}", indent));

                    if last {
                        try!(write!(f, "\\-"));
                        indent = indent + "  ";
                    } else {
                        try!(write!(f, "|-"));
                        indent = indent + "| ";
                    }

                    try!(write!(f, "{}", key));

                    match *value {
                        Some(ref v) => try!(write!(f, " => ({})", v)),
                        _ => ()
                    }

                    try!(writeln!(f, ""));

                    for (i, (_, c)) in children.iter().enumerate() {
                        try!(aux(c, indent.clone(), i == children.len() - 1, f));
                    }

                    Result::Ok(())
                }
            }
        }

        aux(self, "".to_string(), true, f)
    }
}

#[test]
fn patricia_trie() {
    let t: PatriciaTrie<usize> = Map::empty();
    let t2 = t.bind("test".to_string(), 0)
        .bind("slow".to_string(), 1)
        .bind("water".to_string(), 2)
        .bind("slower".to_string(), 3)
        .bind("tester".to_string(), 4)
        .bind("te".to_string(), 5)
        .bind("toast".to_string(), 6)
        .bind("toad".to_string(), 7);

    println!("{}", t2);
    assert_eq!(t2.lookup("test".to_string()), 0);
    assert_eq!(t2.lookup("slow".to_string()), 1);
    assert_eq!(t2.lookup("water".to_string()), 2);
    assert_eq!(t2.lookup("slower".to_string()), 3);
    assert_eq!(t2.lookup("tester".to_string()), 4);
    assert_eq!(t2.lookup("te".to_string()), 5);
    assert_eq!(t2.lookup("toast".to_string()), 6);
    assert_eq!(t2.lookup("toad".to_string()), 7);
}
