extern crate ia_sandbox;
extern crate libc;
extern crate tempfile;

use std::fs::File;
use std::io::Write;
use std::time::Duration;

use ia_sandbox::config::{ClearUsage, Environment, Mount, MountOptions, SpaceUsage, SwapRedirects};
use ia_sandbox::errors::{ChildError, Error, FFIError};

use tempfile::Builder;

mod utils;
#[cfg(feature = "nightly")]
use utils::matchers::KilledBySignal;
use utils::matchers::{
    AnnotateAssert, CompareLimits, IsSuccess, MemoryLimitExceeded, NonZeroExitStatus,
    TimeLimitExceeded, WallTimeLimitExceeded,
};
use utils::{LimitsBuilder, PivotRoot, RunInfoExt, TestRunnerHelper};

const HELLO_WORLD: &str = "./target/debug/hello_world";

const EXIT_WITH_INPUT: &str = "./target/debug/exit_with_input";

const EXIT_WITH_LAST_ARGUMENT: &str = "./target/debug/exit_with_last_argument";

#[cfg(feature = "nightly")]
const KILL_WITH_SIGNAL_ARG: &str = "./target/debug/kill_with_signal_arg";

const SLEEP_1_SECOND: &str = "./target/debug/sleep_1_second";

const LOOP_500_MS: &str = "./target/debug/loop_500_ms";

const THREADS_LOOP_500_MS: &str = "./target/debug/threads_loop_500_ms";

const ALLOCATE_20_MEGABYTES: &str = "./target/debug/allocate_20_megabytes";

const THREADS_ALLOCATE_20_MEGABYTES: &str = "./target/debug/threads_allocate_20_megabytes";

const THREADS_SLEEP_1_SECOND: &str = "./target/debug/threads_sleep_1_second";

const EXIT_WITH_ARG_FILE: &str = "./target/debug/exit_with_arg_file";

const EXIT_WITH_ENV: &str = "./target/debug/exit_with_env";

const WRITE_THEN_READ: &str = "./target/debug/write_then_read";
const READ_THEN_WRITE: &str = "./target/debug/read_then_write";

#[test]
fn test_basic_sandbox() {
    TestRunnerHelper::for_simple_exec("test_basic_sandbox", HELLO_WORLD, PivotRoot::DoNot)
        .config_builder()
        .build_and_run()
        .unwrap()
        .assert(IsSuccess)
}

#[test]
fn test_exec_failed() {
    match TestRunnerHelper::for_simple_exec("test_exec_failed", HELLO_WORLD, PivotRoot::DoNot)
        .config_builder()
        .command("missing")
        .build_and_run()
        .unwrap_err()
    {
        Error::ChildError(ChildError::FFIError(FFIError::ExecError { .. })) => (),
        err => panic!("Expected exec error, got {}", err),
    }
}

#[test]
fn test_pivot_root() {
    TestRunnerHelper::for_simple_exec("test_pivot_root", HELLO_WORLD, PivotRoot::Pivot)
        .config_builder()
        .build_and_run()
        .unwrap()
        .assert(IsSuccess)
}

#[test]
fn test_unshare_net() {
    TestRunnerHelper::for_simple_exec("test_unshare_net", HELLO_WORLD, PivotRoot::Pivot)
        .config_builder()
        .share_net(false)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess)
}

#[test]
fn test_redirect_stdin() {
    let mut helper =
        TestRunnerHelper::for_simple_exec("test_redirect_stdin", EXIT_WITH_INPUT, PivotRoot::Pivot);

    helper.write_file("input", b"0");
    let input_path = helper.file_path("input");
    helper
        .config_builder()
        .stdin(input_path)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    helper.write_file("input", b"23");
    helper
        .config_builder()
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(23));
}

#[test]
fn test_redirect_stdout() {
    let mut helper =
        TestRunnerHelper::for_simple_exec("test_redirect_stdout", HELLO_WORLD, PivotRoot::Pivot);

    let output_path = helper.file_path("output");
    helper
        .config_builder()
        .stdout(&output_path)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    assert_eq!(helper.read_line(output_path), "Hello World!\n");
}

#[test]
fn test_redirect_stderr() {
    let mut helper =
        TestRunnerHelper::for_simple_exec("test_redirect_stderr", HELLO_WORLD, PivotRoot::Pivot);

    let stderr_path = helper.file_path("stderr");
    helper
        .config_builder()
        .stderr(&stderr_path)
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    assert_eq!(helper.read_line(stderr_path), "Hello stderr!\n");
}

#[test]
fn test_arguments() {
    TestRunnerHelper::for_simple_exec("test_arguments", EXIT_WITH_LAST_ARGUMENT, PivotRoot::Pivot)
        .config_builder()
        .arg("0")
        .build_and_run()
        .unwrap()
        .assert(IsSuccess);

    TestRunnerHelper::for_simple_exec("test_arguments", EXIT_WITH_LAST_ARGUMENT, PivotRoot::Pivot)
        .config_builder()
        .args(vec!["24", "0", "17"])
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(17))
}

#[cfg(feature = "nightly")]
#[test]
fn test_killed_by_signal() {
    TestRunnerHelper::for_simple_exec(
        "test_killed_by_signal",
        KILL_WITH_SIGNAL_ARG,
        PivotRoot::Pivot,
    )
    .config_builder()
    .arg("8")
    .build_and_run()
    .unwrap()
    .assert(KilledBySignal(8));

    TestRunnerHelper::for_simple_exec(
        "test_killed_by_signal",
        KILL_WITH_SIGNAL_ARG,
        PivotRoot::Pivot,
    )
    .config_builder()
    .arg("11")
    .build_and_run()
    .unwrap()
    .assert(KilledBySignal(11));
}

#[test]
fn test_wall_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.wall_time(Duration::from_millis(1200));

    TestRunnerHelper::for_simple_exec(
        "test_wall_time_limit_exceeded",
        SLEEP_1_SECOND,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(IsSuccess, limits));

    limits.wall_time(Duration::from_millis(800));
    TestRunnerHelper::for_simple_exec(
        "test_wall_time_limit_exceeded",
        SLEEP_1_SECOND,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(WallTimeLimitExceeded, limits));
}

#[test]
fn test_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.user_time(Duration::from_millis(600));

    TestRunnerHelper::for_simple_exec("test_time_limit_exceeded", LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    limits.user_time(Duration::from_millis(450));
    TestRunnerHelper::for_simple_exec("test_time_limit_exceeded", LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(TimeLimitExceeded, limits));
}

#[test]
fn test_threads_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.user_time(Duration::from_millis(600));

    TestRunnerHelper::for_simple_exec(
        "test_threads_time_limit_exceeded",
        THREADS_LOOP_500_MS,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(IsSuccess, limits));

    limits.user_time(Duration::from_millis(450));
    TestRunnerHelper::for_simple_exec(
        "test_threads_time_limit_exceeded",
        THREADS_LOOP_500_MS,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(TimeLimitExceeded, limits));
}

#[test]
fn test_threads_wall_time_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.wall_time(Duration::from_millis(1200));

    TestRunnerHelper::for_simple_exec(
        "test_threads_wall_time_limit_exceeded",
        THREADS_SLEEP_1_SECOND,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(IsSuccess, limits));

    limits.wall_time(Duration::from_millis(800));
    TestRunnerHelper::for_simple_exec(
        "test_threads_wall_time_limit_exceeded",
        THREADS_SLEEP_1_SECOND,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(WallTimeLimitExceeded, limits));
}

#[test]
fn test_memory_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.memory(SpaceUsage::from_megabytes(26));

    TestRunnerHelper::for_simple_exec(
        "test_memory_limit_exceeded",
        ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(IsSuccess, limits));

    limits.memory(SpaceUsage::from_megabytes(19));
    TestRunnerHelper::for_simple_exec(
        "test_memory_limit_exceeded",
        ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(MemoryLimitExceeded, limits));
}

#[test]
fn test_threads_memory_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.memory(SpaceUsage::from_megabytes(40));

    TestRunnerHelper::for_simple_exec(
        "test_threads_memory_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(IsSuccess, limits));

    limits.memory(SpaceUsage::from_megabytes(19));
    TestRunnerHelper::for_simple_exec(
        "test_threads_memory_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(MemoryLimitExceeded, limits));
}

#[test]
fn test_pids_limit_exceeded() {
    let mut limits = LimitsBuilder::new();
    limits.pids(5);

    TestRunnerHelper::for_simple_exec(
        "test_pids_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(IsSuccess, limits));

    limits.pids(4);
    TestRunnerHelper::for_simple_exec(
        "test_pids_limit_exceeded",
        THREADS_ALLOCATE_20_MEGABYTES,
        PivotRoot::Pivot,
    )
    .config_builder()
    .limits(limits)
    .build_and_run()
    .unwrap()
    .assert(CompareLimits::new(NonZeroExitStatus::any(), limits));
}

#[test]
fn test_mount_directory() {
    let temp_dir = Builder::new()
        .prefix("test_mount_directory_special")
        .tempdir()
        .unwrap();
    let input_path = temp_dir.path().join("input");
    let mut file = File::create(&input_path).unwrap();
    let _ = file.write(b"15\n").unwrap();

    TestRunnerHelper::for_simple_exec("test_mount_directory", EXIT_WITH_ARG_FILE, PivotRoot::Pivot)
        .config_builder()
        .mount(Mount::new(
            temp_dir.path().into(),
            "/mount".into(),
            MountOptions::default(),
        ))
        .arg("/mount/input")
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(15));
}

#[test]
fn test_clear_usage() {
    let mut limits = LimitsBuilder::new();
    limits.user_time(Duration::from_millis(600));

    TestRunnerHelper::for_simple_exec("test_clear_usage", THREADS_LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .clear_usage(ClearUsage::Yes)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));

    TestRunnerHelper::for_simple_exec("test_clear_usage", THREADS_LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .clear_usage(ClearUsage::No)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(TimeLimitExceeded, limits));

    TestRunnerHelper::for_simple_exec("test_clear_usage", THREADS_LOOP_500_MS, PivotRoot::Pivot)
        .config_builder()
        .limits(limits)
        .clear_usage(ClearUsage::Yes)
        .build_and_run()
        .unwrap()
        .assert(CompareLimits::new(IsSuccess, limits));
}

#[test]
fn test_environment() {
    TestRunnerHelper::for_simple_exec("exit_with_env", EXIT_WITH_ENV, PivotRoot::Pivot)
        .config_builder()
        .environment(Environment::EnvList(vec![(
            "arg".to_owned(),
            "12".to_owned(),
        )]))
        .build_and_run()
        .unwrap()
        .assert(NonZeroExitStatus::new(12));
}

#[test]
fn test_interactive() {
    let temp_dir = Builder::new().prefix("test_interactive").tempdir().unwrap();

    let a_path = temp_dir.path().join("a_file");
    let b_path = temp_dir.path().join("b_file");

    utils::make_fifo(&a_path);
    utils::make_fifo(&b_path);

    let mut limits = LimitsBuilder::new();
    limits.wall_time(Duration::from_secs(1));

    let mut write_then_read_helper = TestRunnerHelper::for_simple_exec(
        "test_interactive_write_then_read",
        WRITE_THEN_READ,
        PivotRoot::Pivot,
    );
    let write_then_read = write_then_read_helper
        .config_builder()
        .limits(limits)
        .stdout(&a_path)
        .stdin(&b_path)
        .swap_redirects(SwapRedirects::Yes)
        .build_and_spawn()
        .unwrap();

    let mut read_then_write_helper = TestRunnerHelper::for_simple_exec(
        "test_interactive_read_then_write",
        READ_THEN_WRITE,
        PivotRoot::Pivot,
    );
    let read_then_write = read_then_write_helper
        .config_builder()
        .limits(limits)
        .stdin(&a_path)
        .stdout(&b_path)
        .build_and_spawn()
        .unwrap();

    write_then_read
        .wait()
        .unwrap()
        .assert(AnnotateAssert::new(IsSuccess, "write_then_read"));
    read_then_write
        .wait()
        .unwrap()
        .assert(AnnotateAssert::new(IsSuccess, "read_then_write"));
}
