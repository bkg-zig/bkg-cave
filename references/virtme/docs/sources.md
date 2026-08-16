# sources.md — virtme

## Repository

- **Quelle**: virtme
- **URL**: https://github.com/amluto/virtme
- **Mirror**: references/virtme/upstream/virtme.git
- **Lizenz**: GPL-2.0
- **Autor**: Andy Lutomirski (amluto)

## Stand

- **Commit**: 3dfc002efdb2d8c06ff1306be276ec32f9bd00bb
- **Branch**: master
- **Tag**: —
- **Datum**: (letzter Commit) 3dfc002 — "run.py: Fix deprecated qemu readonly statement"
- **Weitere Branches**: kmod, testing

## Laufzeitabhängigkeiten

| Abhängigkeit | Version    | Zweck                                                  |
| ------------ | ---------- | ------------------------------------------------------ |
| Python       | >= 3.3     | Runtime                                                |
| QEMU         | >= 1.6     | Virtualisierung (>= 1.4 mit Einschränkungen)           |
| KVM          | (optional) | Hardwarebeschleunigung                                 |
| busybox      | (optional) | Statisches Initramfs-Binary                            |
| Linux Kernel | —          | Zu testender Kernel (CONFIG_VIRTIO, CONFIG_9P_FS u.a.) |

## Kernel-Requirements (Gast)

Folgende Kernel-Optionen werden benötigt (built-in oder als Module):

- CONFIG_VIRTIO
- CONFIG_VIRTIO_PCI
- CONFIG_NET_9P
- CONFIG_NET_9P_VIRTIO
- CONFIG_9P_FS
- CONFIG_VIRTIO_NET (für Netzwerk)
- CONFIG_VIRTIO_CONSOLE (für Script-Support)
- CONFIG_SCSI_VIRTIO (für Disk-Support)

## Standards

- POSIX Shell (Init-Skripte)
- Python 3

## Projektstruktur (Original)

```
virtme/
├── virtme-run                 ← CLI Entrypoint: VM starten
├── virtme-configkernel        ← CLI Entrypoint: Kernel konfigurieren
├── virtme-mkinitramfs         ← CLI Entrypoint: Initramfs bauen
├── bin/
│   └── virtme-prep-kdir-mods  ← Kernel-Module vorbereiten
├── virtme/
│   ├── __init__.py
│   ├── architectures.py       ← Architektur-Abstraktion (x86, ARM, RISC-V, PPC64)
│   ├── commands/
│   │   ├── __init__.py
│   │   ├── configkernel.py    ← Kernel-Konfigurationslogik
│   │   ├── mkinitramfs.py     ← Initramfs-Erzeugung
│   │   └── run.py             ← Hauptlogik: QEMU-Aufruf bauen und starten
│   ├── cpiowriter.py          ← CPIO-Archiv schreiben
│   ├── guest/
│   │   ├── __init__.py
│   │   ├── virtme-init        ← Init-Skript im Gast
│   │   └── virtme-udhcpc-script ← DHCP-Client-Skript im Gast
│   ├── mkinitramfs.py         ← Initramfs-Logik (busybox-basiert)
│   ├── modfinder.py           ← Kernel-Module finden
│   ├── qemu_helpers.py        ← QEMU-Parameter zusammenbauen
│   ├── resources.py           ← Resource-Pfade
│   ├── scripts                ← (Shell-Script-Referenz)
│   └── util.py                ← Hilfsfunktionen
├── samples/
│   ├── README.md
│   ├── xfstests
│   └── xfstests_trampoline
├── setup.py
├── make-release-tarball.sh
├── DCO-1.1.txt
├── DCO.txt
└── LICENSE
```
