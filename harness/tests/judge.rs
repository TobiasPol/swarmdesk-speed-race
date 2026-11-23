//! End-to-end verdicts: each test builds real WebAssembly and runs it.

use sdcheck::judge::judge;
use sdcheck::targets;
use std::path::Path;

fn verdict(candidate: &str) -> (String, String) {
    let target = targets::find("crc32").unwrap();
    let work = sdcheck::work_dir();
    // A fixed baseline, not the live record holder: records move, tests must not.
    let holder = Path::new("tests/fixtures/crc32_baseline.rs");
    let report = judge(target, Path::new(candidate), holder, 11, &work);
    let _ = std::fs::remove_dir_all(&work);
    (report.verdict.to_string(), report.reason)
}

#[test]
fn a_faster_file_is_accepted() {
    let (verdict, reason) = verdict("tests/fixtures/crc32_fast.rs");
    assert_eq!((verdict.as_str(), reason.as_str()), ("ACCEPTED", ""));
}

#[test]
fn a_file_does_not_beat_itself() {
    let (verdict, reason) = verdict("tests/fixtures/crc32_baseline.rs");
    assert_eq!(verdict, "REJECTED");
    assert!(reason.contains("percent less fuel"), "{reason}");
}

#[test]
fn a_wrong_answer_is_rejected() {
    let (verdict, reason) = verdict("tests/fixtures/wrong.rs");
    assert_eq!(verdict, "REJECTED");
    assert!(reason.contains("wrong answer"), "{reason}");
}

#[test]
fn an_endless_loop_runs_out_of_fuel() {
    let (verdict, reason) = verdict("tests/fixtures/endless.rs");
    assert_eq!(verdict, "REJECTED");
    assert!(reason.contains("ran out of fuel"), "{reason}");
}

#[test]
fn a_length_beyond_the_buffer_is_rejected() {
    let (verdict, reason) = verdict("tests/fixtures/too_long.rs");
    assert_eq!(verdict, "REJECTED");
    assert!(reason.contains("beyond the output buffer"), "{reason}");
}

#[test]
fn a_file_that_does_not_compile_is_rejected() {
    let (verdict, reason) = verdict("tests/fixtures/broken.rs");
    assert_eq!(
        (verdict.as_str(), reason.as_str()),
        ("REJECTED", "does not compile")
    );
}

#[test]
fn a_file_with_a_comment_never_reaches_the_compiler() {
    let (verdict, reason) = verdict("tests/fixtures/comment.rs");
    assert_eq!(verdict, "REJECTED");
    assert!(reason.starts_with("guard: comments"), "{reason}");
}

#[test]
fn every_record_holder_matches_its_reference() {
    for target in targets::all() {
        let source = Path::new("../targets").join(target.name).join("holder.rs");
        let work = sdcheck::work_dir();
        let report = judge(target, &source, &source, 3, &work);
        let _ = std::fs::remove_dir_all(&work);
        assert_eq!(
            report.verdict, "REJECTED",
            "{}: {}",
            target.name, report.reason
        );
        assert!(
            report.reason.contains("percent less fuel"),
            "{}: {}",
            target.name,
            report.reason
        );
    }
}
