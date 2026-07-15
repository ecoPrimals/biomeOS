// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::parse_vm_names_from_list;

#[test]
fn test_parse_vm_names_from_list() {
    let list = " Id    Name                           State\n----------------------------------------------------\n 1     my-fed-node1                   running\n 2     my-fed-node2                   running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-node1", "my-fed-node2"]);
}

#[test]
fn test_parse_vm_names_from_list_empty() {
    let names = parse_vm_names_from_list("", "nonexistent");
    assert!(names.is_empty());
}

#[test]
fn test_parse_vm_names_from_list_partial_match() {
    let list =
        " 1     fed-node1    running\n 2     fed-node2    running\n 3     other-node   running\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1", "fed-node2"]);
}

#[test]
fn test_parse_vm_names_from_list_single_vm() {
    let list = " 1     my-fed-node1    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-node1"]);
}

#[test]
fn test_parse_vm_names_from_list_no_match() {
    let list = " 1     other-vm-1    running\n 2     other-vm-2    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert!(names.is_empty());
}

#[test]
fn test_parse_vm_names_from_list_single_column() {
    let list = " 1     fed-node1\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1"]);
}

#[test]
fn test_parse_vm_names_from_list_extra_columns() {
    let list = " Id    Name                State       CPU    Memory\n----------------------------------------------------\n 1     fed-node1           running     1      1024\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1"]);
}

#[test]
fn test_parse_vm_names_from_list_malformed_line() {
    let list = " 1     fed-node1    running\n single_word\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1"]);
}

#[test]
fn test_parse_vm_names_single_token_line_not_pushed() {
    let list = "my-fed\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert!(names.is_empty());
}

#[test]
fn test_parse_vm_names_header_line_extracts_second_column() {
    // Only lines containing the federation substring participate; `real-vm-1` line has no `my-fed`.
    let list = "my-fed header line\n 1     real-vm-1    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["header"]);
}

#[test]
fn test_parse_vm_names_tabs_and_multiple_spaces() {
    let list = "1\tmy-fed-node1\trunning\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-node1"]);
}

#[test]
fn test_parse_vm_names_duplicate_lines() {
    let list = " 1     dup-fed-a    running\n 2     dup-fed-b    running\n";
    let names = parse_vm_names_from_list(list, "dup-fed");
    assert_eq!(names, vec!["dup-fed-a", "dup-fed-b"]);
}

#[test]
fn test_parse_vm_names_numeric_id_with_federation_in_name() {
    let list = " 10    my-fed-10    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-10"]);
}

#[test]
fn test_parse_vm_names_line_contains_fed_but_less_than_two_columns() {
    let list = "my-fed\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert!(names.is_empty());
}

#[test]
fn test_parse_vm_names_preserves_order() {
    let list = " 2     fed-b    running\n 1     fed-a    running\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-b", "fed-a"]);
}

#[test]
fn test_parse_vm_names_match_in_first_column_extracts_second_column() {
    // Any line containing the federation substring participates; the VM name is always column 2.
    let list = "my-fed-prefix    actual-vm-name    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["actual-vm-name"]);
}

#[test]
fn test_parse_vm_names_long_federation_substring() {
    let list = " 1     prefix-my-fed-suffix    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["prefix-my-fed-suffix"]);
}

#[test]
fn test_parse_vm_names_windows_style_line_endings() {
    let list = " 1     my-fed-w1    running\r\n 2     my-fed-w2    running\r\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-w1", "my-fed-w2"]);
}

#[test]
fn test_parse_vm_names_uuid_suffix_in_name() {
    let list = " 1     fed-node-550e8400-e29b-41d4-a716-446655440000    running\n";
    let names = parse_vm_names_from_list(list, "fed-node");
    assert_eq!(names, vec!["fed-node-550e8400-e29b-41d4-a716-446655440000"]);
}

#[test]
fn test_parse_vm_names_three_columns_id_name_state() {
    let list = "42    vm-fed-core    shut off\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["vm-fed-core"]);
}
