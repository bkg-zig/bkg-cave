# traceability.md — osbuild Traceability

## Vorgehen

Traceability verbindet jede Rust-Funktion mit ihrer Original-Quelle.
Wird während des Ports befüllt.

## Template pro Eintrag

```yaml
source:
  repo: https://github.com/osbuild/osbuild
  commit: _
  branch: _
  license: _
  file: _
  function: _
  lines: _
  purpose: _

target:
  crate: osbuild
  module: _
  function: _
  status: Not Started

tests:
  - _

docs:
  - _
```

## Einträge

_(Wird während Rust-Port befüllt)_
