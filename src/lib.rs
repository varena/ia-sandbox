#![recursion_limit = "1024"]
#![feature(fnbox)]
#![feature(conservative_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![deny(missing_copy_implementations, missing_debug_implementations, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results, variant_size_differences, warnings)]

extern crate bincode;
#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate simple_signal;

#[macro_use]
mod meta;
pub mod errors;
pub mod run_info;
pub mod config;
mod ffi;

use config::Config;
pub use errors::*;
use run_info::RunInfo;


pub fn run_jail(config: Config) -> Result<RunInfo> {
    let user_group_id = ffi::get_user_group_id();
    let handle = ffi::clone(
        || {
            if let Some(stdin) = config.redirect_stdin() {
                ffi::redirect_fd(ffi::STDIN, stdin)?;
            }

            if let Some(stdout) = config.redirect_stdout() {
                ffi::redirect_fd(ffi::STDOUT, stdout)?;
            }

            if let Some(stderr) = config.redirect_stderr() {
                ffi::redirect_fd(ffi::STDERR, stderr)?;
            }

            if let Some(new_root) = config.new_root() {
                ffi::pivot_root(new_root, || {
                    // Mount proc (since we are in a new pid namespace)
                    // Must be done after pivot_root so we mount this in the right location
                    // but also before we unmount the old root because ... I don't know
                    ffi::mount_proc()
                })?;
            } else {
                ffi::mount_proc()?;
            }

            // Make sure we are root (we don't really need to,
            // but this way the child process can do anything it likes
            // inside its namespace and nothing outside)
            // Must be done after mount_proc so we can properly read and write
            // /proc/self/uid_map and /proc/self/gid_map
            ffi::set_uid_gid_maps(user_group_id)?;

            // Move the process to a different process group (so it can't kill it's own
            // father by sending signals to the whole process group)
            ffi::move_to_different_process_group()?;

            ffi::exec_command(config.command(), config.args())
        },
        config.share_net(),
    )?;
    use std::result::Result as StdResult;
    handle
        .wait()
        .and_then(|run_info: StdResult<RunInfo, ChildResult<()>>| {
            run_info
                .map_err(|err| {
                    err.map(|()| {
                        ChildError::Custom(
                            "Child process successfully completed even though it used exec".into(),
                        )
                    }).unwrap_or_else(|err| err)
                })
                .map_err(|err| err.into())
        })
}
