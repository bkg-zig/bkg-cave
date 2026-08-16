# bkg-cave Agent Rules

## Gültigkeitsbereich

Diese Datei definiert die verbindlichen Regeln für alle Coding-Agenten innerhalb des Projekts bkg-cave.

Projektpfad:

/home/workspace/work/alpha/bkg-cave

---

# Grundregel

Jeder Agent darf ausschließlich innerhalb seines zugewiesenen Arbeitsverzeichnisses arbeiten.

Ein Agent darf niemals Dateien außerhalb seines Arbeitsbereichs verändern.

---

# Agenten-Arbeitsbereiche

## OpenCode Agent

Arbeitsbereich:

/home/workspace/work/alpha/bkg-cave/docs/agents/opencode/code

Erlaubt:

* Dateien lesen
* Dateien erstellen
* Dateien ändern
* Tests und Hilfsskripte innerhalb dieses Ordners ausführen

Nicht erlaubt:

* Änderungen außerhalb dieses Ordners
* Änderungen an crates/
* Änderungen an src/
* Änderungen an Cargo-Dateien
* Änderungen an Projektkonfigurationen außerhalb dieses Ordners

---

## Kilo Agent

Arbeitsbereich:

/home/workspace/work/alpha/bkg-cave/docs/agents/kilo/code

Erlaubt:

* Dateien lesen
* Dateien erstellen
* Dateien ändern
* Tests und Hilfsskripte innerhalb dieses Ordners ausführen

Nicht erlaubt:

* Änderungen außerhalb dieses Ordners
* Änderungen an crates/
* Änderungen an src/
* Änderungen an Cargo-Dateien
* Änderungen an Projektkonfigurationen außerhalb dieses Ordners

---

# Verbotene Aktionen

Agenten dürfen nicht:

* nach oben aus ihrem Arbeitsverzeichnis wechseln und dort Dateien bearbeiten
* Projektdateien außerhalb ihres Ordners erzeugen
* bestehende Architekturentscheidungen selbstständig ändern
* neue Arbeitsbereiche anlegen
* andere Agenten-Verzeichnisse verändern

---

# Architekturänderungen

Architekturentscheidungen erfolgen ausschließlich über die Projektplanung.

Coding-Agenten setzen nur freigegebene Aufgaben um.

Keine eigenständigen Umstrukturierungen.

---

# Änderungen außerhalb des Bereichs

Falls eine Aufgabe Änderungen außerhalb des eigenen Arbeitsbereichs benötigt:

1. Änderung dokumentieren
2. Grund beschreiben
3. Freigabe einholen
4. Erst danach ändern

---

# Ziel

bkg-cave bleibt kontrolliert, nachvollziehbar und reproduzierbar.

Jeder Agent besitzt einen klar abgegrenzten Arbeitsbereich.
Keine stillen Änderungen außerhalb der vorgesehenen Zone.
