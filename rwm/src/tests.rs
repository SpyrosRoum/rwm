use std::path::PathBuf;

use crate::utils::expand_tilde;

#[test]
fn test_tilde_expand() {
    let home = dirs::home_dir().expect("No home dir");

    let p = expand_tilde(&PathBuf::from("~"));
    assert_eq!(p, home);

    let p = expand_tilde(&PathBuf::from("~/"));
    assert_eq!(p, home);

    let p = expand_tilde(&PathBuf::from("~/blah"));
    let mut expected = home;
    expected.push("blah");
    assert_eq!(p, expected);

    let p = expand_tilde(&PathBuf::from("/blah"));
    assert_eq!(p, PathBuf::from("/blah"));

    let p = expand_tilde(&PathBuf::from("/~/blah"));
    assert_eq!(p, PathBuf::from("/~/blah"));
}
