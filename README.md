# Billflux – validierter ZUGFeRD-Prototyp

## Grafische Oberfläche

Die Tauri-Oberfläche liegt in `desktop/`. Die Seitenleiste bietet eine neue Rechnung, die letzten fünf Rechnungen, Suche nach Nummer oder Betreff, Kunden, Templates und Einstellungen. Unten wird die aktuelle eigene Firma gewechselt. Jeder Firma ist eine Rechnungsvorlage zugeordnet; beim Anlegen einer weiteren Firma wird `templates/standard/` in `templates/firma-ID/` kopiert. Die endgültige PDF verwendet die zugeordnete Vorlage. Rechts wird eine HTML-Vorschau mit fester A4-Breite gezeigt. Die Beträge werden in Rust centgenau und je Umsatzsteuersatz berechnet. Entwürfe werden während der Eingabe automatisch in `database/billflux.sqlite` gespeichert. Kunden können aus der Rechnungsmaske gespeichert und für neue Rechnungen wiederverwendet werden.

Unter **Einstellungen** werden pro eigener Firma Kontoinhaber, IBAN, BIC, Bank und das Zahlungsziel gespeichert. Neue Rechnungen übernehmen die Zahlungsdaten; das Zahlungsfeld ist dann zunächst eingeklappt. Leistungsdatum und Fälligkeitsdatum sind in der Maske optional. Ohne Fälligkeitsdatum berechnet der Export die Frist aus dem Rechnungsdatum und dem eingestellten Zahlungsziel (anfangs 14 Tage). Lange Positionstexte umbrechen; das PDF kann dadurch mehrere A4-Seiten umfassen.

Beim ersten Start mit der neuen Version wird die bisherige Einzelentwurf-Datenbank übernommen. Zuvor wird `database/billflux.v1-backup.sqlite` angelegt. Rechnungsnummern sind über alle eigenen Firmen hinweg eindeutig; eine bereits ausgestellte Rechnung ist schreibgeschützt. Vorhandene PDFs im Ausgabeordner werden bei der Nummernprüfung berücksichtigt.

```powershell
cargo run --offline --manifest-path desktop/Cargo.toml
```

Die Schaltfläche **ZUGFeRD-PDF erstellen** speichert den Entwurf, erzeugt aus den eingegebenen Daten die endgültige PDF und CII-XML, bettet die XML ein und validiert das Ergebnis. Nur bei erfolgreicher Validierung landen PDF und Prüfbericht in `output/`. Der Export weist auf fehlende Pflichtangaben hin und überschreibt keine vorhandene Rechnungsnummer. Eine USt-IdNr. braucht ein Länderkürzel wie `DE`; wer nur eine Steuernummer hat, lässt das USt-IdNr.-Feld leer. Die HTML-Vorschau ist eine editiernahe Ansicht im selben Stil, aber noch nicht die endgültige PDF-Vorlage. Für den Export werden weiterhin die gebündelten Werkzeuge einschließlich Chrome benötigt. 0 % Umsatzsteuer wird derzeit zurückgewiesen, weil die steuerliche Kategorie und der Befreiungsgrund im XML noch nicht abgebildet sind.

Der Prototyp erzeugt aus einem gemeinsamen Rust-Rechnungsmodell eine gestaltete A4-PDF und eine CII-Rechnungs-XML. Ghostscript erstellt PDF/A-3, Mustang bettet die XML ein und validiert die Hybrid-PDF. Ein fehlgeschlagener Prüfbericht verhindert die Übernahme nach `output/`.

## Start

```powershell
cargo test --offline
cargo run --offline -- --zugferd
```

`--pdf` erstellt nur die Layout-Vorschau. `--zugferd` erstellt zusätzlich [die geprüfte Beispiel-PDF](output/PROTOTYP-BF-2026-0001.pdf) und [den Prüfbericht](output/PROTOTYP-BF-2026-0001.validation.xml). Die Dateien in `preview/` sind Zwischenstände. Für den ZUGFeRD-Lauf werden lokale Werkzeuge in `.tools/` benötigt; die genaue Einrichtung steht in [docs/zugferd-validierung.md](docs/zugferd-validierung.md).

## Portabler Windows-Ordner

Mit `./scripts/build-portable.ps1` wird `dist/Billflux/` gebaut. Der Build-Rechner benötigt die in `.tools/` vorbereiteten Werkzeuge und eine lokale Chrome-Installation; im fertigen Ordner liegen die Laufzeitdateien unter `vendor/`. Der frühere CLI-Bundle-Export wurde direkt aus diesem Ordner validiert. Der aktualisierte Bundle-Build mit Desktop-Oberfläche ist noch nicht separat auf einem frischen System getestet.

```text
Billflux/
├── Billflux.exe       (Desktop-Oberfläche)
├── Billflux-CLI.exe   (Beispielexport)
├── templates/standard/
│   ├── invoice.html
│   ├── style.css
│   └── fonts/
├── vendor/
│   ├── chrome/
│   ├── gs/
│   ├── jre11/
│   ├── Mustang-CLI-2.26.0.jar
│   ├── PDFA_def.ps
│   └── srgb.icc
├── database/
├── output/
└── preview/          (wird beim Start angelegt)
```

`database/` und `output/` werden nicht durch den Bundle-Build geleert. Die Vorlage und Schriften sind im Bundle bearbeitbar. `Billflux.exe` startet die Eingabemaske, `Billflux-CLI.exe` erzeugt die fest definierte Beispielrechnung. Für eine spätere Weitergabe müssen die Lizenzen der mitgelieferten Werkzeuge geprüft werden. Die Lauffähigkeit auf einem frisch aufgesetzten Windows-System ist noch nicht getestet.

## Aufbau

- `src/model.rs`: Firmen-, Kunden- und Rechnungsdaten, Cent-genaue Beträge und Umsatzsteuer je Steuersatz.
- `src/view.rs`: HTML aus dem finalisierten Rechnungsmodell.
- `src/xml.rs`: CII-XML aus demselben Modell.
- `src/export.rs`: PDF/A-3-Erstellung, XML-Einbettung, Validierung und Freigabe des Prototyp-Exports.
- `src/main.rs`: Ablaufsteuerung und Pfade relativ zum EXE-Verzeichnis.
- `templates/standard/`: frei bearbeitbare HTML/CSS-Vorlage; die Schriften liegen in `fonts/` mit OFL-Lizenzdateien.

Die Gestaltung orientiert sich an der bereitgestellten Rechnung. Der Markenname **artMessengers.de** steht im Beispiel; Anschrift, Steuerkennzeichen, Kunde und Bankdaten sind Platzhalter. Die IBAN hat gültige Prüfziffern, gehört aber zu keiner realen Bankverbindung. Die Rechnung darf nicht versendet werden.

Die Beispielrechnung ergibt 517,00 € netto, 98,23 € Umsatzsteuer und 615,23 € brutto. Die vier verwendeten Fira-Schriften sind im finalen PDF eingebettet. Die aktuelle Ausgabe umfasst eine A4-Seite.

## Status und nächste Schritte

Der konkrete Beispielexport und ein vollständiger Export aus einem ausgefüllten Desktop-Entwurf wurden mit Mustang 2.26.0 als PDF/A-3u und EN-16931-XML **gültig** geprüft. Weitere Rechnungsfälle, besonders 0 % Umsatzsteuer, sowie unveränderliche Archivierung und die Klärung der Laufzeit- und Lizenzverteilung sind noch offen.
