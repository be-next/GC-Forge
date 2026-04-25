//! Docker-backed [`Runner`] implementation.
//!
//! Reference: SPEC-TECHNIQUE §4.5.1.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use gc_forge_scenario::ByteSize;

use crate::flags::build_jvm_command;
use crate::runner::{ExitStatus, RunOutcome, RunSpec, Runner, RunnerError};

/// Path inside the runner container where the harness JAR is mounted.
const HARNESS_PATH_IN_CONTAINER: &str = "/work/harness.jar";
/// Directory inside the runner container where the GC log lands.
const WORK_DIR_IN_CONTAINER: &str = "/work";
/// Default base image; selected per `scenario.spec.jvm.major`.
fn default_image_for_major(major: u8) -> String {
    format!("eclipse-temurin:{major}-jdk-jammy")
}

/// Docker-backed Runner.
#[derive(Debug, Clone)]
pub struct DockerRunner {
    /// Path to the `docker` executable. Mostly here to make tests injectable.
    docker_bin: PathBuf,
    /// Optional explicit image override. When `None`, derived from the
    /// scenario's `jvm.major`.
    image_override: Option<String>,
    /// When `Some(path)`, the runner assumes the harness jar is **already
    /// inside the image** at the given path and skips the read-only mount
    /// that would otherwise place it at `/work/harness.jar`. Used by the
    /// `gc-forge-runner:*-jdk*` variants distributed by the project.
    embedded_harness_path: Option<String>,
    /// Optional `--cpus` value passed to `docker run`.
    cpus: Option<f64>,
    /// Optional `--memory` value passed to `docker run`.
    memory: Option<ByteSize>,
}

impl Default for DockerRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerRunner {
    /// Builds a Docker runner with sensible defaults (`docker` on `PATH`,
    /// image derived from the scenario, harness mounted from host, no
    /// host limits).
    #[must_use]
    pub fn new() -> Self {
        Self {
            docker_bin: PathBuf::from("docker"),
            image_override: None,
            embedded_harness_path: None,
            cpus: None,
            memory: None,
        }
    }

    /// Overrides the image tag used by `docker run`. Useful for pinning a
    /// digest (`...@sha256:…`) for reproducibility.
    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image_override = Some(image.into());
        self
    }

    /// Tells the runner the harness jar is already inside the image (e.g.
    /// the `gc-forge-runner:*-jdk*` variants ship with `harness.jar` baked
    /// in at `/opt/gc-forge/harness.jar`). When set, the runner does not
    /// mount the host jar.
    #[must_use]
    pub fn with_embedded_harness(mut self, path_in_image: impl Into<String>) -> Self {
        self.embedded_harness_path = Some(path_in_image.into());
        self
    }

    /// Sets the `--cpus` limit.
    #[must_use]
    pub const fn with_cpus(mut self, cpus: f64) -> Self {
        self.cpus = Some(cpus);
        self
    }

    /// Sets the `--memory` limit.
    #[must_use]
    pub const fn with_memory(mut self, memory: ByteSize) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Returns the image that will be used for a given run.
    fn image_for(&self, spec: &RunSpec) -> String {
        self.image_override
            .clone()
            .unwrap_or_else(|| default_image_for_major(spec.scenario.spec.jvm.major))
    }

    /// Builds the full `docker run …` argv (without the leading `docker`).
    /// Public for testing (asserting the shape) and for printing in
    /// diagnostics.
    #[must_use]
    pub fn build_docker_argv(&self, spec: &RunSpec) -> Vec<String> {
        let mut argv = vec![
            "run".to_owned(),
            "--rm".to_owned(),
            "--network=none".to_owned(),
        ];

        // Per-run CPU/memory limits override the runner's defaults.
        let cpus = spec.cpu_limit.or(self.cpus);
        if let Some(c) = cpus {
            argv.push(format!("--cpus={c}"));
        }
        let memory = spec.memory_limit.or(self.memory);
        if let Some(m) = memory {
            argv.push(format!("--memory={}", m.as_u64()));
        }

        // Mounts: the host's log-file directory becomes /work (rw). The
        // harness jar is mounted read-only at /work/harness.jar unless
        // the image already embeds it (see `with_embedded_harness`).
        let host_work = host_work_dir(&spec.log_path);
        argv.push("-v".to_owned());
        argv.push(format!("{}:{}", host_work.display(), WORK_DIR_IN_CONTAINER));
        if self.embedded_harness_path.is_none() {
            argv.push("-v".to_owned());
            argv.push(format!(
                "{}:{}:ro",
                spec.harness_jar.display(),
                HARNESS_PATH_IN_CONTAINER
            ));
        }

        // Force the entrypoint to `java` so the assembled argv is the JVM
        // command line, regardless of whether the image has a default
        // ENTRYPOINT (e.g. `gc-forge-runner:*-jdk*` does, the public
        // `eclipse-temurin:*` does not).
        argv.push("--entrypoint=java".to_owned());

        // Image.
        argv.push(self.image_for(spec));

        // Command: <jvm flags> -jar <harness-path> <workload args>
        let log_in_container = log_path_in_container(&spec.log_path);
        argv.extend(build_jvm_command(&spec.scenario, &log_in_container));
        argv.push("-jar".to_owned());
        argv.push(
            self.embedded_harness_path
                .clone()
                .unwrap_or_else(|| HARNESS_PATH_IN_CONTAINER.to_owned()),
        );
        argv.extend(spec.workload_args.iter().cloned());

        argv
    }

    fn probe_jvm_version(&self, image: &str) -> Option<String> {
        let output = Command::new(&self.docker_bin)
            .args(["run", "--rm", "--network=none", image, "java", "-version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .ok()?;
        // `java -version` prints to stderr historically; merge both.
        let mut text = String::from_utf8_lossy(&output.stderr).into_owned();
        if text.is_empty() {
            text = String::from_utf8_lossy(&output.stdout).into_owned();
        }
        // First non-empty line is enough for traceability.
        text.lines()
            .find(|l| !l.trim().is_empty())
            .map(str::to_owned)
    }
}

impl Runner for DockerRunner {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn check_available(&self) -> Result<(), RunnerError> {
        let status = Command::new(&self.docker_bin)
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| RunnerError::NotAvailable {
                name: "docker",
                reason: format!(
                    "could not invoke `{} version`: {e}",
                    self.docker_bin.display()
                ),
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(RunnerError::NotAvailable {
                name: "docker",
                reason: format!("`docker version` exited with status {status}"),
            })
        }
    }

    fn execute(&self, spec: &RunSpec) -> Result<RunOutcome, RunnerError> {
        self.check_available()?;

        // Make sure the host work directory exists before mounting it.
        if let Some(parent) = spec.log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RunnerError::LogCapture {
                path: spec.log_path.clone(),
                source: e,
            })?;
        }

        let image = self.image_for(spec);
        let jvm_version = self.probe_jvm_version(&image);

        let argv = self.build_docker_argv(spec);
        let started_at = SystemTime::now();

        let status = Command::new(&self.docker_bin)
            .args(&argv)
            .status()
            .map_err(RunnerError::Spawn)?;

        let ended_at = SystemTime::now();
        let exit_status = if status.success() {
            ExitStatus::Success
        } else if let Some(code) = status.code() {
            // 137 == 128+9 (SIGKILL, common OOM-killer signal under Docker)
            if code == 137 {
                ExitStatus::Oom
            } else {
                ExitStatus::Failure(code)
            }
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = status.signal() {
                    return Ok(RunOutcome {
                        log_path: spec.log_path.clone(),
                        exit_status: ExitStatus::Signal(sig),
                        started_at,
                        ended_at,
                        jvm_version,
                        jvm_locator: image,
                    });
                }
            }
            ExitStatus::Failure(-1)
        };

        Ok(RunOutcome {
            log_path: spec.log_path.clone(),
            exit_status,
            started_at,
            ended_at,
            jvm_version,
            jvm_locator: image,
        })
    }
}

fn host_work_dir(log_path: &Path) -> PathBuf {
    log_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn log_path_in_container(log_path: &Path) -> PathBuf {
    let file_name = log_path.file_name().map_or_else(
        || std::ffi::OsString::from("gc.log"),
        std::ffi::OsStr::to_os_string,
    );
    let mut p = PathBuf::from(WORK_DIR_IN_CONTAINER);
    p.push(file_name);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc_forge_scenario::Scenario;
    use std::path::PathBuf;

    fn scenario(yaml: &str) -> Scenario {
        Scenario::from_yaml(yaml, "<test>").unwrap()
    }

    fn good() -> Scenario {
        scenario(
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: smoke
spec:
  jvm: { vendor: temurin, major: 21 }
  gc:
    algorithm: G1
    options:
      heap: { min: 1g, max: 2g }
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
",
        )
    }

    fn dummy_spec() -> RunSpec {
        RunSpec {
            scenario: good(),
            log_path: PathBuf::from("/tmp/gcforge-runner-test/gc.log"),
            harness_jar: PathBuf::from("/opt/gc-forge/harness.jar"),
            workload_args: vec!["PT5S".to_owned(), "10".to_owned()],
            cpu_limit: None,
            memory_limit: None,
        }
    }

    #[test]
    fn argv_starts_with_run_rm_no_network() {
        let argv = DockerRunner::new().build_docker_argv(&dummy_spec());
        assert_eq!(argv[0], "run");
        assert_eq!(argv[1], "--rm");
        assert_eq!(argv[2], "--network=none");
    }

    #[test]
    fn argv_includes_workdir_and_harness_mounts() {
        let argv = DockerRunner::new().build_docker_argv(&dummy_spec());
        let host_work = "/tmp/gcforge-runner-test:/work".to_owned();
        let harness = "/opt/gc-forge/harness.jar:/work/harness.jar:ro".to_owned();
        assert!(argv.windows(2).any(|w| w[0] == "-v" && w[1] == host_work));
        assert!(argv.windows(2).any(|w| w[0] == "-v" && w[1] == harness));
    }

    #[test]
    fn argv_image_defaults_from_jvm_major() {
        let argv = DockerRunner::new().build_docker_argv(&dummy_spec());
        assert!(argv.iter().any(|a| a == "eclipse-temurin:21-jdk-jammy"));

        let mut spec_17 = dummy_spec();
        spec_17.scenario.spec.jvm.major = 17;
        let argv = DockerRunner::new().build_docker_argv(&spec_17);
        assert!(argv.iter().any(|a| a == "eclipse-temurin:17-jdk-jammy"));
    }

    #[test]
    fn argv_image_can_be_overridden() {
        let argv = DockerRunner::new()
            .with_image("ghcr.io/example/gc-forge-runner:dev-jdk21")
            .build_docker_argv(&dummy_spec());
        assert!(argv
            .iter()
            .any(|a| a == "ghcr.io/example/gc-forge-runner:dev-jdk21"));
        assert!(!argv.iter().any(|a| a == "eclipse-temurin:21-jdk-jammy"));
    }

    #[test]
    fn argv_passes_workload_args_after_jar() {
        let argv = DockerRunner::new().build_docker_argv(&dummy_spec());
        let jar_idx = argv.iter().position(|a| a == "-jar").unwrap();
        // After "-jar" comes "/work/harness.jar", then the workload args.
        assert_eq!(argv[jar_idx + 1], HARNESS_PATH_IN_CONTAINER);
        assert_eq!(argv[jar_idx + 2], "PT5S");
        assert_eq!(argv[jar_idx + 3], "10");
    }

    #[test]
    fn argv_forces_java_entrypoint() {
        let argv = DockerRunner::new().build_docker_argv(&dummy_spec());
        assert!(argv.iter().any(|a| a == "--entrypoint=java"));
    }

    #[test]
    fn argv_includes_xlog_with_container_path() {
        let argv = DockerRunner::new().build_docker_argv(&dummy_spec());
        let xlog = argv.iter().find(|a| a.starts_with("-Xlog:")).unwrap();
        assert!(xlog.contains("file=/work/gc.log"), "{xlog}");
    }

    #[test]
    fn argv_honours_cpu_and_memory_limits() {
        let runner = DockerRunner::new()
            .with_cpus(2.5)
            .with_memory(ByteSize(1024 * 1024 * 512));
        let argv = runner.build_docker_argv(&dummy_spec());
        assert!(argv.iter().any(|a| a == "--cpus=2.5"));
        assert!(argv
            .iter()
            .any(|a| a == &format!("--memory={}", 1024 * 1024 * 512)));
    }

    #[test]
    fn per_run_limits_override_runner_defaults() {
        let runner = DockerRunner::new().with_cpus(2.0);
        let mut spec = dummy_spec();
        spec.cpu_limit = Some(4.0);
        let argv = runner.build_docker_argv(&spec);
        assert!(argv.iter().any(|a| a == "--cpus=4"));
        assert!(!argv.iter().any(|a| a == "--cpus=2"));
    }

    #[test]
    fn check_available_reports_not_available_when_docker_missing() {
        // We point at a deliberately bogus path so the spawn fails.
        let runner = DockerRunner {
            docker_bin: PathBuf::from("/nonexistent/binary/docker"),
            image_override: None,
            embedded_harness_path: None,
            cpus: None,
            memory: None,
        };
        let err = runner.check_available().unwrap_err();
        assert!(matches!(err, RunnerError::NotAvailable { .. }));
    }

    #[test]
    fn embedded_harness_skips_jar_mount() {
        let runner = DockerRunner::new().with_embedded_harness("/opt/gc-forge/harness.jar");
        let argv = runner.build_docker_argv(&dummy_spec());
        // Only the work-dir mount remains.
        let mount_count = argv.iter().filter(|a| a.as_str() == "-v").count();
        assert_eq!(mount_count, 1, "argv = {argv:?}");
        // The -jar argument points at the embedded path.
        let jar_idx = argv.iter().position(|a| a == "-jar").unwrap();
        assert_eq!(argv[jar_idx + 1], "/opt/gc-forge/harness.jar");
    }

    #[test]
    fn host_work_dir_falls_back_to_dot() {
        let p = host_work_dir(Path::new("gc.log"));
        assert_eq!(p, PathBuf::from(""));
    }

    #[test]
    fn log_path_in_container_strips_host_directory() {
        let p = log_path_in_container(Path::new("/some/host/dir/run-123.log"));
        assert_eq!(p, PathBuf::from("/work/run-123.log"));
    }
}
