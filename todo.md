# Rust-Migrationsplan für Referenzprojekte

## Ziel

Dieses Dokument beschreibt den verbindlichen Ablauf für die Migration von Referenzprojekten nach Rust.

Die Referenzprojekte dienen ausschließlich als Wissensquelle.

Es werden **keine** neuen Funktionen entwickelt.

Es werden **keine** Projekte miteinander kombiniert.

Es wird **keine** neue Architektur entworfen.

Ziel ist ausschließlich eine möglichst vollständige, idiomatische Rust-Portierung mit identischem Verhalten.

---

# Grundregeln

## Verboten

* Funktionen hinzufügen
* Verhalten verändern
* Projekte vermischen
* APIs neu entwerfen
* BKG-spezifische Änderungen
* Optimierungen ohne Nachweis
* "Verbesserungen", die das Verhalten ändern
* Komponenten aus anderen Projekten übernehmen
* Neue Features entwickeln

## Erlaubt

* Shell → Rust
* Python → Rust
* C → Rust (soweit möglich)
* Buildsystem auf Cargo umstellen
* Fehlerbehandlung mit Result
* Logging mit tracing
* Tests ergänzen
* Dokumentation erweitern

---

# Für jedes Referenzprojekt

Es wird ein eigenes Cargo Workspace erstellt.

Beispiel:

project-name/

* Cargo.toml
* README.md
* LICENSE
* docs/
* src/
* tests/
* xtask/

Keine Abhängigkeiten zwischen den Referenzprojekten.

---

# Pflichtdokumente

## docs/analysis.md

Enthält:

* Gesamtübersicht
* Architektur
* Module
* Datenfluss
* Initialisierung
* Buildprozess
* Startsequenz
* Dateistruktur
* externe Bibliotheken
* bekannte Einschränkungen

---

## docs/migration.md

Für jede Quelldatei:

Originaldatei

↓

Rust-Modul

Beschreibung

Status

Risiken

Offene Punkte

---

## docs/api.md

Beschreibung aller öffentlichen APIs.

* Funktionen
* Parameter
* Rückgabewerte
* Fehlerfälle

---

## docs/tests.md

Liste aller Tests.

* übernommen
* neu erstellt
* noch fehlend

---

## docs/todo.md

Nur technische Restarbeiten.

Keine Architekturideen.

---

## docs/sources.md

Pflichtdatei.

Für jede Funktion muss die Herkunft dokumentiert werden.

Beispiel:

* Quelle
* Repository
* Commit
* Datei
* Zeilenbereich
* Zweck

Keine Funktion ohne dokumentierte Herkunft.

---

# Migration

Für jede Datei:

1.

Analysieren

2.

Abhängigkeiten bestimmen

3.

Verhalten dokumentieren

4.

Rust-Version erstellen

5.

Tests

6.

Verhalten vergleichen

7.

Erst danach Refactoring

---

# Refactoring-Regeln

Refactoring ist erst erlaubt wenn:

* Rust-Code kompiliert
* Tests erfolgreich
* Verhalten identisch

Vorher keine Änderungen an:

* API
* Datenfluss
* Initialisierung
* Ablauf

---

# Tests

Pflicht:

* Unit Tests
* Integration Tests
* Regression Tests

Falls möglich:

Golden Tests

Original

↓

Rust

↓

Ausgabe identisch

---

# Logging

Nur tracing.

Keine println-Ausgaben.

---

# Fehlerbehandlung

Keine panics für normale Fehler.

Nur:

Result

thiserror

anyhow

---

# Sicherheit

Alle unsafe-Blöcke müssen:

* dokumentiert werden
* begründet werden
* minimiert werden

---

# Codequalität

Pflicht:

cargo fmt

cargo clippy

cargo test

cargo doc

---

# Externe Quellen

Für jedes Referenzprojekt muss dokumentiert werden:

Repository

URL

Lizenz

Autor

Commit

Branch

Tag

Dokumentation

Verwendete Standards

---

# Referenzprojekte

## virtme

Quelle:

https://github.com/amluto/virtme

Zu analysieren:

* Kernelstart
* RootFS
* QEMU
* KVM
* Netzwerk
* Shared Filesystem
* Startparameter
* CLI

---

## ugrd

Quelle:

https://github.com/desultory/ugrd

Zu analysieren:

* Initramfs
* Bootprozess
* Module
* Hooks
* Mounts
* Storage
* Netzwerk
* Overlay
* Konfiguration

---

## osbuild

Quelle:

https://github.com/osbuild/osbuild

Zu analysieren:

* Pipeline
* Buildgraph
* Imageaufbau
* Stages
* Manifest
* Reproduzierbarkeit

---

# Agent-Aufgaben

## Task 1

* Status: offen
* Priorität: P1
* Frist: offen
* Abhängigkeiten: keine

Repository vollständig analysieren.

Ergebnis:

docs/analysis.md

---

## Task 2

* Status: offen
* Priorität: P1
* Frist: nach Task 1
* Abhängigkeiten: Task 1

Dateien inventarisieren.

Ergebnis:

docs/files.md

---

## Task 3

* Status: offen
* Priorität: P1
* Frist: nach Task 2
* Abhängigkeiten: Task 2

Abhängigkeiten dokumentieren.

Ergebnis:

docs/dependencies.md

---

## Task 4

* Status: offen
* Priorität: P2
* Frist: nach Task 3
* Abhängigkeiten: Task 3

Startsequenz dokumentieren.

Ergebnis:

docs/startup.md

---

## Task 5

* Status: offen
* Priorität: P1
* Frist: nach Task 4
* Abhängigkeiten: Tasks 1–4

Rust-Port erstellen.

---

## Task 6

* Status: offen
* Priorität: P1
* Frist: nach Task 5
* Abhängigkeiten: Task 5

Tests schreiben.

---

## Task 7

* Status: offen
* Priorität: P1
* Frist: nach Task 6
* Abhängigkeiten: Task 6

Original gegen Rust vergleichen.

---

## Task 8

* Status: offen
* Priorität: P2
* Frist: nach Task 7
* Abhängigkeiten: Task 7

Dokumentation vervollständig.

---

# Qualitätskriterien

Die Migration gilt erst als abgeschlossen wenn:

* alle Funktionen portiert sind
* alle Quellen dokumentiert sind
* alle Tests erfolgreich sind
* Verhalten identisch ist
* keine Shell-Skripte mehr erforderlich sind
* Cargo Build erfolgreich ist
* Clippy ohne Fehler läuft
* Formatierung vollständig ist

---

# Wichtig

Diese Phase dient ausschließlich der Wissensgewinnung.

Es wird ausdrücklich **keine** neue Runtime entwickelt.

Es wird ausdrücklich **keine** neue Architektur entworfen.

Es wird ausdrücklich **kein** Code zwischen Projekten übernommen.

Die Referenzprojekte bleiben vollständig voneinander getrennt.

Erst nach Abschluss aller Migrationen beginnt eine separate Architekturphase, in der Gemeinsamkeiten ausgewertet und eine neue Implementierung entworfen wird.
