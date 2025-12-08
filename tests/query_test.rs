mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

const QUERY: &str = "MATCH (c: Candidate) RETURN c.id LIMIT 5";

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
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Execute a one-off query

    Usage: gcue query [OPTIONS] <QUERY>

    Arguments:
      <QUERY>  Cypher query to execute

    Options:
      -p, --page-results                    Display results via a pager ("less", by default, can be overridden by $GCUE_PAGER)
      -b, --bench                           Whether to benchmark the query
          --debug                           Output debug information without doing anything
      -n, --bench-num-runs <NUMBER>         Number of benchmark runs [default: 5]
      -W, --bench-num-warmup-runs <NUMBER>  Number of benchmark warmup runs [default: 3]
      -P, --print-query                     Print query
      -w, --write-results                   Write results to filesystem
      -d, --results-dir <DIRECTORY>         Directory to write results in [default: .gcue]
      -f, --results-format <FORMAT>         Format to write results in [default: json] [possible values: csv, json]
      -h, --help                            Print help

    ----- stderr -----
    "#);
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
    display results via pager:  false
    benchmark:                  false
    print query:                false
    write results:              false

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
        QUERY,
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    display results via pager:  false
    benchmark:                  true
    benchmark num runs:         10
    benchmark num warmup runs:  5
    print query:                true
    write results:              false

    query:
    ---
    MATCH (c: Candidate) RETURN c.id LIMIT 5
    ---

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_for_write_results_flags() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "query",
        "--write-results",
        "--results-dir",
        "path/to/results/dir",
        "--results-format",
        "json",
        "--debug",
        QUERY,
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    display results via pager:  false
    benchmark:                  false
    print query:                false
    write results:              true
    results directory:          path/to/results/dir
    results format:             json

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
    let mut cmd = fx.cmd(["query", "--bench", "--bench-num-runs", "0", QUERY]);

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

#[test]
fn fails_if_both_benchmark_and_write_flags_are_provided() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--bench", "--write-results", QUERY]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: cannot benchmark and write results at the same time
    ");
}

#[test]
fn fails_if_incorrect_results_format_provided() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "query",
        "--write-results",
        "--results-format",
        "unknown",
        QUERY,
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'unknown' for '--results-format <FORMAT>'
      [possible values: csv, json]

    For more information, try '--help'.
    ");
}
