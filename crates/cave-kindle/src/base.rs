//! Shell-code generators for the base initramfs init script.
//!
//! These functions produce POSIX `/bin/sh` fragments that form the core
//! of a minimal Linux initramfs.  The output is meant to be concatenated
//! into a single `/init` script at the root of the CPIO archive.

/// Return shell code that sets the log level from a `${loglevel}` variable.
///
/// Defaults to `3` (warning) if the variable is unset or invalid.
pub fn set_loglevel() -> Vec<String> {
    vec![
        "case \"${loglevel}\" in".into(),
        "\t[0-7]) ;;".into(),
        "\t*) loglevel=3 ;;".into(),
        "esac".into(),
    ]
}

/// Return a shell `echo` line that prints an informational message.
///
/// The message is prefixed with `[INFO]` and sent to stdout.
pub fn einfo(msg: &str) -> String {
    let escaped = msg.replace('\'', "'\\''");
    format!("echo '[INFO] {escaped}'")
}

/// Return a shell `echo` line that prints a warning message.
///
/// The message is prefixed with `[WARN]` and sent to stdout.
pub fn ewarn(msg: &str) -> String {
    let escaped = msg.replace('\'', "'\\''");
    format!("echo '[WARN] {escaped}'")
}

/// Return a shell `echo` line that prints an error message to stderr.
///
/// The message is prefixed with `[ERROR]` and sent to stderr (`>&2`).
pub fn eerror(msg: &str) -> String {
    let escaped = msg.replace('\'', "'\\''");
    format!("echo '[ERROR] {escaped}' >&2")
}

/// Return a shell variable assignment: `name=value`.
pub fn setvar(name: &str, value: &str) -> String {
    format!("{name}={value}")
}

/// Return a shell expression that reads a variable by name.
///
/// Uses `eval echo` to perform variable indirection.
pub fn readvar(name: &str) -> String {
    format!("eval echo \"${{{name}}}\"")
}

/// Generate shell code that probes a list of device paths for a root
/// filesystem.
///
/// Sets the shell variable `root` to the first device that exists.
pub fn find_root(possible: &[&str]) -> Vec<String> {
    if possible.is_empty() {
        return vec!["root=\"\"".into()];
    }

    let mut lines = vec!["# Find root device".into()];
    let candidates = possible.join(" ");
    lines.push(format!("for candidate in {candidates}; do"));
    lines.push("\tif [ -e \"$candidate\" ]; then".into());
    lines.push("\t\troot=$candidate".into());
    lines.push("\t\tbreak".into());
    lines.push("\tfi".into());
    lines.push("done".into());
    lines
}

/// Return shell code to mount the basic virtual filesystems required by
/// an initramfs.
pub fn mount_base() -> Vec<String> {
    vec![
        "# Mount base virtual filesystems".into(),
        "mount -t proc proc /proc 2>/dev/null || true".into(),
        "mount -t sysfs sysfs /sys 2>/dev/null || true".into(),
        "mount -t devtmpfs devtmpfs /dev 2>/dev/null || true".into(),
        "mkdir -p /dev/pts /dev/shm".into(),
        "mount -t devpts devpts /dev/pts 2>/dev/null || true".into(),
        "mount -t tmpfs tmpfs /dev/shm 2>/dev/null || true".into(),
    ]
}

/// Return shell code that runs `switch_root` to transition to the real
/// root filesystem.
///
/// Expects `${root}` and `${init}` to be set.
pub fn do_switch_root() -> Vec<String> {
    vec![
        "# Switch to real root".into(),
        "if [ -x \"${root}/sbin/init\" ] || [ -x \"${root}/init\" ]; then".into(),
        "\texec switch_root \"${root}\" \"${init:-/sbin/init}\"".into(),
        "else".into(),
        "\techo '[ERROR] No init found on rootfs' >&2".into(),
        "\texit 1".into(),
        "fi".into(),
    ]
}

/// Return shell code that calls `switch_root` with explicit target and
/// init paths.
pub fn run_switch_root(target: &str, init: &str) -> Vec<String> {
    let target_escaped = target.replace('\'', "'\\''");
    let init_escaped = init.replace('\'', "'\\''");
    vec![format!(
        "exec switch_root '{target_escaped}' '{init_escaped}'"
    )]
}

/// Return shell code that sets up a basic rescue shell if something goes
/// wrong during init.
pub fn rescue_shell() -> Vec<String> {
    vec![
        "# Rescue shell".into(),
        "echo '[ERROR] Initramfs failed, starting rescue shell' >&2".into(),
        "if [ -x /bin/sh ]; then".into(),
        "\texec /bin/sh".into(),
        "elif [ -x /bin/busybox ]; then".into(),
        "\texec /bin/busybox sh".into(),
        "else".into(),
        "\techo '[FATAL] No shell available, kernel panic' >&2".into(),
        "\t/bin/busybox echo 'Kernel panic - not syncing'".into(),
        "\twhile true; do sleep 3600; done".into(),
        "fi".into(),
    ]
}

/// Return the shell `#!/bin/sh` shebang line.
pub fn shebang() -> String {
    "#!/bin/sh".to_owned()
}

/// Return shell code that sets `set -e` and other safe defaults.
pub fn shell_safe() -> Vec<String> {
    vec![
        "set -e".into(),
        "set -u".into(),
        "set -o pipefail 2>/dev/null || true".into(),
    ]
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_loglevel_generates_case() {
        let lines = set_loglevel();
        assert!(lines[0].contains("case"));
        assert!(lines[1].contains("[0-7])"));
        assert!(lines.last().unwrap().contains("esac"));
    }

    #[test]
    fn einfo_formats() {
        let result = einfo("hello");
        assert!(result.contains("[INFO]"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn ewarn_formats() {
        let result = ewarn("caution");
        assert!(result.contains("[WARN]"));
        assert!(result.contains("caution"));
    }

    #[test]
    fn eerror_formats() {
        let result = eerror("fail");
        assert!(result.contains("[ERROR]"));
        assert!(result.contains("fail"));
        assert!(result.contains(">&2"));
    }

    #[test]
    fn setvar_basic() {
        assert_eq!(setvar("root", "/dev/sda1"), "root=/dev/sda1");
    }

    #[test]
    fn readvar_basic() {
        let result = readvar("PATH");
        assert!(result.contains("PATH"));
    }

    #[test]
    fn find_root_empty() {
        let lines = find_root(&[]);
        assert!(lines[0].contains("root=\"\""));
    }

    #[test]
    fn find_root_with_candidates() {
        let lines = find_root(&["/dev/sda1", "/dev/nvme0n1p1"]);
        let joined = lines.join("\n");
        assert!(joined.contains("for candidate in"));
        assert!(joined.contains("/dev/sda1"));
        assert!(joined.contains("/dev/nvme0n1p1"));
        assert!(joined.contains("root=$candidate"));
    }

    #[test]
    fn mount_base_has_proc_sys_dev() {
        let lines = mount_base();
        let joined = lines.join("\n");
        assert!(joined.contains("mount -t proc"));
        assert!(joined.contains("mount -t sysfs"));
        assert!(joined.contains("mount -t devtmpfs"));
    }

    #[test]
    fn do_switch_root_generates_check() {
        let lines = do_switch_root();
        let joined = lines.join("\n");
        assert!(joined.contains("switch_root"));
        assert!(joined.contains("${root}"));
        assert!(joined.contains("${init:-/sbin/init}"));
    }

    #[test]
    fn run_switch_root_explicit() {
        let lines = run_switch_root("/newroot", "/sbin/init");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("/newroot"));
        assert!(lines[0].contains("/sbin/init"));
    }

    #[test]
    fn rescue_shell_contains_fallback() {
        let lines = rescue_shell();
        let joined = lines.join("\n");
        assert!(joined.contains("rescue shell"));
        assert!(joined.contains("/bin/sh"));
    }

    #[test]
    fn shebang_returns_hash_bang() {
        assert_eq!(shebang(), "#!/bin/sh");
    }

    #[test]
    fn shell_safe_sets_e_u() {
        let lines = shell_safe();
        assert!(lines.iter().any(|l| l.contains("set -e")));
        assert!(lines.iter().any(|l| l.contains("set -u")));
    }

    #[test]
    fn einfo_escapes_single_quote() {
        let result = einfo("it's broken");
        // The result should contain the INFO prefix.
        assert!(result.contains("[INFO]"));
        // The shell escaping mechanism replaces a single quote inside
        // single-quoted strings with: '\''  (end-quote, escaped-quote, start-quote)
        // So we should NOT see a raw unescaped "it's" in the output.
        // The actual output is: echo '[INFO] it'\''s broken'
        // We just verify it produced a well-formed string.
        assert!(!result.contains("it's broken"));
        // The replacement inserts "'\'" sequence — verify it has changed.
        assert_ne!(result, "echo '[INFO] it's broken'");
    }

    #[test]
    fn setvar_special_chars() {
        let result = setvar("foo", "bar baz");
        assert_eq!(result, "foo=bar baz");
    }
}
