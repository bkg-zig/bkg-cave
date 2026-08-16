//! CPIO archive writer producing archives in the **newc** (SVR4 portable) format.
//!
//! This module provides [`CpioWriter`], which can create CPIO archives
//! consumable by the Linux kernel (e.g. for initramfs).
//!
//! # Format overview
//!
//! Each archive entry consists of:
//!
//! 1. A 110-byte header with 6-byte magic `"070701"` followed by 13 fields
//!    encoded as 8-byte uppercase ASCII hexadecimal numbers.
//! 2. The null-terminated file name.
//! 3. Zero padding to bring the position to a 4-byte boundary.
//! 4. The file data.
//! 5. Zero padding to bring the position to a 4-byte boundary.
//!
//! The archive is terminated by a special entry with file name `"TRAILER!!!"`.

use std::io::Write;
use std::path::Path;

use crate::error::BkgCaveError;

// ── Constants ───────────────────────────────────────────────────────────────

/// CPIO newc magic bytes.
const CPIO_NEWC_MAGIC: &[u8; 6] = b"070701";

/// Length of the CPIO header in bytes.
const CPIO_HEADER_LEN: usize = 110;

/// Name of the trailer entry that marks the end of the archive.
const CPIO_TRAILER_NAME: &[u8; 10] = b"TRAILER!!!"; // + NUL = 11 bytes

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Compute padding to align the current position to the next 4-byte boundary.
fn align_to_4(pos: usize) -> usize {
    (4 - (pos % 4)) % 4
}

/// Write a single CPIO header field (8 uppercase hex digits) into a buffer.
fn write_hex_field(buf: &mut [u8; 8], val: u32) {
    let s = format!("{val:08X}");
    buf.copy_from_slice(s.as_bytes());
}

// ── CpioEntryType ───────────────────────────────────────────────────────────

/// The type of a CPIO archive entry.
///
/// Maps to the `c_mode` field's file-type bits in the newc header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpioEntryType {
    /// Regular file.
    File,
    /// Directory.
    Dir,
    /// Symbolic link.
    Symlink,
    /// Block device (requires root to create).
    Block,
    /// Character device (requires root to create).
    Char,
    /// Named pipe (FIFO).
    Fifo,
    /// Socket.
    Socket,
}

impl CpioEntryType {
    /// Return the file-type bits for the `c_mode` header field.
    #[allow(clippy::unusual_byte_groupings)]
    pub fn mode(&self) -> u32 {
        match self {
            CpioEntryType::File => 0o100_000,    // S_IFREG
            CpioEntryType::Dir => 0o040_000,     // S_IFDIR
            CpioEntryType::Symlink => 0o120_000, // S_IFLNK
            CpioEntryType::Block => 0o060_000,   // S_IFBLK
            CpioEntryType::Char => 0o020_000,    // S_IFCHR
            CpioEntryType::Fifo => 0o010_000,    // S_IFIFO
            CpioEntryType::Socket => 0o140_000,  // S_IFSOCK
        }
    }
}

// ── CpioWriter ──────────────────────────────────────────────────────────────

/// A writer that produces CPIO archives in the newc format.
///
/// # Example
///
/// ```ignore
/// use cave_core::cpio::{CpioWriter, CpioEntryType};
///
/// let mut buf = Vec::new();
/// {
///     let mut writer = CpioWriter::new(&mut buf);
///     writer.add_entry("hello.txt", CpioEntryType::File, b"world").unwrap();
///     writer.add_dir("mydir", 0o755).unwrap();
///     writer.finish().unwrap();
/// }
/// // `buf` now contains a valid CPIO archive.
/// ```
#[derive(Debug)]
pub struct CpioWriter<W: Write> {
    writer: W,
    ino_counter: u32,
    entry_count: u32,
}

impl<W: Write> CpioWriter<W> {
    /// Create a new `CpioWriter` wrapping the given `writer`.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            ino_counter: 1,
            entry_count: 0,
        }
    }

    /// Add an entry to the archive from raw bytes.
    ///
    /// `path` is the archive path (e.g. `"usr/bin/hello"`).  It is
    /// **not** null-terminated in Rust — the method adds the NUL.
    ///
    /// `entry_type` determines the file-type bits in the header.
    ///
    /// `data` is the raw content (for symlinks this is the link target).
    pub fn add_entry(
        &mut self,
        path: &str,
        entry_type: CpioEntryType,
        data: &[u8],
    ) -> Result<(), BkgCaveError> {
        let path_bytes = path.as_bytes();
        let name_len = path_bytes.len() + 1; // +1 for NUL terminator
        let mode = entry_type.mode()
            | match entry_type {
                CpioEntryType::File | CpioEntryType::Symlink => 0o777,
                CpioEntryType::Dir => 0o755,
                CpioEntryType::Block | CpioEntryType::Char => 0o600,
                CpioEntryType::Fifo | CpioEntryType::Socket => 0o644,
            };

        let filesize = data.len() as u32;

        // Write 110-byte header.
        self.write_header(
            self.ino_counter, // ino
            mode,
            0, // uid
            0, // gid
            1, // nlink
            0, // mtime
            filesize,
            0, // dev_maj
            0, // dev_min
            0, // rdev_maj
            0, // rdev_min
            name_len as u32,
            0, // check
        )?;

        // Track position for alignment calculations.
        // After the header we are at position 110.
        let mut pos: usize = CPIO_HEADER_LEN;

        // Write null-terminated name.
        self.writer.write_all(path_bytes)?;
        self.writer.write_all(&[0u8])?; // NUL
        pos += name_len;

        // Pad so the data starts at a 4-byte boundary.
        let name_pad = align_to_4(pos);
        if name_pad > 0 {
            self.writer.write_all(&[0u8; 4][..name_pad])?;
            pos += name_pad;
        }

        // Write file data.
        self.writer.write_all(data)?;
        pos += data.len();

        // Pad so the next entry starts at a 4-byte boundary.
        let data_pad = align_to_4(pos);
        if data_pad > 0 {
            self.writer.write_all(&[0u8; 4][..data_pad])?;
        }

        self.ino_counter += 1;
        self.entry_count += 1;
        Ok(())
    }

    /// Add a regular file from the filesystem.
    ///
    /// `target_path` is the archive path (e.g. `"bin/busybox"`).
    /// `source` is the path on disk to read the contents from.
    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        target_path: &str,
        source: P,
    ) -> Result<(), BkgCaveError> {
        let source = source.as_ref();
        let data = std::fs::read(source).map_err(BkgCaveError::Io)?;
        self.add_entry(target_path, CpioEntryType::File, &data)
    }

    /// Add a directory entry.
    ///
    /// `mode` is the permission bits (e.g. `0o755`).
    pub fn add_dir(&mut self, path: &str, mut mode: u32) -> Result<(), BkgCaveError> {
        // Ensure the directory type bits are set.
        mode |= CpioEntryType::Dir.mode();
        let path_bytes = path.as_bytes();
        let name_len = path_bytes.len() + 1;

        // Write 110-byte header.
        self.write_header(
            self.ino_counter,
            mode,
            0,
            0,
            2, // nlink = 2 for "." + ".."
            0, // mtime
            0, // filesize = 0 for directories
            0,
            0,
            0,
            0,
            name_len as u32,
            0,
        )?;

        let mut pos: usize = CPIO_HEADER_LEN;

        // Write null-terminated name.
        self.writer.write_all(path_bytes)?;
        self.writer.write_all(&[0u8])?;
        pos += name_len;

        // Pad so next entry starts at a 4-byte boundary.
        let pad = align_to_4(pos);
        if pad > 0 {
            self.writer.write_all(&[0u8; 4][..pad])?;
        }

        self.ino_counter += 1;
        self.entry_count += 1;
        Ok(())
    }

    /// Add a symbolic link.
    ///
    /// `path` is the link location in the archive.
    /// `target` is the link target (stored as the file data).
    pub fn add_symlink(&mut self, path: &str, target: &str) -> Result<(), BkgCaveError> {
        self.add_entry(path, CpioEntryType::Symlink, target.as_bytes())
    }

    /// Write the `TRAILER!!!` entry that terminates the CPIO archive.
    ///
    /// After calling `finish` the underlying writer is at the end of the
    /// archive and can be closed or flushed by the caller.
    pub fn finish(&mut self) -> Result<(), BkgCaveError> {
        let name = CPIO_TRAILER_NAME.as_slice();
        let name_len = name.len() + 1; // + NUL

        self.write_header(
            0, // ino
            0, // mode
            0,
            0,
            0,
            0,
            0, // filesize
            0,
            0,
            0,
            0,
            name_len as u32,
            0,
        )?;

        let mut pos: usize = CPIO_HEADER_LEN;

        self.writer.write_all(name)?;
        self.writer.write_all(&[0u8])?; // NUL
        pos += name_len;

        // Pad to 4-byte boundary (trailer has no data).
        let pad = align_to_4(pos);
        if pad > 0 {
            self.writer.write_all(&[0u8; 4][..pad])?;
        }

        Ok(())
    }

    /// Add a character device node.
    ///
    /// `path` is the archive path (e.g. `"dev/null"`).
    /// `major` and `minor` identify the device.
    /// `perm` is the permission bits (e.g. `0o666`).
    pub fn add_chardev(
        &mut self,
        path: &str,
        major: u32,
        minor: u32,
        perm: u32,
    ) -> Result<(), BkgCaveError> {
        self.add_device(path, CpioEntryType::Char, major, minor, perm)
    }

    /// Add a block device node.
    ///
    /// `path` is the archive path (e.g. `"dev/sda"`).
    /// `major` and `minor` identify the device.
    /// `perm` is the permission bits (e.g. `0o600`).
    pub fn add_blockdev(
        &mut self,
        path: &str,
        major: u32,
        minor: u32,
        perm: u32,
    ) -> Result<(), BkgCaveError> {
        self.add_device(path, CpioEntryType::Block, major, minor, perm)
    }

    // ── internal: device node helper ─────────────────────────────────────────

    fn add_device(
        &mut self,
        path: &str,
        entry_type: CpioEntryType,
        major: u32,
        minor: u32,
        perm: u32,
    ) -> Result<(), BkgCaveError> {
        let path_bytes = path.as_bytes();
        let name_len = path_bytes.len() + 1;
        let mode = entry_type.mode() | (perm & 0o777);

        self.write_header(
            self.ino_counter, // ino
            mode,
            0, // uid
            0, // gid
            1, // nlink
            0, // mtime
            0, // filesize – must be 0 for device nodes
            0, // dev_maj
            0, // dev_min
            major,
            minor,
            name_len as u32,
            0, // check
        )?;

        let mut pos: usize = CPIO_HEADER_LEN;
        self.writer.write_all(path_bytes)?;
        self.writer.write_all(&[0u8])?;
        pos += name_len;

        let pad = align_to_4(pos);
        if pad > 0 {
            self.writer.write_all(&[0u8; 4][..pad])?;
        }

        self.ino_counter += 1;
        self.entry_count += 1;
        Ok(())
    }

    /// Consume the writer and return the inner `W`.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Number of entries written so far (excluding the trailer).
    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }

    // ── internal helpers ────────────────────────────────────────────────────

    /// Write a full 110-byte CPIO newc header.
    #[allow(clippy::too_many_arguments)]
    fn write_header(
        &mut self,
        ino: u32,
        mode: u32,
        uid: u32,
        gid: u32,
        nlink: u32,
        mtime: u32,
        filesize: u32,
        dev_maj: u32,
        dev_min: u32,
        rdev_maj: u32,
        rdev_min: u32,
        namesize: u32,
        check: u32,
    ) -> Result<(), BkgCaveError> {
        let mut hdr = [0u8; CPIO_HEADER_LEN];

        // Magic (6 bytes).
        hdr[0..6].copy_from_slice(CPIO_NEWC_MAGIC);

        // 13 fields, each 8 bytes = 104 bytes, starting at offset 6.
        let mut offset = 6;
        let fields = [
            ino, mode, uid, gid, nlink, mtime, filesize, dev_maj, dev_min, rdev_maj, rdev_min,
            namesize, check,
        ];

        for &val in &fields {
            let mut buf = [0u8; 8];
            write_hex_field(&mut buf, val);
            hdr[offset..offset + 8].copy_from_slice(&buf);
            offset += 8;
        }

        self.writer.write_all(&hdr).map_err(BkgCaveError::Io)?;
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn write_single_file() {
        let mut buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut buf);
            writer
                .add_entry("test.txt", CpioEntryType::File, b"hello")
                .unwrap();
            writer.finish().unwrap();
        }

        assert!(buf.starts_with(b"070701"));
        assert!(buf.len() > 110);
    }

    #[test]
    fn write_directory() {
        let mut buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut buf);
            writer.add_dir("mydir", 0o755).unwrap();
            writer.finish().unwrap();
        }
        assert!(buf.starts_with(b"070701"));
    }

    #[test]
    fn write_symlink() {
        let mut buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut buf);
            writer.add_symlink("mylink", "/target/path").unwrap();
            writer.finish().unwrap();
        }
        assert!(buf.starts_with(b"070701"));
    }

    #[test]
    fn write_file_from_disk() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("input.bin");
        fs::write(&src, b"some binary content").unwrap();

        let mut buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut buf);
            writer.add_file("output.bin", &src).unwrap();
            writer.finish().unwrap();
        }
        assert!(buf.starts_with(b"070701"));
    }

    #[test]
    fn multiple_entries() {
        let mut buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut buf);
            writer.add_entry("a", CpioEntryType::File, b"1").unwrap();
            writer.add_entry("b", CpioEntryType::File, b"2").unwrap();
            writer.add_dir("d", 0o755).unwrap();
            writer.finish().unwrap();
        }
        assert!(buf.starts_with(b"070701"));
        // The trailer should be at the end — search for "TRAILER!!!" near the end.
        let trailer_start = buf.len().saturating_sub(128);
        assert!(
            buf[trailer_start..].windows(10).any(|w| w == b"TRAILER!!!"),
            "TRAILER!!! not found near end of archive"
        );
    }

    #[test]
    fn align_to_4_basics() {
        assert_eq!(align_to_4(0), 0);
        assert_eq!(align_to_4(1), 3);
        assert_eq!(align_to_4(2), 2);
        assert_eq!(align_to_4(3), 1);
        assert_eq!(align_to_4(4), 0);
        assert_eq!(align_to_4(110), 2); // 110 % 4 = 2
        assert_eq!(align_to_4(120), 0); // 120 % 4 = 0
        assert_eq!(align_to_4(121), 3); // 121 % 4 = 1
    }

    #[test]
    fn entry_type_modes() {
        assert_eq!(CpioEntryType::File.mode(), 0o100000);
        assert_eq!(CpioEntryType::Dir.mode(), 0o040000);
        assert_eq!(CpioEntryType::Symlink.mode(), 0o120000);
    }

    #[test]
    fn entry_count_tracking() {
        let mut buf = Vec::new();
        let mut writer = CpioWriter::new(&mut buf);
        assert_eq!(writer.entry_count(), 0);

        writer.add_entry("a", CpioEntryType::File, b"").unwrap();
        assert_eq!(writer.entry_count(), 1);

        writer.add_dir("d", 0o755).unwrap();
        assert_eq!(writer.entry_count(), 2);

        writer.finish().unwrap();
        // finish does NOT increment entry_count
        assert_eq!(writer.entry_count(), 2);
    }

    /// Verify that 110 + name + pad aligns the data correctly.
    #[test]
    fn name_padding_is_correct() {
        let mut buf = Vec::new();
        {
            let mut writer = CpioWriter::new(&mut buf);
            // "hello.txt" is 9 chars + NUL = 10 bytes
            // After header (110): 110 + 10 = 120 → 120 % 4 = 0, so NO padding needed.
            writer
                .add_entry("hello.txt", CpioEntryType::File, b"world")
                .unwrap();
            // "abc" is 3 chars + NUL = 4 bytes
            // After header (110) + 4 = 114 → 114 % 4 = 2, so 2 bytes padding.
            writer
                .add_entry("abc", CpioEntryType::File, b"data")
                .unwrap();
            writer.finish().unwrap();
        }

        // Check first entry alignment: name at 110, "hello.txt\0" = 10 bytes.
        // Data should start at 120 (4-byte aligned).
        let name1 = &buf[110..120]; // 10 bytes including NUL
        assert_eq!(&name1[..9], b"hello.txt");
        assert_eq!(name1[9], 0);
        // The data "world" should be right after name at byte 120
        assert_eq!(&buf[120..125], b"world");

        // Second entry header starts right after first entry's data padding.
        // First entry: 110 + 10 (name) + 0 (pad) + 5 (data) + 3 (pad) = 128
        // Second entry header at 128.
        let magic2 = &buf[128..134];
        assert_eq!(magic2, b"070701");
    }

    /// Verify that the produced archive is parseable by the `cpio` command.
    #[test]
    fn cpio_roundtrip() {
        let dir = TempDir::new().unwrap();
        let arc_path = dir.path().join("test.cpio");

        {
            let file = std::fs::File::create(&arc_path).unwrap();
            let mut writer = CpioWriter::new(file);
            writer
                .add_entry("hello.txt", CpioEntryType::File, b"world")
                .unwrap();
            writer.add_dir("subdir", 0o755).unwrap();
            writer
                .add_entry("subdir/nested.txt", CpioEntryType::File, b"nested")
                .unwrap();
            writer.finish().unwrap();
        }

        // Try bsdcpio first (more lenient), then GNU cpio.
        let cpio_bin = if crate::util::find_binary("bsdcpio").is_some() {
            "bsdcpio"
        } else if crate::util::find_binary("cpio").is_some() {
            "cpio"
        } else {
            return; // skip if no cpio
        };

        let output = std::process::Command::new(cpio_bin)
            .args(["-it", "-F"])
            .arg(&arc_path)
            .output()
            .expect("cpio command failed");

        assert!(
            output.status.success(),
            "{cpio_bin} -it failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello.txt"));
        assert!(stdout.contains("subdir"));
        assert!(stdout.contains("subdir/nested.txt"));
    }
}
