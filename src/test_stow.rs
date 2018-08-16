use super::*;
use std::fs::*;

#[cfg(test)]
use test_utils::*;

//#[test]
//fn test_basic_stow() {
//    build_source_target_directories();
//    let source: PathBuf = PathBuf::from(TEST_SOURCE);
//    let target: PathBuf = PathBuf::from(TEST_TARGET);
//    add_file_to("file.txt", &source);
//    let force_flag = false;
//    let backup_flag = false;
//    let unstow_flag = false;
//    let mut operations: LinkedList<io::Result<FSOperation>> = LinkedList::new();
//
//    visit(source.as_path(), target.as_path(), force_flag, backup_flag, unstow_flag, operations.borrow_mut()).expect("An error occurred when visiting directories");
//
//    let mut iter = operations.iter();
//
//    let value = iter.next().unwrap().as_ref().unwrap();
//    assert!(value == &FSOperation::CreateSymlink { source: source.join("file.txt"), target: target.join("file.txt") });
//    clear_directories();
//}