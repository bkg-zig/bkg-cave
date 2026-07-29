//! Shell-code generator for initramfs scripts and other POSIX shell fragments.
//!
//! All methods return raw strings that can be concatenated or collected into a
//! final shell script.

/// Generator for POSIX `/bin/sh` shell code.
///
/// Each method returns one or more lines of shell code.  The caller is
/// responsible for joining lines and writing the result.
///
/// # Example
///
/// ```ignore
/// use cave_core::ShellCodeGenerator as sh;
///
/// let script = format!(
///     "{shebang}{set_root}{echo}",
///     shebang = sh::shebang(),
///     set_root = sh::set_var("root", "/dev/sda1"),
///     echo = sh::echo("hello world"),
/// );
/// ```
pub struct ShellCodeGenerator;

impl ShellCodeGenerator {
    /// Returns `#!/bin/sh\n`.
    pub fn shebang() -> String {
        "#!/bin/sh\n".to_owned()
    }

    /// Returns a variable assignment: `key=val\n`.
    pub fn set_var(key: &str, val: &str) -> String {
        format!("{key}={val}\n")
    }

    /// Returns an export statement: `export key=val\n`.
    pub fn export(key: &str, val: &str) -> String {
        format!("export {key}={val}\n")
    }

    /// Returns a `.` (source) command: `. path\n`.
    pub fn source(path: &str) -> String {
        format!(". {path}\n")
    }

    /// Returns an `echo` command: `echo 'msg'\n`.
    ///
    /// Single quotes inside `msg` are escaped by ending the current quote,
    /// inserting an escaped single-quote, and resuming.
    pub fn echo(msg: &str) -> String {
        let escaped = msg.replace('\'', "'\\''");
        format!("echo '{escaped}'\n")
    }

    /// Returns a raw command followed by a newline: `command\n`.
    pub fn cmd(command: &str) -> String {
        format!("{command}\n")
    }

    /// Generates a `if command; then …; fi` block.
    ///
    /// `then_body` is indented by one tab.
    pub fn if_has_cmd(cmd: &str, then_body: &[String]) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("if {cmd}; then"));
        for line in then_body {
            // Indent by one tab.
            lines.push(format!("\t{line}"));
        }
        lines.push("fi".into());
        lines
    }

    /// Generates a `if ! command; then …; fi` block.
    ///
    /// `then_body` is indented by one tab.
    pub fn if_not_cmd(cmd: &str, then_body: &[String]) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("if ! {cmd}; then"));
        for line in then_body {
            lines.push(format!("\t{line}"));
        }
        lines.push("fi".into());
        lines
    }

    /// Returns an error-exit sequence: `echo 'msg' >&2; exit 1\n`.
    ///
    /// Single quotes inside `msg` are escaped appropriately.
    pub fn error_exit(msg: &str) -> String {
        let escaped = msg.replace('\'', "'\\''");
        format!("echo '{escaped}' >&2; exit 1\n")
    }

    /// Returns a shell comment line: `# msg\n`.
    pub fn comment(msg: &str) -> String {
        format!("# {msg}\n")
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shebang_correct() {
        assert_eq!(ShellCodeGenerator::shebang(), "#!/bin/sh\n");
    }

    #[test]
    fn set_var_basic() {
        assert_eq!(ShellCodeGenerator::set_var("FOO", "bar"), "FOO=bar\n");
    }

    #[test]
    fn export_basic() {
        assert_eq!(
            ShellCodeGenerator::export("PATH", "/usr/bin"),
            "export PATH=/usr/bin\n"
        );
    }

    #[test]
    fn source_basic() {
        assert_eq!(
            ShellCodeGenerator::source("/etc/profile"),
            ". /etc/profile\n"
        );
    }

    #[test]
    fn echo_simple() {
        assert_eq!(ShellCodeGenerator::echo("hi"), "echo 'hi'\n");
    }

    #[test]
    fn echo_with_single_quote() {
        let result = ShellCodeGenerator::echo("it's me");
        // Should escape the single quote: echo 'it'\''s me'
        assert_eq!(result, "echo 'it'\\''s me'\n");
    }

    #[test]
    fn cmd_basic() {
        assert_eq!(
            ShellCodeGenerator::cmd("mount -t proc proc /proc"),
            "mount -t proc proc /proc\n"
        );
    }

    #[test]
    fn if_has_cmd_basic() {
        let body = vec![ShellCodeGenerator::echo("found")];
        let result = ShellCodeGenerator::if_has_cmd("which foo", &body);
        let joined = result.join("\n");
        assert!(joined.starts_with("if which foo; then"));
        assert!(joined.contains("\techo 'found'"));
        assert!(joined.ends_with("fi"));
    }

    #[test]
    fn if_not_cmd_basic() {
        let body = vec![ShellCodeGenerator::echo("missing")];
        let result = ShellCodeGenerator::if_not_cmd("which foo", &body);
        let joined = result.join("\n");
        assert!(joined.starts_with("if ! which foo; then"));
        assert!(joined.contains("\techo 'missing'"));
        assert!(joined.ends_with("fi"));
    }

    #[test]
    fn error_exit_basic() {
        let result = ShellCodeGenerator::error_exit("something bad");
        assert!(result.contains("echo 'something bad' >&2"));
        assert!(result.contains("exit 1"));
    }

    #[test]
    fn comment_basic() {
        assert_eq!(
            ShellCodeGenerator::comment("this is a comment"),
            "# this is a comment\n"
        );
    }
}
