use regex::Regex;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct TrieNode<T: Clone> {
    /// The key associated with this node
    key: String,
    /// The value associated with this node, if any
    value: Option<T>,
    /// All branches from this node
    children: Vec<Box<TrieNode<T>>>,
    /// If Some(regex), this node represents a param.
    /// This node will match anything fitting `regex`. If no
    /// regex was passed with the param, the regex `[.+]` is
    /// used, which matches (almost) any input.
    /// If `param` is Some, `key` is the name of the param.
    param: Option<Regex>,
}

impl<T: Clone> TrieNode<T> {
    /// Creates a new Trie
    pub fn new() -> Self {
        TrieNode {
            key: String::new(),
            value: None,
            children: Vec::new(),
            param: None,
        }
    }

    pub fn get(&self, key: &str) -> Option<(Option<T>, HashMap<String, String>)> {
        let mut params = HashMap::new();

        let val = self.get_recurse(key, &mut params);
        if val.is_some() {
            Some((val, params))
        } else {
            None
        }
    }

    fn get_recurse(&self, key: &str, map: &mut HashMap<String, String>) -> Option<T> {
        if self.param.is_some() {
            if key.contains('/') {
                let keys = splitn(key, 2, '/');
                let regex = self.param.clone().unwrap();
                if regex.is_match(&keys[0]) {
                    map.insert(self.key.clone(), keys[0].clone());
                    return self.get_children(&keys[1], map);
                }
            } else {
                let regex = self.param.clone().unwrap();
                if regex.is_match(key) {
                    map.insert(self.key.clone(), key.to_string());
                    return self.value.clone();
                }
            }
        } else {
            let match_len = get_match_len(&self.key, key);
            if match_len == self.key.len() {
                if match_len == key.len() {
                    return self.value.clone();
                } else {
                    let key = &key[match_len..];
                    return self.get_children(key, map);
                }
            }
        }

        None
    }

    fn get_children(&self, key: &str, map: &mut HashMap<String, String>) -> Option<T> {
        // Match against non-param children first.
        // We favor a static match over a dynamic one.
        let non_param_children = self.children.iter().filter(|c| c.param.is_none());
        for child in non_param_children {
            let val = child.get_recurse(key, map);
            if val.is_some() {
                return val;
            }
        }
        let param_children = self.children.iter().filter(|c| c.param.is_some());
        for child in param_children {
            let val = child.get_recurse(key, map);
            if val.is_some() {
                return val;
            }
        }
        None
    }

    /// Create a new child node with the given key-value pair and add it
    /// as a child to node `self`.
    fn add_new_child(&mut self, key: String, value: Option<T>, param: Option<Regex>) {
        let child = TrieNode {
            key: key,
            value: value,
            children: Vec::new(),
            param: param,
        };
        self.children.push(Box::new(child));
    }

    /// Inserts a key-value pair into the trie.
    pub fn insert<S: Into<String>>(&mut self, key: S, value: T) {
        let key = key.into();
        // Empty tree, simply set key/value for this node to given key/value.
        if self.key.is_empty() {
            // Get list of params
            let params = get_params_indices(&key);

            // No params given, just a static path
            if params.is_empty() {
                self.key = key;
                self.value = Some(value);
            } else {
                let (start, _) = params[0];
                self.key = key[0..start].to_string();
                self.insert_param(key, value, &params);
            }

            // Non-empty tree cases
        } else if self.param.is_none() {
            // This node is not a param

            // Get the length of the match for our nodes
            // NOTE: The length of the match should always be
            // at least 1. We disallow routes that do not start
            // with '/'.
            let match_len = get_match_len(&self.key, &key);

            // If the length of the match is the length of this node's key,
            // we do not need to split the node.
            if match_len == self.key.len() || match_len == 0 {
                let key = key[match_len..].to_string();
                // This failing implies that we were given two of the same key
                assert!(!key.is_empty());

                let params = get_params_indices(&key);
                // If there are no children, we just add a new node. No need to
                // worry about another node with a matching prefix.
                if self.children.is_empty() {
                    if params.is_empty() {
                        self.add_new_child(key, Some(value), None);
                    } else {
                        let (start, _) = params[0];
                        // The key begins with a param
                        if start == 0 {
                            self.insert_param(key, value, &params);
                        } else {
                            // The key begins with a static string
                            self.insert_children(key, value, &params);
                        }
                    }
                } else {
                    self.insert_children(key, value, &params);
                }
            } else {
                // Match length was less than the length of this node's key.
                // Split node into two seperate nodes
                let child_key = self.key[match_len..].to_string();
                self.key = self.key[0..match_len].to_string();
                // TODO(nokaa): Cloning should be fine since this is an Rc
                let child_value = self.value.clone();
                self.add_new_child(child_key, child_value, None);
                self.value = None;

                // Insert new node
                let key = key[match_len..].to_string();
                // This failing implies that we were given two of the same key
                assert!(!key.is_empty());

                self.insert_children(key, value, &[]);
            }
        } else {
            // Current node is a param
            let key = end_of_param(&key);
            let params = get_params_indices(&key);
            self.insert_children(key, value, &params);
        }
    }

    fn insert_children(&mut self, key: String, value: T, params: &[(usize, usize)]) {
        // Check all children of this node for one that has a
        // common prefix of any length. If a common prefix is
        // found, we  insert at that node.
        for mut child in &mut self.children {
            if child.is_match(&key) {
                child.insert(key, value);
                return;
            }
        }

        // No matching node found, add new child
        if params.is_empty() {
            self.add_new_child(key, Some(value), None);
        } else {
            let (start, _) = params[0];
            let mut child = TrieNode {
                key: key[0..start].to_string(),
                value: None,
                children: Vec::new(),
                param: None,
            };
            child.insert(key[start..].to_string(), value);
            self.children.push(Box::new(child));
        }
    }

    fn insert_param(&mut self, key: String, value: T, params: &[(usize, usize)]) {
        let (start, end) = params[0];
        // let param = key[start + 1..end].to_string();
        let (param, regex) = parse_param(&key[start + 1..end - 1]);
        let params = &params[1..];
        if params.is_empty() && key.len() == end {
            self.add_new_child(param, Some(value), Some(regex));
        } else {
            let mut child = TrieNode {
                key: param,
                value: None,
                children: Vec::new(),
                param: Some(regex),
            };
            child.insert(key[end..].to_string(), value);
            self.children.push(Box::new(child));
        }
    }

    /// Determines if key matches this node.
    // This function is used internally for determining whether or not
    // we need to split a node or create a new node upon insertion.
    fn is_match(&self, key: &str) -> bool {
        // If the given key marks a param
        if key.starts_with('{') {
            // If the current node is a param
            if self.param.is_some() {
                return get_match_len(&self.key, &key[1..]) > 0;
            }
            return false;
        }
        get_match_len(&self.key, key) > 0
    }
}

fn end_of_param(path: &str) -> String {
    match path.find('/') {
        Some(i) => path[i..].to_string(),
        None => String::new(),
    }
}

fn parse_param(param: &str) -> (String, Regex) {
    let mut param_name = String::new();
    let mut regex = String::new();
    let mut in_param = true;
    for c in param.chars() {
        match c {
            ':' => in_param = false,
            c => {
                if in_param {
                    param_name.push(c);
                } else {
                    regex.push(c);
                }
            }
        }
    }
    if regex.is_empty() {
        regex = ".+".to_string();
    }
    let regex = Regex::new(&regex).unwrap();
    (param_name, regex)
}

/// Determines the length of the shared prefix of two strings.
/// E.g. `get_match_len("apple", "ape") => 2`.
fn get_match_len(a: &str, b: &str) -> usize {
    let mut match_len = 0;
    for (ac, bc) in a.chars().zip(b.chars()) {
        if ac == bc && bc != '{' {
            match_len += 1;
        } else {
            break;
        }
    }
    match_len
}

fn get_params_indices(path: &str) -> Vec<(usize, usize)> {
    let mut indices = Vec::new();
    let mut start = 0;
    let mut braces = 0usize;
    for (i, c) in path.char_indices() {
        match c {
            '{' => {
                if braces == 0 {
                    start = i;
                }
                braces += 1;
            }
            '}' => {
                braces -= 1;
                if braces == 0 {
                    indices.push((start, i + 1));
                }
            }
            _ => {}
        }
    }
    // If `in_param`, there is a missing `}`.
    assert_eq!(braces, 0);
    indices
}

fn splitn(s: &str, n: usize, pat: char) -> Vec<String> {
    let mut n = n;
    let mut strings = Vec::new();
    let mut buf = String::new();

    for ch in s.chars() {
        if n <= 1 {
            buf.push(ch);
            continue;
        }
        if ch == pat {
            n -= 1;
            strings.push(buf);
            buf = String::new();
            buf.push(ch);
        } else {
            buf.push(ch);
        }
    }
    strings.push(buf);
    strings
}

#[cfg(test)]
mod test {
    use super::{get_match_len, TrieNode};
    use regex::Regex;

    fn wild_regex() -> Regex {
        Regex::new(".+").unwrap()
    }

    #[test]
    fn match_len() {
        let a = "apple";
        let b = "ape";
        assert_eq!(get_match_len(a, b), 2);
        let c = "rat";
        assert_eq!(get_match_len(a, c), 0);
    }

    #[test]
    fn single_insert() {
        let data = "Data";
        let mut trie = TrieNode::new();
        trie.insert("/", data);

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some(data),
            children: Vec::new(),
            param: None,
        };

        assert_eq!(trie, trie2);
        assert_eq!(trie.get("/").unwrap().0, Some(data));
    }

    #[test]
    fn multiple_insert() {
        let mut trie = TrieNode::new();
        trie.insert("/", "Data");
        trie.insert("/2", "Data2");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some("Data"),
            children: vec![Box::new(TrieNode {
                               key: "2".to_string(),
                               value: Some("Data2"),
                               children: Vec::new(),
                               param: None,
                           })],
            param: None,
        };

        assert_eq!(trie, trie2);
        assert_eq!(trie.get("/").unwrap().0, Some("Data"));
        assert_eq!(trie.get("/2").unwrap().0, Some("Data2"));
    }

    #[test]
    fn split_node() {
        let mut trie = TrieNode::new();
        trie.insert("/1", "Data");
        trie.insert("/2", "Data2");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: None,
            children: vec![Box::new(TrieNode {
                               key: "1".to_string(),
                               value: Some("Data"),
                               children: Vec::new(),
                               param: None,
                           }),
                           Box::new(TrieNode {
                               key: "2".to_string(),
                               value: Some("Data2"),
                               children: Vec::new(),
                               param: None,
                           })],
            param: None,
        };

        assert_eq!(trie, trie2);
        assert_eq!(trie.get("/"), None);
        assert_eq!(trie.get("/1").unwrap().0, Some("Data"));
        assert_eq!(trie.get("/2").unwrap().0, Some("Data2"));
    }

    #[test]
    fn single_insert_param() {
        let mut trie = TrieNode::new();
        trie.insert("/{test}", "Data");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: None,
            children: vec![Box::new(TrieNode {
                               key: "test".to_string(),
                               value: Some("Data"),
                               children: Vec::new(),
                               param: Some(wild_regex()),
                           })],
            param: None,
        };

        assert_eq!(trie, trie2);
        assert_eq!(trie.get("/"), None);
        let (val, map) = trie.get("/cock").unwrap();
        assert_eq!(val, Some("Data"));
        let param = map.get(&"test".to_string());
        assert_eq!(param, Some(&"cock".to_string()));
    }

    #[test]
    fn multiple_insert_param() {
        let mut trie = TrieNode::new();
        trie.insert("/", "Data");
        trie.insert("/{test}", "Data2");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some("Data"),
            children: vec![Box::new(TrieNode {
                               key: "test".to_string(),
                               value: Some("Data2"),
                               children: Vec::new(),
                               param: Some(wild_regex()),
                           })],
            param: None,
        };

        assert_eq!(trie, trie2);
        assert_eq!(trie.get("/").unwrap().0, Some("Data"));
        let (val, map) = trie.get("/cock").unwrap();
        assert_eq!(val, Some("Data2"));
        let param = map.get(&"test".to_string());
        assert_eq!(param, Some(&"cock".to_string()));
    }

    #[test]
    fn param_with_child() {
        let mut trie = TrieNode::new();
        trie.insert("/{test}", "Data");
        trie.insert("/{test}/cock", "Data2");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: None,
            children: vec![Box::new(TrieNode {
                               key: "test".to_string(),
                               value: Some("Data"),
                               children: vec![Box::new(TrieNode {
                                                  key: "/cock".to_string(),
                                                  value: Some("Data2"),
                                                  children: Vec::new(),
                                                  param: None,
                                              })],
                               param: Some(wild_regex()),
                           })],
            param: None,
        };

        assert_eq!(trie, trie2);
        let (val, map) = trie.get("/cock").unwrap();
        assert_eq!(val, Some("Data"));
        let param = map.get(&"test".to_string());
        assert_eq!(param, Some(&"cock".to_string()));

        let (val, map) = trie.get("/horse/cock").unwrap();
        assert_eq!(val, Some("Data2"));
        let param = map.get(&"test".to_string());
        assert_eq!(param, Some(&"horse".to_string()));
    }

    #[test]
    fn senatus_01() {
        let mut trie = TrieNode::new();
        trie.insert("/", "/");
        trie.insert("/b/{board}", "/b/:board");
        trie.insert("/b/{board}/{thread}", "/b/:board/:thread");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some("/"),
            param: None,
            children: vec![Box::new(TrieNode {
                               key: "b/".to_string(),
                               value: None,
                               param: None,
                               children: vec![Box::new(TrieNode {
                                                  key: "board".to_string(),
                                                  value: Some("/b/:board"),
                                                  param: Some(wild_regex()),
                                                  children: vec![Box::new(TrieNode {
                                                                     key: "/".to_string(),
                                                                     value: None,
                                                                     param: None,
                                                                     children:
                                                                         vec![Box::new(TrieNode {
                            key: "thread".to_string(),
                            value: Some("/b/:board/:thread"),
                            param: Some(wild_regex()),
                            children: Vec::new(),
                        })],
                                                                 })],
                                              })],
                           })],
        };

        assert_eq!(trie, trie2);
    }

    #[test]
    fn regex_test() {
        let mut trie = TrieNode::new();
        trie.insert("/", "/");
        trie.insert("/b/{board}", "/b/:board");
        trie.insert("/b/{board}/{thread:[0-9]+}", "/b/:board/:thread");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some("/"),
            param: None,
            children: vec![Box::new(TrieNode {
                               key: "b/".to_string(),
                               value: None,
                               param: None,
                               children: vec![Box::new(TrieNode {
                                                  key: "board".to_string(),
                                                  value: Some("/b/:board"),
                                                  param: Some(wild_regex()),
                                                  children: vec![Box::new(TrieNode {
                                                                     key: "/".to_string(),
                                                                     value: None,
                                                                     param: None,
                                                                     children:
                                                                         vec![Box::new(TrieNode {
                            key: "thread".to_string(),
                            value: Some("/b/:board/:thread"),
                            param: Some(Regex::new("[0-9]+").unwrap()),
                            children: Vec::new(),
                        })],
                                                                 })],
                                              })],
                           })],
        };

        assert_eq!(trie, trie2);

        let (val, map) = trie.get("/b/a/5").unwrap();
        assert_eq!(val, Some("/b/:board/:thread"));
        assert_eq!(map.get(&"thread".to_string()), Some(&"5".to_string()));

        assert_eq!(trie.get("/b/a/b"), None);
    }
}
