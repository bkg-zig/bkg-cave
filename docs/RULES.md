# Regeln

## Verboten

- Funktionen hinzufügen
- Verhalten verändern
- Projekte vermischen
- APIs neu entwerfen
- BKG-spezifische Änderungen
- Optimierungen ohne Nachweis
- "Verbesserungen", die das Verhalten ändern
- Komponenten aus anderen Projekten übernehmen
- Neue Features entwickeln
- async ohne zwingenden Grund
- Neue Crates, wenn std ausreicht
- Architekturänderungen während der Portierung

## Erlaubt

- Shell → Rust
- Python → Rust
- C → Rust (soweit möglich)
- Buildsystem auf Cargo umstellen
- Fehlerbehandlung mit Result
- Logging mit tracing
- Tests ergänzen
- Dokumentation erweitern

## Lizenz

Originallizenz dokumentieren. Übernahme in eigene Werke erst nach expliziter Entscheidung prüfen.

## Migration-Status

- Not Started
- Analysed
- Porting
- Ported
- Verified
- Refactored
