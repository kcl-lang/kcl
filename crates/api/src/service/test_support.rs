//! Cross-platform helpers for tests that need to observe process RSS.
//!
//! Used by the `#[ignore]`d RSS stability tests in `service_impl.rs` to
//! validate that `release_memory()` bounds the RSS growth of a long-lived
//! `KclServiceImpl` across many `exec_program` calls.
//!
//! `rss_bytes()` returns `None` on platforms where we cannot read the
//! resident set cheaply, so the RSS tests auto-skip on those platforms
//! rather than failing.

/// Resident set size of the current process in bytes, or `None` if the
/// platform does not expose it cheaply.
///
/// - **Linux**: parses `/proc/self/statm` (resident pages × page size).
///   Available on every Linux, no syscall, no privileges needed.
/// - **macOS**: uses `task_info(mach_task_self(), TASK_BASIC_INFO, ...)`
///   via `libc`. Returns `resident_size`.
/// - **Other** (Windows, WASM, …): returns `None`. RSS tests will skip.
#[cfg(target_os = "linux")]
pub fn rss_bytes() -> Option<u64> {
    // `/proc/self/statm` is one line, space-separated. Field 2 (1-indexed)
    // is `resident` in pages. Format is documented in `man proc`:
    //   size       (1) total program size
    //              (2) resident set size
    //              (3) shared pages
    //              ...
    let data = std::fs::read("/proc/self/statm").ok()?;
    let mut fields = data.split(|&b| b == b' ' || b == b'\n');
    let _size = fields.next()?;
    let resident = fields.next()?;
    let pages: u64 = std::str::from_utf8(resident).ok()?.parse().ok()?;
    // SAFETY: `sysconf(_SC_PAGESIZE)` is always safe to call. The page
    // size is a process-wide constant; we only call it once and reuse the
    // value for the lifetime of the process via a `OnceLock`.
    let page_size: u64 =
        *PAGE_SIZE.get_or_init(|| unsafe { libc::sysconf(libc::_SC_PAGESIZE).max(1) as u64 });
    Some(pages.saturating_mul(page_size))
}

/// Process page size cache (Linux). Populated on first RSS read.
#[cfg(target_os = "linux")]
static PAGE_SIZE: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

/// Process RSS in bytes on macOS via `task_info`.
#[cfg(target_os = "macos")]
#[allow(deprecated)] // `mach_task_self` is deprecated in favour of `mach2`;
// bringing in `mach2` for one FFI call is overkill.
pub fn rss_bytes() -> Option<u64> {
    // SAFETY: `mach_task_self()` returns the calling task's port (always
    // valid for the current process) and `task_info` with a writable
    // `mach_task_basic_info` buffer is documented as safe for
    // `MACH_TASK_BASIC_INFO`.
    unsafe {
        let mut info: libc::mach_task_basic_info = std::mem::zeroed();
        let mut count = (std::mem::size_of::<libc::mach_task_basic_info>()
            / std::mem::size_of::<libc::natural_t>())
            as libc::mach_msg_type_number_t;
        let kr = libc::task_info(
            libc::mach_task_self(),
            libc::MACH_TASK_BASIC_INFO,
            (&mut info as *mut _) as *mut _,
            &mut count,
        );
        if kr == libc::KERN_SUCCESS {
            // `resident_size` is already in bytes (`mach_vm_size_t` = u64).
            Some(info.resident_size)
        } else {
            None
        }
    }
}

/// RSS unsupported on Windows / WASM / other platforms.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn rss_bytes() -> Option<u64> {
    None
}
