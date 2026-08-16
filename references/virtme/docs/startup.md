# startup.md — virtme Startsequenz

## 1. CLI-Start (virtme-run)

```
┌─────────────────────────────────────────────────────┐
│ 1. virtme-run (Shebang → python3)                   │
│    → virtme.commands.run:main()                     │
│                                                      │
│ 2. make_parser() → argparse.ArgumentParser           │
│    - Kernel: --installed-kernel / --kimg / --kdir    │
│    - Gast: --root / --rw / --graphics / --net        │
│    - Ressourcen: --disk / --blk-disk / --rwdir       │
│    - Scripting: --script-sh / --script-exec          │
│    - Debug: --dry-run / --show-command /--save-...  │
│                                                      │
│ 3. do_it():                                          │
│    a) Architektur ermitteln (architectures.get)      │
│    b) QEMU suchen (qemu_helpers.Qemu().probe())      │
│    c) Kernel + Module finden (find_kernel_and_mods)  │
│    d) Config: mkinitramfs.Config()                   │
│    e) QEMU-Argumente bauen                           │
│    f) Initramfs bauen (mkinitramfs.mkinitramfs)      │
│    g) os.execv(qemu, qemuargs)                       │
└─────────────────────────────────────────────────────┘
```

## 2. Initialisierung im Detail

### Schritt A: Architektur

- `architectures.get(args.arch)` → Arch-Objekt
- Unterstützt: x86_64, i386, arm, aarch64, ppc64, riscv64, sparc64, s390x
- Fallback: `Arch_unknown`

### Schritt B: QEMU-Probe

- `qemu_helpers.Qemu(arch.qemuname)` → sucht `qemu-system-<arch>`
- Fallback natives: `qemu-kvm`
- `probe()` liest `qemu --version`
- Prüft: `cannot_overmount_virtfs` (QEMU < 1.6)
- Prüft: `has_multidevs` (QEMU >= 4.2)

### Schritt C: Kernel finden

```python
find_kernel_and_mods(arch, args)
  ├── --installed-kernel: /usr/lib/modules/<ver>/vmlinuz
  │   └── modfiles = modfinder.find_modules_from_install(aliases, kver)
  ├── --kdir: <kdir>/arch/<arch>/boot/bzImage + .virtme_mods
  │   └── .config einlesen → CONFIG_MODULES prüfen
  │   └── modmode: none/use/auto
  │       └── auto → virtme-prep-kdir-mods ausführen
  │       └── modfiles = modfinder.find_modules_from_install(aliases, root=virtme_mods)
  └── --kimg: direkter Kernel-Image-Pfad (keine Module)
```

### Schritt D: QEMU-Argumente bauen

1. `-name` setzen
2. **VirtFS**: `/dev/root` (9p) → RootFS einbinden
3. **VirtFS**: `virtme.guesttools` (9p) → Gast-Tools
4. **Kernel-Args**: Module-Pfade, `virtme_root_mods` / `virtme_link_mods`
5. **Zusätzliche Mounts**: `--rwdir` / `--rodir` als 9p-Freigaben
6. **KVM**: `-machine accel=kvm:tcg` (wenn nativ)
7. **Arch-spezifische Optionen**: CPU, Watchdog, etc.
8. **Console/Display**: Serial Console oder `--graphics`
9. **Optional**: Netzwerk, Disks, Balloon, Memory, CPUs
10. **Scripting**: virtserialport + script-Daten
11. **Kernel-Argumente**: `init=/bin/sh` + `--` + `initcmds`

### Schritt E: Initramfs bauen

- Wenn erforderlich (Module, Scripting, `--force-initramfs`, QEMU < 1.6):
  1. `CpioWriter` → CPIO-Archiv
  2. Basis-Layout: `/bin`, `/lib`, `/dev`, `/proc`, `/run_virtme`
  3. Device-Nodes: `/dev/null`, `/dev/kmsg`, `/dev/console`
  4. `busybox` installieren + Symlinks (sh, mount, umount, switch_root, ...)
  5. `modprobe`-Stub installieren
  6. Module-Dateien kopieren + `load_all.sh`
  7. `init`-Skript generieren (mount root, switch_root)
  8. `TRAILER!!!` im CPIO

### Schritt F: Start

- `os.execv(qemu.qemubin, qemuargs)` → ersetzt den Prozess durch QEMU

## 3. Gast-Startsequenz (virtme-init)

```
virtme-init (PID 1)
  │
  ├── 1. sysfs mounten
  ├── 2. 9p-Mount-Tags erkennen (/sys/bus/virtio/drivers/9pnet_virtio/)
  ├── 3. Kernel-Version ermitteln (`uname -r`)
  ├── 4. Module einrichten:
  │   ├── virtme_root_mods → /lib/modules vom Host
  │   ├── virtme_link_mods → Symlink
  │   └── sonst → /lib/modules/<kver> maskieren
  ├── 5. tmpfs für /tmp, /var/log
  ├── 6. /etc/fstab → tmpfs, /etc/shadow → dummy
  ├── 7. udevd suchen und starten
  ├── 8. proc mounten
  ├── 9. devtmpfs mounten
  ├── 10. Init-Mounts (--rwdir/--rodir) einbinden
  ├── 11. systemd-tmpfiles ausführen (falls vorhanden)
  ├── 12. udev coldplug: trigger + settle
  ├── 13. configfs, debugfs, tracefs mounten
  ├── 14. cgroupv2 einrichten
  ├── 15. devpts, /dev/shm mounten
  ├── 16. /proc/self/fd-Links einrichten
  ├── 17. Hostname setzen
  ├── 18. Netzwerk (loopback + DHCP falls --net)
  │
  ├── [Script-Modus]: script ausführen → poweroff
  │
  └── [Console-Modus]: Console erkennen → Login-Shell (loop)
```

## 4. Kernel-Konfiguration (virtme-configkernel)

```
virtme-configkernel
  → configkernel.main()
  ├── Prüfung: in Kernel-Source-Verzeichnis
  ├── Architektur bestimmen
  ├── Konfiguration zusammensetzen
  │   ├── Generic Config (30+ Einträge)
  │   └── Arch-spezifische Config
  ├── Mode: --allnoconfig / --defconfig / --update
  ├── make <target> ausführen
  ├── Konfig an .config anhängen
  └── make olddefconfig / syncconfig
```

## 5. Initramfs-CLI (virtme-mkinitramfs)

```
virtme-mkinitramfs
  → mkinitramfs.main()
  ├── Config erstellen
  ├── Module suchen (optional)
  ├── busybox suchen
  └── mkinitramfs.mkinitramfs(buf, config)
```
