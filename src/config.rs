use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShareNet {
    Share,
    Unshare,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpaceUsage(u64);

impl SpaceUsage {
    pub fn from_bytes(bytes: u64) -> SpaceUsage {
        SpaceUsage(bytes)
    }

    pub fn from_kilobytes(kilobytes: u64) -> SpaceUsage {
        SpaceUsage::from_bytes(kilobytes * 1_000)
    }

    pub fn from_megabytes(megabytes: u64) -> SpaceUsage {
        SpaceUsage::from_kilobytes(megabytes * 1_000)
    }

    pub fn from_gigabytes(gigabytes: u64) -> SpaceUsage {
        SpaceUsage::from_megabytes(gigabytes * 1_000)
    }

    pub fn from_kibibytes(kibibytes: u64) -> SpaceUsage {
        SpaceUsage::from_bytes(kibibytes * 1_024)
    }

    pub fn from_mebibytes(mebibytes: u64) -> SpaceUsage {
        SpaceUsage::from_kibibytes(mebibytes * 1_024)
    }

    pub fn from_gibibytes(gibibytes: u64) -> SpaceUsage {
        SpaceUsage::from_mebibytes(gibibytes * 1_024)
    }

    pub fn as_bytes(&self) -> u64 {
        self.0
    }

    pub fn as_kilobytes(&self) -> u64 {
        self.0 / 1_000
    }
}

impl Display for SpaceUsage {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if self.0 % (1 << 30) == 0 {
            write!(fmt, "{} gibibytes", self.0 >> 30)
        } else if self.0 % (1 << 20) == 0 {
            write!(fmt, "{} mebibytes", self.0 >> 20)
        } else if self.0 % (1 << 10) == 0 {
            write!(fmt, "{} kibibytes", self.0 >> 10)
        } else if self.0 % 1_000_000_000 == 0 {
            write!(fmt, "{} gigabytes", self.0 / 1_000_000_000)
        } else if self.0 % 1_000_000 == 0 {
            write!(fmt, "{} megabytes", self.0 / 1_000_000)
        } else if self.0 % 1_000 == 0 {
            write!(fmt, "{} kilobytes", self.0 / 1_000)
        } else {
            write!(fmt, "{} bytes", self.0)
        }
    }
}

/// Limits for memory/time
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Limits {
    wall_time: Option<Duration>,
    user_time: Option<Duration>,
    memory: Option<SpaceUsage>,
    stack: Option<SpaceUsage>,
    pids: Option<usize>,
}

impl Limits {
    pub fn new(
        wall_time: Option<Duration>,
        user_time: Option<Duration>,
        memory: Option<SpaceUsage>,
        stack: Option<SpaceUsage>,
        pids: Option<usize>,
    ) -> Limits {
        Limits {
            wall_time,
            user_time,
            memory,
            stack,
            pids,
        }
    }

    pub fn wall_time(&self) -> Option<Duration> {
        self.wall_time
    }

    pub fn user_time(&self) -> Option<Duration> {
        self.user_time
    }

    pub fn memory(&self) -> Option<SpaceUsage> {
        self.memory
    }

    pub fn stack(&self) -> Option<SpaceUsage> {
        self.stack
    }

    pub fn pids(&self) -> Option<usize> {
        self.pids
    }
}

impl Default for Limits {
    fn default() -> Limits {
        Limits::new(None, None, None, None, None)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ControllerPath {
    cpuacct: Option<PathBuf>,
    memory: Option<PathBuf>,
    pids: Option<PathBuf>,
}

impl ControllerPath {
    pub fn new(
        cpuacct: Option<PathBuf>,
        memory: Option<PathBuf>,
        pids: Option<PathBuf>,
    ) -> ControllerPath {
        ControllerPath {
            cpuacct,
            memory,
            pids,
        }
    }

    pub fn cpuacct(&self) -> Option<&Path> {
        self.cpuacct.as_ref().map(|path_buf| path_buf.as_path())
    }

    pub fn memory(&self) -> Option<&Path> {
        self.memory.as_ref().map(|path_buf| path_buf.as_path())
    }

    pub fn pids(&self) -> Option<&Path> {
        self.pids.as_ref().map(|path_buf| path_buf.as_path())
    }
}

impl Default for ControllerPath {
    fn default() -> ControllerPath {
        ControllerPath::new(None, None, None)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct MountOptions {
    read_only: bool,
    dev: bool,
    exec: bool,
}

impl MountOptions {
    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn dev(&self) -> bool {
        self.dev
    }

    pub fn exec(&self) -> bool {
        self.exec
    }

    pub fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
    }

    pub fn set_dev(&mut self, value: bool) {
        self.dev = value;
    }

    pub fn set_exec(&mut self, value: bool) {
        self.exec = value;
    }
}

impl Default for MountOptions {
    fn default() -> MountOptions {
        MountOptions {
            read_only: true,
            dev: false,
            exec: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Mount {
    source: PathBuf,
    destination: PathBuf,
    mount_options: MountOptions,
}

impl Mount {
    pub fn new(source: PathBuf, destination: PathBuf, mount_options: MountOptions) -> Mount {
        Mount {
            source,
            destination,
            mount_options,
        }
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn destination(&self) -> &Path {
        &self.destination
    }

    pub fn mount_options(&self) -> MountOptions {
        self.mount_options
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    command: PathBuf,
    args: Vec<OsString>,
    new_root: Option<PathBuf>,
    share_net: ShareNet,
    redirect_stdin: Option<PathBuf>,
    redirect_stdout: Option<PathBuf>,
    redirect_stderr: Option<PathBuf>,
    limits: Limits,
    instance_name: Option<OsString>,
    controller_path: ControllerPath,
    mounts: Vec<Mount>,
}

impl Config {
    pub fn new(
        command: PathBuf,
        args: Vec<OsString>,
        new_root: Option<PathBuf>,
        share_net: ShareNet,
        redirect_stdin: Option<PathBuf>,
        redirect_stdout: Option<PathBuf>,
        redirect_stderr: Option<PathBuf>,
        limits: Limits,
        instance_name: Option<OsString>,
        controller_path: ControllerPath,
        mounts: Vec<Mount>,
    ) -> Config {
        Config {
            command,
            args,
            new_root,
            share_net,
            redirect_stdin,
            redirect_stdout,
            redirect_stderr,
            limits,
            instance_name,
            controller_path,
            mounts,
        }
    }

    pub fn command(&self) -> &Path {
        &self.command
    }

    pub fn args<'a>(&'a self) -> Vec<&'a OsStr> {
        self.args
            .iter()
            .map(|os_string| os_string.as_os_str())
            .collect()
    }

    pub fn new_root(&self) -> Option<&Path> {
        self.new_root.as_ref().map(|path_buf| path_buf.as_path())
    }

    pub fn share_net(&self) -> ShareNet {
        self.share_net
    }

    pub fn redirect_stdin(&self) -> Option<&Path> {
        self.redirect_stdin
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }

    pub fn redirect_stdout(&self) -> Option<&Path> {
        self.redirect_stdout
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }

    pub fn redirect_stderr(&self) -> Option<&Path> {
        self.redirect_stderr
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    pub fn instance_name(&self) -> Option<&OsStr> {
        self.instance_name
            .as_ref()
            .map(|os_string| os_string.as_os_str())
    }

    pub fn controller_path(&self) -> &ControllerPath {
        &self.controller_path
    }

    pub fn mounts(&self) -> &[Mount] {
        self.mounts.as_ref()
    }
}
