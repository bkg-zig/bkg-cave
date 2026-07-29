# OpenCode Agent Rules

## Arbeitsbereich

Dieser Agent arbeitet ausschließlich innerhalb dieses Verzeichnisses:

/home/workspace/work/alpha/bkg-cave/docs/agents/opencode/code

---

## Erlaubte Aktionen

Der OpenCode Agent darf:

* Dateien innerhalb dieses Ordners lesen
* Dateien innerhalb dieses Ordners erstellen
* Dateien innerhalb dieses Ordners ändern
* Dokumentationen, Pläne und Code-Artefakte innerhalb dieses Ordners pflegen

---

## Harte Grenze

Der OpenCode Agent darf niemals außerhalb dieses Ordners Änderungen durchführen.

Verboten:

* Änderungen an `/home/workspace/work/alpha/bkg-cave/crates/`
* Änderungen an `/home/workspace/work/alpha/bkg-cave/src/`
* Änderungen an Cargo-Dateien außerhalb dieses Ordners
* Änderungen an Projektkonfigurationen außerhalb dieses Ordners
* Änderungen an anderen Agenten-Verzeichnissen

---

## Lesen außerhalb des Arbeitsbereichs

Lesen außerhalb dieses Ordners ist nur erlaubt, wenn es für Analyse oder Verständnis notwendig ist.

Änderungen außerhalb bleiben verboten.

---

## Architekturregeln

Der OpenCode Agent:

* erfindet keine neue Architektur
* ändert keine Projektentscheidungen eigenständig
* ersetzt keine bestehenden Pläne ohne Freigabe
* dokumentiert Erkenntnisse statt sie ungefragt umzusetzen

---

## Aufgabenbereich

OpenCode dient für:

* Analyse
* Planung
* technische Dokumentation
* vorbereitende Code-Arbeiten innerhalb dieses Ordners
* Architekturvergleich
* Referenzanalyse

---

## Sicherheit

Bei Unsicherheit:

* keine Änderung außerhalb des Ordners durchführen
* Änderung dokumentieren
* Freigabe abwarten

Der Arbeitsbereich ist eine geschützte Zone.
