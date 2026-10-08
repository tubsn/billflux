use crate::{draft, export, model, store, view, xml};
use serde::Serialize;
use chrono::{Duration,NaiveDate};
use base64::Engine;
use std::{fs, path::{Path, PathBuf}, process::Command};

#[cfg(windows)]
fn hide_window(command:&mut Command){use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
#[cfg(not(windows))]
fn hide_window(_: &mut Command){}

#[derive(Serialize)]
pub struct ExportResult { pub pdf_path: String, pub report_path: String }

fn base_dir() -> Result<PathBuf, String> {
    #[cfg(test)]
    if let Ok(path) = std::env::var("BILLFLUX_TEST_BASE") { return Ok(PathBuf::from(path)); }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let folder = exe.parent().ok_or("Programmverzeichnis fehlt")?;
    if folder.join("templates").is_dir() { Ok(folder.to_path_buf()) }
    else { Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))) }
}

fn party(value: draft::Party) -> model::Party {
    model::Party { contact:value.contact, name:value.name, alternative_name:value.alternative_name, street:value.street, city:value.city, country:value.country, email:value.email, vat_id:value.vat_id, economic_id:value.economic_id, phone:value.phone, website:value.website, tax_number:value.tax_number }
}

fn invoice(value: draft::Workspace) -> model::Invoice {
    model::Invoice {
        show_due_date:!value.due_date.is_empty(),
        payment_note:value.payment_note.clone(),
        payment_reference:if value.payment_reference.trim().is_empty(){format!("{}{}",value.payment_reference_prefix,value.number)}else{value.payment_reference.clone()},
        number:value.number, date:value.date, service_date:value.service_date, due_date:value.due_date,
        subject:value.subject, subject_prefix:value.subject_prefix, account_holder:value.account_holder, iban:value.iban, bic:value.bic,
        bank_name:value.bank_name, seller:party(value.seller), buyer:party(value.buyer),
        items:value.items.into_iter().map(|item| {
            let unit_code = match item.unit.as_str() { "Stunden" | "Stunde" | "h" => "HUR", "Tage" | "Tag" => "DAY", _ => "C62" };
            model::Item { description:item.description, detail:item.detail, quantity:item.quantity,
                unit:item.unit, unit_code:unit_code.into(), unit_price_cents:item.unit_price_cents,
                vat_percent:item.vat_percent }
        }).collect(),
    }
}

fn validate(data: &draft::Workspace) -> Result<(), String> {
    for (label, value) in [
        ("Rechnungsnummer", &data.number), ("Rechnungsdatum", &data.date),
        ("Firmenname", &data.seller.name), ("Firmenanschrift", &data.seller.street),
        ("Firmenort", &data.seller.city),
        ("Kundenname", &data.buyer.name), ("Kundenanschrift", &data.buyer.street),
        ("Kundenort", &data.buyer.city), ("IBAN", &data.iban),
    ] { if value.trim().is_empty() { return Err(format!("{label} fehlt")); } }
    if data.seller.vat_id.trim().is_empty() && data.seller.tax_number.trim().is_empty() {
        return Err("USt-IdNr. oder Steuernummer fehlt".into());
    }
    if !data.seller.vat_id.trim().is_empty() && (data.seller.vat_id.chars().take(2).count()!=2 || !data.seller.vat_id.chars().take(2).all(|c|c.is_ascii_alphabetic())) {
        return Err("USt-IdNr. muss mit einem zweibuchstabigen Länderkürzel beginnen (z. B. DE). Wenn nur eine Steuernummer vorliegt, das USt-IdNr.-Feld bitte leer lassen.".into());
    }
    if data.items.is_empty() { return Err("Mindestens eine Position ist erforderlich".into()); }
    if !data.number.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') { return Err("Rechnungsnummer darf nur Buchstaben, Zahlen, - und _ enthalten".into()); }
    for (index, item) in data.items.iter().enumerate() {
        if item.description.trim().is_empty() { return Err(format!("Beschreibung für Position {} fehlt", index+1)); }
        if item.vat_percent == 0 { return Err("0 % Umsatzsteuer ist im ZUGFeRD-Export noch nicht abgebildet; bitte keinen anderen Satz als Ersatz wählen".into()); }
    }
    Ok(())
}

fn copy_assets(base: &Path, template_dir: &Path, preview: &Path) -> Result<(), String> {
    fs::copy(template_dir.join("style.css"), preview.join("style.css")).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(template_dir).map_err(|e| e.to_string())? {
        let entry=entry.map_err(|e| e.to_string())?;
        let path=entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("png")) {
            fs::copy(&path,preview.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    for (family, names) in [
        ("Fira_Sans", ["FiraSans-Regular.ttf", "FiraSans-Bold.ttf"]),
        ("Fira_Sans_Condensed", ["FiraSansCondensed-Regular.ttf", "FiraSansCondensed-Bold.ttf"]),
    ] {
        let destination=preview.join("fonts").join(family);
        fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
        for name in names {
            let bundled=template_dir.join("fonts").join(family).join(name);
            let source=if bundled.is_file(){bundled}else{base.join("fonts").join(family).join(name)};
            fs::copy(source,destination.join(name)).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn create(data: draft::Workspace) -> Result<ExportResult, String> { create_to(data,None) }

pub fn validate_export(data:&draft::Workspace) -> Result<(),String> {
    validate(data)?;
    draft::calculate(data)?;
    for (label,date) in [("Rechnungsdatum",data.date.as_str()),("Leistungsdatum",data.service_date.as_str()),("Fälligkeitsdatum",data.due_date.as_str())] {
        if !date.is_empty(){NaiveDate::parse_from_str(date,"%Y-%m-%d").map_err(|_|format!("{label} ist ungültig"))?;}
    }
    let template=store::company(data.company_id)?.template;
    if !store::templates()?.contains(&template){return Err("Die Vorlage dieser Firma wurde nicht gefunden".into());}
    Ok(())
}

pub fn create_to(mut data: draft::Workspace, destination:Option<PathBuf>) -> Result<ExportResult, String> {
    validate_export(&data)?;
    let show_due_date=!data.due_date.is_empty();
    let issued=NaiveDate::parse_from_str(&data.date,"%Y-%m-%d").map_err(|_|"Rechnungsdatum ist ungültig")?;
    if !data.service_date.is_empty(){NaiveDate::parse_from_str(&data.service_date,"%Y-%m-%d").map_err(|_|"Leistungsdatum ist ungültig")?;}
    if data.due_date.is_empty(){data.due_date=(issued+Duration::days(i64::from(store::settings(data.company_id)?.due_days))).format("%Y-%m-%d").to_string();}
    else{NaiveDate::parse_from_str(&data.due_date,"%Y-%m-%d").map_err(|_|"Fälligkeitsdatum ist ungültig")?;}
    validate(&data)?;
    let base=base_dir()?;
    let template_name=store::company(data.company_id)?.template;
    if !store::templates()?.contains(&template_name){return Err("Die Vorlage dieser Firma wurde nicht gefunden".into());}
    let template_dir=base.join("templates").join(&template_name);
    let output=base.join("logs");
    fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let pdf=destination.clone().unwrap_or_else(||output.join(format!("{}.pdf",data.number)));
    let report=output.join(format!("{}.validation.xml",data.number));
    if destination.is_none() && (pdf.exists() || report.exists()) { return Err("Zu dieser Rechnungsnummer existiert bereits ein Export. Bitte eine neue Nummer vergeben.".into()); }
    let preview=base.join("preview");
    fs::create_dir_all(&preview).map_err(|e| e.to_string())?;
    let mut invoice=invoice(data);
    invoice.show_due_date=show_due_date;
    let finalized=invoice.finalize()?;
    let template=fs::read_to_string(template_dir.join("invoice.html")).map_err(|e| e.to_string())?;
    let html_path=preview.join("current.html");
    let xml_path=preview.join("current.xml");
    let pdf_path=preview.join("current.pdf");
    fs::write(&html_path,view::html(&finalized,&template)).map_err(|e| e.to_string())?;
    fs::write(&xml_path,xml::render(&finalized)).map_err(|e| e.to_string())?;
    copy_assets(&base,&template_dir,&preview)?;
    let bundled=base.join("bin/chrome-headless-shell/chrome-headless-shell.exe");
    let local=base.join(".tools/chrome-headless-shell/chrome-headless-shell-win64/chrome-headless-shell.exe");
    let chrome=if bundled.is_file(){bundled}else{local};
    if !chrome.is_file() { return Err("Chrome Headless Shell für die PDF-Erstellung fehlt".into()); }
    if pdf_path.exists(){fs::remove_file(&pdf_path).map_err(|e| e.to_string())?;}
    let url=format!("file:///{}",html_path.to_string_lossy().replace('\\',"/").replace(' ',"%20"));
    let run_id=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?.as_nanos();
    let chrome_profile=std::env::temp_dir().join(format!("billflux-chrome-{}-{run_id}",std::process::id()));
    let rendered=crate::renderer::prepare(&chrome,&html_path,&chrome_profile,&run_id.to_string())?;
    fs::write(&html_path,rendered).map_err(|e|e.to_string())?;
    let mut chrome_command=Command::new(chrome);
    chrome_command.args(["--headless","--disable-gpu","--no-sandbox","--no-first-run","--no-pdf-header-footer"])
        .arg(format!("--user-data-dir={}-print",chrome_profile.display()))
        .arg(format!("--print-to-pdf={}",pdf_path.display())).arg(url);
    hide_window(&mut chrome_command);
    let result=chrome_command.output().map_err(|e| e.to_string())?;
    // Unter Windows kann der Chrome-Starter bereits beendet sein, während ein Kindprozess
    // die PDF noch schreibt. Deshalb nach Exit 0 kurz auf die fertige Datei warten.
    let mut pdf_ready=false;
    if result.status.success() {
        for _ in 0..150 {
            pdf_ready=fs::metadata(&pdf_path).is_ok_and(|metadata|metadata.len()>0);
            if pdf_ready { break; }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    if !result.status.success() || !pdf_ready {
        return Err(format!("PDF-Erstellung fehlgeschlagen (Chrome-Exit: {}, PDF vorhanden: {}). {} {}",
            result.status, pdf_ready, String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr)));
    }
    export::create_to(&base,&preview,&pdf_path,&xml_path,&invoice.number,Some(&pdf)).map_err(|e| e.to_string())?;
    Ok(ExportResult { pdf_path:pdf.display().to_string(), report_path:report.display().to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_are_rejected_before_export_setup() {
        let data=draft::Workspace::default();
        assert_eq!(validate_export(&data).unwrap_err(),"Rechnungsdatum fehlt");
    }

    #[test]
    #[ignore = "benötigt isolierte BILLFLUX_TEST_BASE und BILLFLUX_TEST_INVOICE JSON-Datei"]
    fn exports_invoice_fixture() {
        let source = std::env::var("BILLFLUX_TEST_INVOICE").unwrap();
        let workspace: draft::Workspace = serde_json::from_str(&fs::read_to_string(source).unwrap()).unwrap();
        let destination = base_dir().unwrap().join("logs/verified.pdf");
        let result = create_to(workspace, Some(destination)).unwrap();
        assert!(Path::new(&result.pdf_path).is_file());
        assert!(Path::new(&result.report_path).is_file());
    }

    #[test]
    #[ignore = "benötigt lokale PDF-Werkzeuge"]
    fn exports_and_validates_filled_draft() {
        let sample=model::sample();
        let mut data=draft::Workspace::default();
        data.company_id=store::load_app().unwrap().workspace.company_id;
        data.number=format!("TEST-{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        data.date=sample.date; data.service_date=sample.service_date; data.due_date=sample.due_date;
        data.subject=sample.subject; data.account_holder=sample.account_holder;
        data.iban=sample.iban; data.bic=sample.bic; data.bank_name=sample.bank_name;
        data.seller=draft::Party { contact:sample.seller.contact, name:sample.seller.name,street:sample.seller.street,city:sample.seller.city,country:sample.seller.country,email:sample.seller.email,phone:sample.seller.phone,website:sample.seller.website,tax_number:sample.seller.tax_number,vat_id:sample.seller.vat_id,economic_id:sample.seller.economic_id,alternative_name:sample.seller.alternative_name };
        data.buyer=draft::Party { contact:sample.buyer.contact, name:sample.buyer.name,street:sample.buyer.street,city:sample.buyer.city,country:sample.buyer.country,email:sample.buyer.email,phone:sample.buyer.phone,website:sample.buyer.website,tax_number:sample.buyer.tax_number,vat_id:sample.buyer.vat_id,economic_id:sample.buyer.economic_id,alternative_name:sample.buyer.alternative_name };
        data.items=sample.items.into_iter().map(|item| draft::Item { description:item.description,detail:item.detail,quantity:item.quantity,unit:"Stunden".into(),unit_price_cents:item.unit_price_cents,vat_percent:item.vat_percent }).collect();
        data.items[0].quantity=1.5;
        data.due_date.clear();data.payment_reference_prefix="AM".into();data.payment_reference="Auftrag 42".into();data.buyer.contact="Abteilung Einkauf".into();
        let destination=std::env::temp_dir().join(format!("{} custom.pdf",data.number));
        let result=create_to(data.clone(),Some(destination.clone())).unwrap();
        data.subject="Überarbeitete Rechnung".into();
        let repeated=create_to(data,Some(destination.clone())).unwrap();
        assert_eq!(repeated.pdf_path,result.pdf_path);
        assert_eq!(PathBuf::from(&result.pdf_path),destination);
        assert!(Path::new(&result.pdf_path).is_file());
        assert!(Path::new(&result.report_path).is_file());
        fs::remove_file(result.pdf_path).unwrap();
        fs::remove_file(result.report_path).unwrap();
    }

    #[test]
    #[ignore = "benötigt lokale PDF-Werkzeuge und pdfinfo"]
    fn optional_dates_and_long_positions_paginate() {
        let sample=model::sample();
        let mut data=draft::Workspace::default();
        data.company_id=store::load_app().unwrap().workspace.company_id;
        data.number=format!("TEST-LONG-{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        data.date=sample.date;
        data.subject="Umfangreiche Leistungen".into();
        data.account_holder=sample.account_holder;data.iban=sample.iban;data.bic=sample.bic;
        data.seller=draft::Party{contact:sample.seller.contact,name:sample.seller.name,street:sample.seller.street,city:sample.seller.city,country:sample.seller.country,email:sample.seller.email,phone:sample.seller.phone,website:sample.seller.website,tax_number:sample.seller.tax_number,vat_id:sample.seller.vat_id,economic_id:sample.seller.economic_id,alternative_name:sample.seller.alternative_name};
        data.buyer=draft::Party{contact:sample.buyer.contact,name:sample.buyer.name,street:sample.buyer.street,city:sample.buyer.city,country:sample.buyer.country,email:sample.buyer.email,phone:sample.buyer.phone,website:sample.buyer.website,tax_number:sample.buyer.tax_number,vat_id:sample.buyer.vat_id,economic_id:sample.buyer.economic_id,alternative_name:sample.buyer.alternative_name};
        data.items=(0..15).map(|n|draft::Item{description:format!("Leistungsposition {} mit einer langen Beschreibung und mehreren Details",n+1),detail:"Ausarbeitung, Abstimmung und Dokumentation der vereinbarten Leistungen. ".repeat(7),quantity:1.0,unit:"Stunden".into(),unit_price_cents:8500,vat_percent:19}).collect();
        let result=create(data).unwrap();
        let info=Command::new("pdfinfo").arg(&result.pdf_path).output().unwrap();
        assert!(info.status.success());
        let output=String::from_utf8_lossy(&info.stdout);
        let pages:usize=output.lines().find_map(|line|line.strip_prefix("Pages:")).unwrap().trim().parse().unwrap();
        assert!(pages>1,"lange Positionen sollen auf mehrere Seiten fließen");
        let tools=if base_dir().unwrap().join("bin/gs").is_dir(){base_dir().unwrap().join("bin")}else{base_dir().unwrap().join(".tools")};
        let mut last_page=Command::new(tools.join("gs/Library/bin/gswin64c.exe"));
        last_page.args(["-q","-dNOPAUSE","-dBATCH","-sDEVICE=txtwrite","-sOutputFile=-"])
            .arg(format!("-dFirstPage={pages}")).arg(format!("-dLastPage={pages}")).arg(&result.pdf_path);
        hide_window(&mut last_page);
        let text=String::from_utf8_lossy(&last_page.output().unwrap().stdout).into_owned();
        assert!(text.contains("Bitte überweisen Sie den Betrag"));
        assert!(text.contains("Ich bedanke mich") && text.contains("Zusammenarbeit"));
        assert!(text.contains("Steuernummer"));
        let mut previous_page=Command::new(tools.join("gs/Library/bin/gswin64c.exe"));
        previous_page.args(["-q","-dNOPAUSE","-dBATCH","-sDEVICE=txtwrite","-sOutputFile=-"])
            .arg(format!("-dFirstPage={}",pages-1)).arg(format!("-dLastPage={}",pages-1)).arg(&result.pdf_path);
        hide_window(&mut previous_page);
        let previous=String::from_utf8_lossy(&previous_page.output().unwrap().stdout).into_owned();
        assert!(!previous.contains("Bitte überweisen Sie den Betrag"));
        fs::remove_file(result.pdf_path).unwrap();fs::remove_file(result.report_path).unwrap();
    }

    #[test]
    #[ignore = "benötigt lokale PDF-Werkzeuge und pdfinfo"]
    fn oversized_position_continues_before_payment() {
        let sample=model::sample();
        let mut data=draft::Workspace::default();
        data.company_id=store::load_app().unwrap().workspace.company_id;
        data.number=format!("TEST-OVERSIZE-{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        data.date=sample.date;data.subject=sample.subject;
        data.account_holder=sample.account_holder;data.iban=sample.iban;data.bic=sample.bic;
        data.seller=draft::Party{contact:sample.seller.contact,name:sample.seller.name,street:sample.seller.street,city:sample.seller.city,country:sample.seller.country,email:sample.seller.email,phone:sample.seller.phone,website:sample.seller.website,tax_number:sample.seller.tax_number,vat_id:sample.seller.vat_id,economic_id:sample.seller.economic_id,alternative_name:sample.seller.alternative_name};
        data.buyer=draft::Party{contact:sample.buyer.contact,name:sample.buyer.name,street:sample.buyer.street,city:sample.buyer.city,country:sample.buyer.country,email:sample.buyer.email,phone:sample.buyer.phone,website:sample.buyer.website,tax_number:sample.buyer.tax_number,vat_id:sample.buyer.vat_id,economic_id:sample.buyer.economic_id,alternative_name:sample.buyer.alternative_name};
        data.items=vec![draft::Item{description:"Lange Einzelposition".into(),detail:format!("{}ENDE DER POSITION", "Ausarbeitung und Dokumentation der Leistungen. ".repeat(100)),quantity:1.0,unit:"Stunden".into(),unit_price_cents:8500,vat_percent:19}];
        let result=create(data).unwrap();
        let info=Command::new("pdfinfo").arg(&result.pdf_path).output().unwrap();
        let pages:usize=String::from_utf8_lossy(&info.stdout).lines().find_map(|line|line.strip_prefix("Pages:")).unwrap().trim().parse().unwrap();
        assert!(pages>1);
        let tools=if base_dir().unwrap().join("bin/gs").is_dir(){base_dir().unwrap().join("bin")}else{base_dir().unwrap().join(".tools")};
        let mut last_page=Command::new(tools.join("gs/Library/bin/gswin64c.exe"));
        last_page.args(["-q","-dNOPAUSE","-dBATCH","-sDEVICE=txtwrite","-sOutputFile=-"])
            .arg(format!("-dFirstPage={pages}")).arg(format!("-dLastPage={pages}")).arg(&result.pdf_path);
        hide_window(&mut last_page);
        let text=String::from_utf8_lossy(&last_page.output().unwrap().stdout).into_owned();
        assert!(text.contains("Bitte überweisen Sie den Betrag"));
        assert!(text.contains("Steuernummer"));
        let mut detail_page=Command::new(tools.join("gs/Library/bin/gswin64c.exe"));
        detail_page.args(["-q","-dNOPAUSE","-dBATCH","-sDEVICE=txtwrite","-sOutputFile=-"])
            .arg(format!("-dFirstPage={}",pages-1)).arg(format!("-dLastPage={}",pages-1)).arg(&result.pdf_path);
        hide_window(&mut detail_page);
        let detail_text=String::from_utf8_lossy(&detail_page.output().unwrap().stdout).into_owned();
        let ending=format!("{detail_text} {text}").split_whitespace().collect::<Vec<_>>().join(" ");
        let position_end=ending.find("ENDE DER POSITION").expect("Textende der langen Position fehlt");
        let payment_start=ending.find("Bitte überweisen Sie den Betrag").unwrap();
        assert!(position_end<payment_start,"Positionsende muss vor dem Zahlungsblock stehen");
        assert!(!detail_text.contains("Bitte überweisen Sie den Betrag"));
        fs::remove_file(result.pdf_path).unwrap();fs::remove_file(result.report_path).unwrap();
    }

    #[test]
    #[ignore = "benötigt BILLFLUX_EXPORT_TEST_DB und lokale PDF-Werkzeuge"]
    fn exports_saved_draft() {
        let db=std::env::var("BILLFLUX_EXPORT_TEST_DB").expect("Testdatenbank fehlt");
        let conn=rusqlite::Connection::open(db).unwrap();
        let json: String=conn.query_row("SELECT data FROM workspace WHERE id=1",[],|row|row.get(0)).unwrap();
        let mut data: draft::Workspace=serde_json::from_str(&json).unwrap();
        data.number=format!("TEST-{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        assert!(validate(&data).is_err());
        data.seller.vat_id.clear();
        let result=create(data).unwrap();
        assert!(Path::new(&result.pdf_path).is_file());
        fs::remove_file(result.pdf_path).unwrap();
        fs::remove_file(result.report_path).unwrap();
    }
}


pub fn preview_document(data:draft::Workspace) -> Result<String,String> {
    let template=store::company(data.company_id)?.template;
    preview_template(data,template)
}

pub fn preview_template(data:draft::Workspace,template_name:String) -> Result<String,String> {
    let base=base_dir()?;
    if !store::templates()?.contains(&template_name){return Err("Vorlage nicht gefunden".into());}
    let template_dir=base.join("templates").join(&template_name);
    let invoice=invoice(data);
    let totals=invoice.finalize()?;
    let template=fs::read_to_string(template_dir.join("invoice.html")).map_err(|e|e.to_string())?;
    let css=fs::read_to_string(template_dir.join("style.css")).map_err(|e|e.to_string())?
        .replace("fonts/Fira_Sans/FiraSans", "fonts/FiraSans")
        .replace("fonts/Fira_Sans_Condensed/FiraSansCondensed", "fonts/FiraSansCondensed");
    let mut html=view::html(&totals,&template).replace(r#"<link rel="stylesheet" href="style.css">"#,&format!("<style>{css}</style>"));
    for entry in fs::read_dir(&template_dir).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;
        let path=entry.path();
        if path.is_file() && path.extension().is_some_and(|ext|ext.eq_ignore_ascii_case("png")) {
            let name=entry.file_name().to_string_lossy().into_owned();
            let encoded=base64::engine::general_purpose::STANDARD.encode(fs::read(&path).map_err(|e|e.to_string())?);
            html=html.replace(&format!("src=\"{name}\""),&format!("src=\"data:image/png;base64,{encoded}\""));
        }
    }
    Ok(html)
}
