# Billflux

Billflux ist eine portable Windows-Anwendung für Rechnungen mit ZUGFeRD-PDF. Das Repository enthält ein einziges Rust-/Tauri-Paket. Die Anwendung liegt nach dem Build unter `dist/Billflux/`.

## Build

Aus PowerShell im Projektverzeichnis:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-portable.ps1
```

Das Skript baut nur `Billflux.exe`, kopiert die benötigten Laufzeitwerkzeuge nach `dist/Billflux/vendor/` und entfernt anschließend den Cargo-Build-Cache. Für schnellere Folgebuilds lässt sich der Cache mit `-KeepBuildCache` behalten. Der Build-Rechner benötigt die in `.tools/` vorbereiteten ZUGFeRD-Werkzeuge. Die feste Version von Chrome Headless Shell wird beim ersten Build über `scripts/install-renderer.ps1` heruntergeladen und per SHA-256 geprüft; eine lokale Chrome-Installation ist nicht nötig. Die Einrichtung steht in [docs/zugferd-validierung.md](docs/zugferd-validierung.md).

`dist/Billflux/database/`, `dist/Billflux/output/` und bereits vorhandene Vorlagen werden bei einem erneuten Build nicht überschrieben oder geleert. Die Datenbank in `dist/Billflux/database/billflux.sqlite` enthält die live genutzten Rechnungsdaten und sollte regelmäßig gesichert werden. Zum Weitergeben an andere Personen einen frischen portablen Ordner ohne eigene Datenbank und Exporte erstellen.

## Anwendung

`dist/Billflux/Billflux.exe` startet die Oberfläche. Ohne vorhandene Datenbank beginnt Billflux mit „Musterfirma“ und der Vorlage „Example“. Die bisherige Vorlage `templates/standard/` heißt in der Oberfläche „Artmessengers“. Vorlagen lassen sich auf der Template-Seite ansehen und für die aktuelle Firma aktivieren. Der PDF-Seitenumbruch wird aus `src/pagination.js` eingebunden; die Vorlagen selbst enthalten nur HTML und CSS.

Entwürfe werden automatisch gespeichert. Beim PDF-Export erzeugt Billflux eine gestaltete PDF und eine CII-Rechnungs-XML, bettet die XML ein und validiert das Ergebnis mit den gebündelten Werkzeugen. Die Laufzeitwerkzeuge unter `vendor/` sind Teil der portablen Distribution. Ihre Lizenzen und die Lauffähigkeit auf einem frisch aufgesetzten Windows-System sollten vor öffentlicher Weitergabe geprüft werden.

## Quellstruktur

- `src/`: Desktop-Anwendung, Rechnungsmodell, HTML-/XML-Erzeugung und PDF-Export.
- `ui/`: Tauri-Oberfläche.
- `templates/`: mitgelieferte HTML-/CSS-Rechnungsvorlagen.
- `fonts/`: Schriften und OFL-Lizenzdateien.
- `scripts/build-portable.ps1`: einziger Distributionsbuild.
