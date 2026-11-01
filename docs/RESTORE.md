# Restore-Status: bkg-cave Cave-Linie

> **Wiederhergestellt:** 2026-11-01
> **Task-ID:** `bkg-cave-cave-restore-20261101`
> **Debatte:** `docs/debat/` (2026-11-01) — 3/3 APPROVE_CONDITIONAL
> **Entscheidungsrecord:** `.brain/decisions.json` → `bkg-cave-cave-restore-20261101`

## 1. Was passiert ist

Die Historie von `bkg-cave` war verzweigt: eine Scaffold-Linie (`d6451ac` → `3f677a1`)
und eine vollständige Cave-Linie (`f28cfc9` → `cd977a0`). Ursache der Verwirrung war ein
Branch-Rename `archive/scaffold-main` → `main` am 2026-11-01 00:14:55, durch den `main`
auf den Scaffold-Stand zeigte und die Cave-Commits nur noch als dangling objects vorlagen.

Die Cave-Linie wurde per Cherry-Pick in das Arbeits-Remote
`https://github.com/bkg-zig/bkg-cave.git` eingespielt:

| Commit | Quelle | Ergebnis |
|---|---|---|
| `d32d487` | `f28cfc9` *(cherry picked from commit f28cfc9…)* | Cave-Vollstand über Scaffold-Stand |
| `295abf6` | `cd977a0` *(cherry picked from commit cd977a0…)* | `.gitignore` (Upstream + lokale Ignore-Regeln) |

Die 14 `add/add`-Konflikte (`AGENTS.md`, `Cargo.lock`, `crates/cave-p2p/*`) wurden
bewusst mit der Cave-Version aufgelöst — der Scaffold-Stand ist dort ein älteres
Skeleton desselben Crates, die Cave-Version die vollständige Implementierung.

## 2. Aktueller Dateibestand

`git ls-files` = **174 Dateien** = 149 Dateien aus `cd977a0` (byte-identisch, per
`git rev-parse` einzeln geprüft) + `.gitignore` (Superset) + 24 scaffold-exklusive
Dokumentationsdateien.

### 24 scaffold-exklusive Dateien (bewusst erhalten, nicht gelöscht)

`README.md`, `ROADMAP.md`, `docs/cave/*` (11), `docs/legendery/*` (3),
`docs/p2p/*` (7), `docs/plans/*` (2).

### Cave-exklusive Inhalte (wiederhergestellt)

`references/{virtme,ugrd,osbuild,bkg-p2p}`, `todo.md`, `ex-sh`,
`bkg-cave.code-workspace`, `.ignore`, `docs/P2P_INVENTORY_PART1-3.md`,
`docs/P2P_M3_M9_AUDIT.md`, `docs/RULES.md`, `docs/TASKS.md`, `docs/analysis.md`.

## 3. Kanonische P2P-Dokumentationspfade

`docs/p2p/INVENTORY_PART1.md`, `docs/p2p/INVENTORY_PART2.md` und
`docs/p2p/INVENTORY_PART3.md` sind **byte-identische Duplikate** von
`docs/P2P_INVENTORY_PART1.md`, `docs/P2P_INVENTORY_PART2.md` und
`docs/P2P_INVENTORY_PART3.md`.

→ **Kanonisch ist `docs/P2P_INVENTORY_PART*.md`** (Cave-Linie, Stand 2026-08-16).
Die Kopien unter `docs/p2p/` stammen aus der Scaffold-Linie und wurden **nicht**
gelöscht, um Datenverlust zu vermeiden. Ihre Konsolidierung ist ein Folge-Task
und benötigt eine eigene Freigabe.

Weitere Pfaddopplungen mit **unterschiedlichem** Inhalt (beide Varianten erhalten,
siehe `docs/plans/` vs. `docs/p2p/`):
`docs/plans/cave-p2p-implementation-plan.md` ↔ `docs/p2p/CAVE_P2P_IMPLEMENTATION_PLAN.md`,
`docs/p2p/TASKS.md` ↔ `docs/TASKS.md`, `docs/p2p/analysis.md` ↔ `docs/analysis.md`.

## 4. Status der Komponenten

- **Build:** `cargo check --workspace --locked --offline` = Exit 0 auf dem Union-Baum.
- **cave-p2p:** implementiert (Cave-Version), Audit unter `docs/P2P_M3_M9_AUDIT.md`.
- **references/:** reine Wissensquelle gemäß `todo.md` und `docs/RULES.md` — die
  Referenzprojekte `virtme`, `ugrd`, `osbuild`, `bkg-p2p` werden portiert, nicht
  umfunktioniert.

## 5. Offene Punkte (bewusst nicht Teil der Wiederherstellung)

1. **Doktrin-Konflikt:** `todo.md` / `docs/RULES.md` (Referenzprojekte ausschließlich
   als Wissensquelle) vs. `ROADMAP.md` / `docs/cave/*` (cave-p2p als Plugin-Port).
   Beide Lesarten liegen jetzt im Baum. Klärung ist ein Folge-Task.
2. **M-Nummern-Kollision:** `ROADMAP.md` M3–M9 = surge/kindle/forge,
   `docs/P2P_M3_M9_AUDIT.md` M3–M9 = cave-p2p-Meilensteine.
3. **`README.md` / `ROADMAP.md`** beschreiben den Stand 2026-07-29 („cave-p2p *Planned*")
   und wurden historisch nicht umgeschrieben.
4. **P2P-Laufzeitverhalten** (Swarm, A2A, Identität) ist durch `cargo check` **nicht**
   abgedeckt — `cargo check` belegt Kompilierbarkeit, nicht Laufzeitkorrektheit.

## 6. Remote-Situation

| Repository | Remote | Status |
|---|---|---|
| `/srv/bkg/privat/dump/bkg-extesions/bkg-cave` | `bkg-zig/bkg-cave` | Arbeits-Remote, empfängt den Restore per Fast-Forward |
| `/srv/bkg/bkg-cave` | `1bitshit/bkg-cave` | **nicht erreichbar** (`Repository not found`, Token ungültig) — bleibt lokaler, eingefrorener Provenienz-Mirror auf `cd977a0` |

Original-Historie ist über die Branches `archive/cave-line`, `archive/scaffold-main`
und `archive/drafts-base` referenziert; Bundles liegen unter
`/srv/bkg/privat/dump/backup/bkg-cave-restore-20261101/`.
