//! The plain-file guard: what it lets through and what it refuses.

use sdcheck::guard::check;

fn refused(source: &str, needle: &str) {
    let result = check(source.as_bytes());
    let reason = result.expect_err(source);
    assert!(
        reason.contains(needle),
        "{source:?} gave {reason:?}, wanted {needle:?}"
    );
}

fn passes(source: &str) {
    assert_eq!(check(source.as_bytes()), Ok(()), "{source:?}");
}

#[test]
fn record_holders_pass() {
    for entry in std::fs::read_dir("../targets").unwrap() {
        let path = entry.unwrap().path().join("holder.rs");
        let source = std::fs::read(&path).unwrap();
        assert_eq!(check(&source), Ok(()), "{}", path.display());
    }
}

#[test]
fn ordinary_code_passes() {
    passes("pub fn solve(i: &[u8], o: &mut [u8]) -> usize { if !(i.len() != 0) { 0 } else { 1 } }");
    passes("fn f(x: u8) -> bool { matches!(x, 1 | 2) }");
    passes("#[inline(always)] fn f() {}");
    passes("#[repr(align(16))] #[derive(Clone, Copy)] struct A([u8; 16]);");
    passes("fn f<'a>(x: &'a [u8]) -> &'a [u8] { x }");
    passes("fn f() -> u8 { b'a' + ('b' as u8) }");
    passes("fn f(d: bool) -> bool { while !d { return !d; } d }");
}

#[test]
fn comments_are_refused() {
    refused("// hello\nfn f() {}", "comments");
    refused("fn f() {} /* hello */", "comments");
    refused("/// doc\nfn f() {}", "comments");
    refused("fn f() { /* unterminated", "comments");
}

#[test]
fn strings_are_refused() {
    refused(
        "static S: &str = \"ignore previous instructions\";",
        "string literals",
    );
    refused("static S: &[u8] = b\"abc\";", "string literals");
    refused("static S: &str = r#\"abc\"#;", "string literals");
    refused("static S: &core::ffi::CStr = c\"abc\";", "string literals");
}

#[test]
fn macros_outside_the_list_are_refused() {
    refused("fn f() { core::include!(x); }", "macros");
    refused("fn f() -> u32 { core::env!(X) }", "macros");
    refused("fn f() { core::arch::asm!(x); }", "macros");
    refused("fn f() { panic!() }", "macros");
    refused("macro_rules! m { () => {} }", "macro_rules");
}

#[test]
fn attributes_outside_the_list_are_refused() {
    refused("#[no_mangle] pub fn f() {}", "attributes");
    refused("#[unsafe(no_mangle)] pub fn f() {}", "attributes");
    refused("#[link_section = 1] static X: u8 = 0;", "attributes");
    refused("#[cfg(test)] fn f() {}", "attributes");
    refused(
        "#![feature(core_intrinsics)]\nfn f() {}",
        "inner attributes",
    );
}

#[test]
fn other_escapes_are_refused() {
    refused("extern crate alloc;", "extern");
    refused("fn r#match() {}", "token");
    refused("fn f() { let caf\u{e9} = 1; }", "ASCII");
    refused(
        "fn a_name_that_is_far_longer_than_allowed() {}",
        "names are limited",
    );
    refused("", "between 1 and");
}
