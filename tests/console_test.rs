mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn shows_help() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["console", "--help"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Open gcue's console

    Usage: gcue console [OPTIONS]

    Options:
      -p, --page-results             Display results via a pager ("less", by default, can be overridden by $GCUE_PAGER)
      -w, --write-results            Write results to filesystem
      -d, --results-dir <DIRECTORY>  Directory to write results in [default: .gcue]
          --debug                    Output debug information without doing anything
      -f, --results-format <FORMAT>  Format to write results in [default: json] [possible values: csv, json]
      -h, --help                     Print help

    ----- stderr -----
    "#);
}

#[test]
fn debug_flag_works_for_defaults() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["console", "--debug"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    console
    display results via pager:  false
    write results:              false
    results directory:          .gcue
    results format:             json

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_for_overridden_flags() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "console",
        "--write-results",
        "--results-dir",
        "path/to/results/dir",
        "--results-format",
        "json",
        "--debug",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    console
    display results via pager:  false
    write results:              true
    results directory:          path/to/results/dir
    results format:             json

    ----- stderr -----
    ");
}
