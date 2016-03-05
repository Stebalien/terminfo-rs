extern crate terminfo;

use terminfo::Terminfo;
use std::fs;

#[test]
fn test_parse() {
    for f in fs::read_dir("tests/data/").unwrap() {
        let _ = Terminfo::from_path(f.unwrap().path()).unwrap();
    }
}
