//! Shared test helpers. Each `tests/*.rs` file includes this with `mod common;`.

/// A path under the temp dir that is unique to this test process: `<tmp>/swayve-<pid>-<name><ext>`.
/// Parallel runs get different pids, so they never share a socket or flag file.
pub fn unique_path(name: &str, ext: &str) -> &'static str {
    let path = std::env::temp_dir().join(format!("swayve-{}-{}{}", std::process::id(), name, ext));
    // Leaked once per call. Test-only, and it keeps the call sites `&'static str`.
    Box::leak(path.to_string_lossy().into_owned().into_boxed_str())
}
