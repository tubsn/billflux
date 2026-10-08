# ZUGFeRD-Validierung in Billflux

Das Zielprofil ist **EN 16931**. FeRD führt seit September 2026 [ZUGFeRD 2.5.2 als aktuelles Infopaket](https://www.ferd-net.de/standards/zugferd). Mustang 2.26.0 nutzt für den Export die passenden ZUGFeRD-2.5-Regeln.

## Lokale Werkzeuge

Beim Entwickeln liegen die Werkzeuge unter `.tools/` und sind nicht Teil des Quellcodes. Das Build-Skript kopiert die benötigten Laufzeitdateien in den portablen `bin/`-Ordner neben der EXE:

- Portable [Temurin-JRE 11](https://api.adoptium.net/v3/binary/latest/11/ga/windows/x64/jre/hotspot/normal/eclipse) unter `.tools/jre11/<jre-verzeichnis>/bin/java.exe`.
- [Mustang CLI 2.26.0](https://repo1.maven.org/maven2/org/mustangproject/Mustang-CLI/2.26.0/Mustang-CLI-2.26.0.jar) als `.tools/Mustang-CLI-2.26.0.jar`.
- [Ghostscript 10.08.0](https://ghostscript.com/releases/) unter `.tools/gs/Library/bin/gswin64c.exe`. Die aktuelle lokale Testumgebung verwendet das Win-64-Paket von conda-forge.
- Ghostscripts [PDF/A-Definitionsdatei](https://github.com/ArtifexSoftware/ghostpdl/blob/master/lib/PDFA_def.ps) als `.tools/PDFA_def.ps` und ein RGB-ICC-Profil als `.tools/srgb.icc`. Im lokalen Test wurde das Windows-sRGB-Profil verwendet.
- [Chrome Headless Shell](https://developer.chrome.com/docs/automation-and-testing/headless-chrome-shell) für HTML zu PDF. `scripts/install-renderer.ps1` lädt die feste Version 154.0.8037.57 aus dem offiziellen Chrome-for-Testing-Archiv und prüft ihren SHA-256-Hash. Der portable Build legt sie unter `bin/chrome-headless-shell/` ab; eine lokale Chrome-Installation wird nicht benötigt.

Der Export in der Desktop-App erstellt zunächst PDF und XML unter `dist/Billflux/preview/`, wandelt die PDF mit Ghostscript in PDF/A-3 um, bettet die XML mit Mustang ein und validiert die fertige Datei. Nur wenn die drei Berichtswerte für PDF, XML und Gesamtergebnis jeweils `valid` sind, übernimmt Billflux PDF und Bericht nach `dist/Billflux/logs/`.

## Ergebnis des Beispieltests

Mustang 2.26.0 meldete für `PROTOTYP-BF-2026-0001.pdf`:

- PDF/A-3u: `isCompliant=true`, PDF-Status `valid`.
- CII XML, Profil `urn:cen.eu:en16931:2017`: XML-Status `valid`.
- Gesamtstatus `valid`.
- Sechs Hinweise aus zusätzlich ausgeführten XRechnung/Peppol-Regeln. Sie sind im Bericht dokumentiert und müssen bei einer späteren Unterstützung dieser Profile gesondert behandelt werden.

Zusätzlich wurde geprüft: Die PDF enthält eine XML-Datei namens `zugferd-invoice.xml`, deren Bytes mit `preview/sample.xml` übereinstimmen. Vier Fira-Fontstreams sind eingebettet und die PDF hat eine A4-Seite. Diese Prüfungen ersetzen keinen sachlichen Abgleich echter Rechnungsdaten.

Für eine unabhängige PDF/A-Gegenprüfung kann [veraPDF](https://docs.verapdf.org/cli/validation/) mit Profil `3u` auf die fertige PDF angewendet werden. Der bisherige PDF/A-Befund stammt aus dem in Mustang eingebundenen veraPDF-Prüfer.

## Grenzen

Die Beispielrechnung nutzt Platzhalter. Der Validierungserfolg bezieht sich genau auf diesen Export; er beweist nicht, dass beliebige Rechnungen korrekt werden. Vor produktiver Nutzung brauchen wir Tests für verschiedene Steuersätze, Rabatte, Gutschriften, lange Positionslisten, echte Bank- und Steuerdaten sowie einen automatisierten Inhaltsabgleich zwischen sichtbarer PDF und XML. Die Verteilung von Ghostscript ist auch lizenzrechtlich zu klären.

## Billflux-Metadaten

Nach der XML-Einbettung normalisiert `src/java/BillfluxPdfMetadata.java` die Softwareangaben mit dem in Mustang enthaltenen PDFBox: Die freiwilligen Angaben PDF-Producer, PDF-Creator, PDF-Author, XMP-Producer, XMP-CreatorTool und XMP-dc:creator werden entfernt. Es wird kein Herstellername eingetragen. Rechnungs-XML, PDF/A-/ZUGFeRD-Deklarationen, Titel und Zeitstempel bleiben erhalten. Erst diese finale Datei wird validiert und zur Ausgabe freigegeben.

Die Java-11-kompatible Klasse liegt mit ihrem Quellcode unter `src/java/` und wird über `include_bytes!` in Billflux eingebettet. Zur Laufzeit sind keine weiteren Werkzeuge nötig. Nach Änderungen am Java-Quellcode muss `scripts/build-pdf-metadata.ps1` mit einem JDK (javac, Java 11 oder neuer) ausgeführt werden, anschließend der Rust-Build. Die bestehende Mustang-JAR bleibt unverändert.

Der Regressionstest `scripts/test-pdf-metadata.py` prüft Softwareangaben, fehlende Autorenangaben, Formatdeklarationen und genau einen byteidentischen XML-Anhang. Der vollständige Export mit Mustang 2.26.0 wurde am 08.10.2026 erneut als PDF/A-3u und EN-16931-XML validiert; die gerenderte Seite ist pixelidentisch mit der PDF/A-Datei vor der Einbettung. Bereits ausgestellte Rechnungen werden nicht nachträglich verändert.


## Prüfung mit Rechnung 2026-42062 am 08.10.2026

Der vollständige Desktop-Export wurde mit einer isolierten Kopie der gespeicherten Rechnung und Datenbank ausgeführt (`production::tests::exports_invoice_fixture`, Umgebungsvariablen `BILLFLUX_TEST_BASE` und `BILLFLUX_TEST_INVOICE`). Die Originaldatenbank und die ausgestellte PDF werden dabei nicht verändert.

- Mustang 2.26.0 einschließlich eingebettetem veraPDF: PDF/A-3u, Rechnungs-XML und Gesamtergebnis jeweils `valid`.
- Offizielle CEN/TC 434 EN-16931-CII-Regeln, Release `validation-1.3.16`, separat mit Saxon ausgeführt: keine fehlgeschlagenen Assertions. Quelle: https://github.com/ConnectingEurope/eInvoicing-EN16931/releases/tag/validation-1.3.16
- pypdf: genau ein XML-Anhang, exakt gleiche Bytes wie die generierte CII-XML und wie vor der Änderung. Betrag 500,00 EUR netto, 95,00 EUR Umsatzsteuer, 595,00 EUR brutto.
- Ghostscript-Seitenrendering vor/nach der Änderung: pixelidentisch.
- Codemeta-Extraktions- und Auswertungscode lokal ohne Upload der Rechnung ausgeführt: erster Treffer ist wieder die korrekt als UTF-8 dekodierte Rechnungs-XML; Nummer, Datum, Typ, Verkäufer, Käufer, Position und Beträge erkannt.

Die vorige Metadaten-Nachbearbeitung hatte beim Speichern die PDFBox-Standardkomprimierung für PDF-Objekte aktiviert. Codemeta sucht teils nur nach Dateinamen und XML-Textmustern, folgt den PDF-Referenzen nicht zuverlässig und verwechselt XMP-Namensräume mit Rechnungsdaten. Seine heuristische Suche erzeugt außerdem Duplikate und falsch dekodierte Treffer. Die neue Ausgabe verwendet wie Mustang `CompressParameters.NO_COMPRESSION` für die Objektstruktur. Dies behebt die Auswahl des ersten Treffers; fehlerhafte zusätzliche Treffer des Tools sind keine echten Anhänge. Quelle des untersuchten Codes: https://codemeta.de/zugferd/app.js

Die PDF/A-XMP-Erweiterungsschema-Beschreibungen (`pdfaProperty:description`) bleiben erforderlich. Sie beschreiben die Bedeutung der vier Rechnungs-Metadatenfelder; sie sind weder Beispieldaten noch XML-Dateianhänge. Siehe https://www.pdflib.com/pdf-knowledge-base/zugferd-and-factur-x/ und https://pdfa.org/resource/technical-note-tn-0009-xmp-extension-schemas-in-pdfa-1/.

Der Export prüft nun zusätzlich nach dem Speichern, dass genau ein Anhang vorhanden ist, `/AF` auf dieselbe Dateispezifikation zeigt und sowohl `/F` als auch `/UF` exakt die ursprünglichen XML-Bytes liefern. Erst danach erfolgt die Gesamtvalidierung. Vorhandene Hinweise aus zusätzlichen XRechnung/Peppol-Regeln bleiben im Bericht sichtbar; die Rechnung wird als EN-16931-Rechnung geprüft, nicht als XRechnung zertifiziert.