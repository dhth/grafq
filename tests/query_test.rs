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
    let mut cmd = fx.cmd(["query", "--help"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Execute a one-off query

    Usage: gcue query [OPTIONS] <QUERY>

    Arguments:
      <QUERY>  Cypher query to execute

    Options:
      -b, --bench                           Whether to benchmark the query
      -n, --bench-num-runs <NUMBER>         Number of benchmark runs [default: 5]
          --debug                           Output debug information without doing anything
      -W, --bench-num-warmup-runs <NUMBER>  Number of benchmark warmup runs [default: 3]
      -p, --print-query                     Print query
      -h, --help                            Print help

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_for_defaults() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--debug", "-"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    benchmark:                  false
    print query:                false
    query:                      -

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_with_overridden_flags() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "query",
        "--bench",
        "--bench-num-runs",
        "10",
        "--bench-num-warmup-runs",
        "5",
        "--print-query",
        "--debug",
        "MATCH (c: Candidate) RETURN c.id LIMIT 5",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    benchmark:                  true
    benchmark num runs:         10
    benchmark num warmup runs:  5
    print query:                true
    query:
    ---
    MATCH (c: Candidate) RETURN c.id LIMIT 5
    ---

    ----- stderr -----
    ");
}

//-------------//
//  FAILURES   //
//-------------//

#[test]
fn fails_if_provided_with_incorrect_benchmark_num_runs() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--bench", "--bench-num-runs", "0"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value '0' for '--bench-num-runs <NUMBER>': needs to be greater than 0

    For more information, try '--help'.
    ");
}
