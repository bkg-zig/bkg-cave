# traceability.md — virtme Traceability

## Vorgehen

Traceability verbindet jede Rust-Funktion mit ihrer Original-Quelle.
Wird während des Ports befüllt.

## Template pro Eintrag

```yaml
source:
  repo: https://github.com/amluto/virtme
  commit: 3dfc002efdb2d8c06ff1306be276ec32f9bd00bb
  branch: master
  license: GPL-2.0
  file: virtme/commands/run.py
  function: do_it
  lines: 282-633
  purpose: Hauptlogik – baut QEMU-Kommandozeile und startet VM

target:
  crate: virtme
  module: src/lib.rs
  function: execute
  status: Not Started

tests:
  - tests/qemu_build.rs::test_full_cmdline
  - tests/qemu_build.rs::test_kernel_args

docs:
  - docs/api.md::do_it
  - docs/startup.md
```

## Einträge

_(Wird während Rust-Port befüllt)_
