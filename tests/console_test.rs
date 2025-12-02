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
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Open gcue's console

    Usage: gcue console [OPTIONS]

    Options:
          --debug  Output debug information without doing anything
      -h, --help   Print help

    ----- stderr -----
    ");
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

    ----- stderr -----
    ");
}
