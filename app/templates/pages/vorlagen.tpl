<main>
<h1>Ai Buddy Installationstext</h1>

<pre>
anbei die AI Buddy Software inklusive unserer Promptdatenbank für den Ai Buddy mit GPT4-Turbo, Dall-E3 und Whisper.

https://www.artmessengers.de/aibuddy/aibuddy.zip
https://www.artmessengers.de/aibuddy/aibuddy-db.zip

Zur Installation:
Die Datei aibuddy.zip beinhaltet das komplette System (inklusive der Vendor Dateien).
Ihr könnt diesen Stand jederzeit mit dem GIT-Repository hier abgleichen: https://github.com/tubsn/gpt-buddy
Dort spiele ich in Zukunft auch Updates ein.

in der Zip Datei befindet sich eine .env Datei Dort werden die Zugangsdaten für die Datenbank sowie der ChatGPT API Key hinterlegt.
Außerdem könnt ihr hier IP Adressen hinterlegen, die keinen Login benötigen. z.B. eure VPN IPs.
Das erkläre ich aber gern nochmal in einem Video oder Telefon Gespräch.

Euer Webserver bzw. die Domain muss im Prinzip auf das "public" Verzeichnis zeigen.
Sofern ihr ein SSL Zertifikat für die Domain hinterlegt habt, könnt ihr In der .htaccess Datei (auch im Public) eine Einstellung aktivieren, sodass immer "https" erzwungen wird.

In der Datei aibuddy-db befindet sich die Prompt Datenbank, welche ihr in eine MySQL Datenbank importieren müsstet.
das geht z.B. über PHPMyAdmin, oder ein anderes gängiges Datenbanktool.

Wenn die Datenbank importiert ist. Könnt ihr das Programm wahrscheinlich bereits aufrufen.
Als Login habe ich euch folgendes angelegt:

Login: ai@buddy.de
Passwort: start123

Nach dem Login könnt ihr über die integrierte Nutzerverwaltung weitere Nutzer hinzufügen.
Danach können wir mit der Konfiguration der Promptkategorien bzw. der Hauptnavigation loslegen.
(Das funktioniert über die „config.php“ Datei im „app“ Ordner)

Falls ihr Probleme habt kann ich die Installation auch gern übernehmen. Dann benötige ich aber Zugriff auf den Server und die MySQL Datenbank.
Bei Fragen einfach kurz per Mail melden.
</pre>

<hr>

<h1>Ai Buddy Onboarding</h1>
<pre>
Hallo 

Ich habe vor kurzem bei Thomas Bertz Marketing in der Runde eine Präsentation über den Ai Buddy gehalten. Die Kollegen haben die Sitzung netterweise aufgezeichnet: https://www.youtube.com/watch?v=YUbsFFuPWdM

Alternativ können wir auch gern direkt ein Video Meeting machen. 
Bei mir ist z.B. morgen ab 11 alles frei. 

Zum System selbst:
Im vertreibe den AI Buddy als Komplettpaket das heißt Audio Transkribierung, Text- und Bildgenerierung und unsere aktuelle Promptdatenbank.
Künftige Aktualisierungen am Kern, veröffentliche ich über ein GitHUB Repository (https://github.com/tubsn/gpt-buddy)

Im System könnt ihr die Prompts über ein Optionsmenu frei einstellen und erweitern. Je nachdem, welche Arbeitsprozesse ihr bei euch im Haus künftig ausfindig macht. Ihr könnt eigene Farben hinterlegen und natürlich das Hauptmenu nach euren Wünschen konfigurieren.

Das Paket kostet 4250€.
Für die Umsetzung wird ein einfacher PHP Server mit einer MySQL Datenbank benötigt. Sowie ein ChatGPT API-Key.
Diesen könnt ihr hier erwerben. https://platform.openai.com

Bei der Einrichtung kann ich euch gern unterstützen und falls es möglich ist Zugriff auf den Server zu bekommen kann ich auch entsprechend Updates bei euch einspielen.

Es gibt mittlerweile auch einen Slack Workspace wo wir uns über Weiterentwicklung und Prompt Ideen austauschen.
https://join.slack.com/t/slack-t7o3912/shared_invite/zt-2892th3s0-MCO1NGj71NqtZzXOeXnriw
</pre>




</main>