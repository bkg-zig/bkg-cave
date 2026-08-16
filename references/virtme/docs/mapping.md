# mapping.md — virtme Mapping: Original → Rust

## CLI-Entrypoints

```
Original                                    → Rust-Modul
────────────────────────────────────────────────────────────────
virtme-run                                  → src/bin/virtme-run.rs
virtme-configkernel                         → src/bin/virtme-configkernel.rs
virtme-mkinitramfs                          → src/bin/virtme-mkinitramfs.rs
bin/virtme-prep-kdir-mods                   → src/bin/virtme-prep-kdir-mods.rs
```

## Python-Package → Rust-Crate

```
Original                                    → Rust
────────────────────────────────────────────────────────────────
virtme/commands/run.py                      → src/lib.rs (main orchestration)
virtme/commands/configkernel.py             → src/configkernel.rs
virtme/commands/mkinitramfs.py              → src/cli/mkinitramfs.rs
virtme/architectures.py                     → src/arch/mod.rs
virtme/qemu_helpers.py                      → src/qemu.rs
virtme/mkinitramfs.py                       → src/initramfs/mod.rs
virtme/cpiowriter.py                        → src/initramfs/cpio.rs
virtme/modfinder.py                         → src/modules.rs
virtme/virtmods.py                          → src/modules/modalias.rs
virtme/util.py                              → src/util.rs
virtme/resources.py                         → src/resources.rs
virtme/guest/virtme-init                    → src/guest/init.sh (embedded)
virtme/guest/virtme-udhcpc-script           → src/guest/udhcpc.sh (embedded)
```

## Funktion-Mapping

### commands/run.py

```
Originalfunktion                            → Rust-Funktion                     Status
──────────────────────────────────────────────────────────────────────────────────────
make_parser()                               → cli::make_parser()               Not Started
arg_fail(message)                           → util::arg_fail()                  Not Started
is_file_more_recent(a, b)                   → util::is_file_more_recent()       Not Started
class Kernel                                → struct Kernel                     Not Started
  Kernel.load_config(kdir)                  → Kernel::load_config()             Not Started
find_kernel_and_mods(arch, args)            → kernel::find_kernel_and_mods()    Not Started
export_virtfs(...)                          → qemu::export_virtfs()             Not Started
quote_karg(arg)                             → util::quote_karg()                Not Started
sanitize_disk_args(func, arg)              → qemu::sanitize_disk_args()        Not Started
do_it()                                     → run::execute()                    Not Started
main()                                      → main()                           Not Started
```

### architectures.py

```
Originalfunktion                            → Rust-Funktion                     Status
──────────────────────────────────────────────────────────────────────────────────────
class Arch                                  → trait Arch                        Not Started
  Arch.qemuargs(is_native)                  → Arch::qemuargs()                  Not Started
  Arch.virtio_dev_type(virtiotype)          → Arch::virtio_dev_type()           Not Started
  Arch.kimg_path()                          → Arch::kimg_path()                 Not Started
  Arch.dtb_path()                           → Arch::dtb_path()                  Not Started
  Arch.serial_dev_name(index)               → Arch::serial_dev_name()           Not Started
  Arch.earlyconsole_args()                  → Arch::earlyconsole_args()         Not Started
  Arch.serial_console_args()                → Arch::serial_console_args()       Not Started
  Arch.qemu_nodisplay_args()                → Arch::qemu_nodisplay_args()      Not Started
  Arch.config_base()                        → Arch::config_base()               Not Started
Arch_x86                                    → struct ArchX86                    Not Started
Arch_arm                                    → struct ArchArm                    Not Started
Arch_aarch64                                → struct ArchAarch64                Not Started
Arch_ppc64                                  → struct ArchPpc64                  Not Started
Arch_riscv64                                → struct ArchRiscv64                Not Started
Arch_sparc64                                → struct ArchSparc64                Not Started
Arch_s390x                                  → struct ArchS390x                  Not Started
Arch_unknown                                → struct ArchUnknown                Not Started
get(arch)                                   → arch::from_name()                 Not Started
```

### qemu_helpers.py

```
Originalfunktion                            → Rust-Funktion                     Status
──────────────────────────────────────────────────────────────────────────────────────
class Qemu                                  → struct Qemu                       Not Started
  Qemu.__init__(arch)                       → Qemu::new(arch)                   Not Started
  Qemu.probe()                              → Qemu::probe()                     Not Started
  Qemu.quote_optarg(a)                      → Qemu::quote_optarg()              Not Started
```

### mkinitramfs.py

```
Originalfunktion                            → Rust-Funktion                     Status
──────────────────────────────────────────────────────────────────────────────────────
class Config                                → struct InitramfsConfig            Not Started
mkinitramfs(out, config)                    → initramfs::build()                Not Started
find_busybox(root, is_native)              → util::find_busybox()              Not Started
generate_init(config)                       → initramfs::generate_init()        Not Started
make_base_layout(cw)                        → initramfs::make_base_layout()     Not Started
make_dev_nodes(cw)                          → initramfs::make_dev_nodes()       Not Started
install_busybox(cw, config)                 → initramfs::install_busybox()      Not Started
install_modprobe(cw)                        → initramfs::install_modprobe()    Not Started
install_modules(cw, modfiles)              → initramfs::install_modules()      Not Started
```

### cpiowriter.py

```
Originalfunktion                            → Rust-Funktion                     Status
──────────────────────────────────────────────────────────────────────────────────────
class CpioWriter                            → struct CpioWriter                 Not Started
  write_object(...)                          → CpioWriter::write_object()       Not Started
  write_trailer()                           → CpioWriter::write_trailer()       Not Started
  mkdir(name, mode)                         → CpioWriter::mkdir()               Not Started
  symlink(src, dst)                         → CpioWriter::symlink()             Not Started
  write_file(name, body, mode)             → CpioWriter::write_file()          Not Started
  mkchardev(name, dev, mode)               → CpioWriter::mkchardev()           Not Started
```
