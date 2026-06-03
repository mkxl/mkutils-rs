use std::collections::{BTreeMap, HashMap};

#[test]
fn builds_hash_map() {
    let map: HashMap<String, i64> = mkutils::map! {
        "foo" => 3,
        "bar" => 5_u8,
    };

    assert_eq!(map.get("foo"), Some(&3));
    assert_eq!(map.get("bar"), Some(&5));
}

#[test]
fn builds_btree_map() {
    let map: BTreeMap<String, String> = mkutils::map! {
        "foo" => "one",
        "bar" => "two",
    };

    assert_eq!(map.get("foo"), Some(&String::from("one")));
    assert_eq!(map.get("bar"), Some(&String::from("two")));
}

#[test]
fn accepts_trailing_comma() {
    let map: HashMap<String, String> = mkutils::map! {
        "foo" => "bar",
    };

    assert_eq!(map.get("foo"), Some(&String::from("bar")));
}

#[test]
fn accepts_path_keus() {
    let map: HashMap<Option<bool>, String> = mkutils::map! {
        Option::<bool>::None => "bar",
    };

    assert_eq!(map.get(&None), Some(&String::from("bar")));
}
