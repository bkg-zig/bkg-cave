# api.md — virtme Öffentliche APIs

## Module: `virtme.commands.run`

### `make_parser() -> argparse.ArgumentParser`

Erzeugt den Argument-Parser für virtme-run.

- **Parameter**: Keine
- **Rückgabe**: Konfigurierter ArgumentParser
- **Fehlerfälle**: Keine

### `arg_fail(message, show_usage=True) -> NoReturn`

Gibt Fehlermeldung aus und beendet.

- **Parameter**: message (str), show_usage (bool)
- **Rückgabe**: Beendet das Programm
- **Fehlerfälle**: Immer

### `find_kernel_and_mods(arch, args) -> Kernel`

Findet Kernel-Image und zugehörige Module.

- **Parameter**: arch (Arch), args (Namespace)
- **Rückgabe**: Kernel-Objekt mit kimg, dtb, modfiles, moddir, config
- **Fehlerfälle**: SilentError bei fehlendem Kernel/Modulen

### `export_virtfs(qemu, arch, qemuargs, path, mount_tag, security_model, readonly)`

Exportiert ein Host-Verzeichnis als 9p-Filesystem ins Gast.

- **Parameter**: qemu (Qemu), arch (Arch), qemuargs (List), path (str), mount_tag (str), security_model (str), readonly (bool)
- **Rückgabe**: None (modifiziert qemuargs direkt)

### `quote_karg(arg: str) -> str`

Quoted ein Kernel-Argument (bei Leerzeichen in Anführungszeichen).

- **Parameter**: arg (str)
- **Rückgabe**: Gequotetes Argument
- **Fehlerfälle**: ValueError bei enthaltenen Anführungszeichen

### `sanitize_disk_args(func: str, arg: str) -> Tuple[str, str]`

Validiert name=path-Argumente für --disk/--blk-disk.

- **Parameter**: func (str), arg (str)
- **Rückgabe**: Tuple (name, filename)
- **Fehlerfälle**: arg_fail bei Fehlern

### `do_it() -> int`

Hauptfunktion: Baut QEMU-Kommandozeile und startet VM.

- **Parameter**: Keine (liest args aus globalem Parser)
- **Rückgabe**: Exit-Code (0 = Erfolg)
- **Fehlerfälle**: SilentError bei Fehlern

### `main() -> int`

Entrypoint mit Fehlerbehandlung.

- **Parameter**: Keine
- **Rückgabe**: Exit-Code

## Module: `virtme.commands.configkernel`

### `make_parser() -> argparse.ArgumentParser`

Erzeugt den Argument-Parser für virtme-configkernel.

- **Rückgabe**: ArgumentParser

### `main() -> int`

Konfiguriert einen Kernel für virtme.

- **Rückgabe**: Exit-Code (0 = Erfolg)

## Module: `virtme.commands.mkinitramfs`

### `make_parser() -> argparse.ArgumentParser`

Erzeugt den Argument-Parser für virtme-mkinitramfs.

- **Rückgabe**: ArgumentParser

### `main() -> int`

Erzeugt ein Initramfs-Image.

- **Rückgabe**: Exit-Code (0 = Erfolg)

## Module: `virtme.architectures`

### `get(arch: str) -> Arch`

Gibt Architektur-Objekt für einen Namen zurück.

- **Parameter**: arch (str) — z.B. "x86_64", "arm", "aarch64"
- **Rückgabe**: Arch-Instanz
- **Fehlerfälle**: Fallback auf Arch_unknown bei unbekannter Arch

### Klasse `Arch`

Basis-Architektur-Abstraktion.

- `qemuargs(is_native) -> List[str]`
- `virtio_dev_type(virtiotype) -> str`
- `kimg_path() -> str`
- `dtb_path() -> Optional[str]`
- `serial_dev_name(index) -> str`
- `earlyconsole_args() -> List[str]`
- `serial_console_args() -> List[str]`
- `qemu_nodisplay_args() -> List[str]`
- `config_base() -> List[str]`

### Klasse `Arch_x86(Arch)`

x86_64/i386-Architektur. CPU host + Watchdog + KVM-Gast.

### Klasse `Arch_arm(Arch)`

ARM vexpress-a15. Kein PCI, DTB erforderlich.

### Klasse `Arch_aarch64(Arch)`

Aarch64 mit virt Machine. Host CPU oder cortex-a57.

### Klasse `Arch_ppc64(Arch)`

PPC64 mit pseries Machine. vmlinux statt zImage.

### Klasse `Arch_riscv64(Arch)`

RISC-V 64 mit virt Machine + OpenSBI.

### Klasse `Arch_sparc64(Arch)`

SPARC64. Kein -display none.

### Klasse `Arch_s390x(Arch)`

S390x mit s390-ccw-virtio + sclpconsole.

## Module: `virtme.qemu_helpers`

### Klasse `Qemu`

QEMU-Findung und Capabilities.

- `__init__(arch)`: Sucht qemu-system-<arch>
- `probe()`: Ermittelt Version, multidevs, virtfs-Features
- `quote_optarg(a)`: Quoted Kommas in QEMU-Optionswerten

## Module: `virtme.mkinitramfs`

### Klasse `Config`

Initramfs-Konfiguration. Slots: modfiles, virtme_data, virtme_init_path, busybox, access

### `mkinitramfs(out, config) -> None`

Baut komplettes Initramfs in einen writeable Buffer.

- **Parameter**: out (BinaryIO), config (Config)
- **Seiteneffekte**: Schreibt CPIO-Archiv in out

### `find_busybox(root, is_native) -> Optional[str]`

Sucht busybox-Binary im System.

- **Parameter**: root (str), is_native (bool)
- **Rückgabe**: Pfad zu busybox oder None

## Module: `virtme.cpiowriter`

### Klasse `CpioWriter`

Schreibt CPIO-Archive im newc-Format.

- `mkdir(name, mode)`: Verzeichnis anlegen
- `symlink(src, dst)`: Symlink anlegen
- `write_file(name, body, mode)`: Datei schreiben
- `mkchardev(name, dev, mode)`: Char-Device anlegen
- `write_trailer()`: CPIO-Trailer schreiben

## Module: `virtme.modfinder`

### `find_modules_from_install(aliases, root, kver, moddir) -> List[str]`

Löst Module-Aliase in Dateipfade auf.

- **Parameter**: aliases (List[str]), root, kver, moddir (optional)
- **Rückgabe**: Liste der Modul-Pfade

## Module: `virtme.util`

### `find_binary(names, root, use_path) -> Optional[str]`

Sucht Binary in Standard-Pfaden.

- **Rückgabe**: Pfad oder None

### `find_binary_or_raise(names, root, use_path) -> str`

Wie find_binary, aber wirft Exception bei Nicht-Fund.

- **Fehlerfälle**: RuntimeError
