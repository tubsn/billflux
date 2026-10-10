use crate::model::{money, FinalInvoice};

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn display_money(cents: i64) -> String {
    money(cents).replace('.', ",")
}
fn german_date(value:&str) -> String {
    chrono::NaiveDate::parse_from_str(value,"%Y-%m-%d").map(|date|date.format("%d.%m.%Y").to_string()).unwrap_or_else(|_|value.to_string())
}
fn service_period(value:&str,as_month:bool) -> String {
    use chrono::Datelike;
    if as_month {
        if let Ok(date)=chrono::NaiveDate::parse_from_str(value,"%Y-%m-%d") {
            let months=["Januar","Februar","März","April","Mai","Juni","Juli","August","September","Oktober","November","Dezember"];
            return format!("{} {}",months[date.month0() as usize],date.format("%Y"));
        }
    }
    german_date(value)
}

pub fn html(invoice: &FinalInvoice<'_>, template: &str) -> String {
    let rows = invoice.items.iter().enumerate().map(|(index, line)| {
        format!("<tr><td>{}</td><td class=\"item\" style=\"white-space:pre-wrap\"><strong>{}</strong><br>{}</td><td class=\"number\">{} %</td><td class=\"number\">{} €</td><td class=\"number\">{}</td><td class=\"number\">{} €</td></tr>",
            index + 1, escape(&line.item.description), escape(&line.item.detail), line.item.vat_percent,
            display_money(line.item.unit_price_cents), if line.item.unit == "Pauschal" { "—".into() } else { format!("{} {}", line.item.quantity.to_string().replace('.', ","), escape(crate::model::unit_label(&line.item.unit, line.item.quantity))) }, display_money(line.net_cents))
    }).collect::<Vec<_>>().join("\n");
    let tax_rows = invoice
        .tax_by_rate
        .iter()
        .map(|(rate, (_, amount))| {
            format!(
                "<div><span>{rate}% MwSt.:</span><span>{} €</span></div>",
                display_money(*amount)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let logo=if invoice.invoice.seller.name.trim().eq_ignore_ascii_case("artMessengers.de") {
        "<span class=\"wordmark-art\">art</span><span class=\"wordmark-rest\">Messengers.de</span>".into()
    }else{escape(&invoice.invoice.seller.name)};
    let bic_entry=if invoice.invoice.bic.trim().is_empty(){String::new()}else{format!("<dt>BIC:</dt><dd>{}</dd>",escape(&invoice.invoice.bic))};
    let bic_line=if invoice.invoice.bic.trim().is_empty(){String::new()}else{format!("<br>BIC: {}",escape(&invoice.invoice.bic))};
    let mut registration_line=String::new();
    if !invoice.invoice.seller.vat_id.trim().is_empty(){registration_line.push_str(&format!("<br>USt-IdNr.: {}",escape(&invoice.invoice.seller.vat_id)));}
    if !invoice.invoice.seller.economic_id.trim().is_empty(){registration_line.push_str(&format!("<br>W-IdNr: {}",escape(&invoice.invoice.seller.economic_id)));}
    let sender_name=if invoice.invoice.seller.alternative_name.trim().is_empty(){&invoice.invoice.seller.name}else{&invoice.invoice.seller.alternative_name};
    let locality=invoice.invoice.seller.city.split_once(' ').map(|(_,city)|city.trim()).filter(|city|!city.is_empty()).unwrap_or(&invoice.invoice.seller.city);
    let date_german=german_date(&invoice.invoice.date);
    let due_german=german_date(&invoice.invoice.due_date);
    let service_german=service_period(&invoice.invoice.service_date,invoice.invoice.service_month);
    let service_meta=if invoice.invoice.service_date.is_empty(){String::new()}else{format!("<br>Leistungszeitraum: {}",escape(&service_german))};
    let heading_start=[invoice.invoice.subject_prefix.trim(),if invoice.invoice.number_in_subject {invoice.invoice.number.trim()}else{""}].into_iter().filter(|part|!part.is_empty()).collect::<Vec<_>>().join(" ");
    let invoice_heading=match (heading_start.is_empty(),invoice.invoice.subject.trim().is_empty()) {
        (true,_)=>invoice.invoice.subject.trim().to_string(),
        (_,true)=>heading_start,
        _=>format!("{} - {}",heading_start,invoice.invoice.subject.trim()),
    };
    let country_prefix=match invoice.invoice.seller.country.trim().to_uppercase().as_str(){"DE"|"DEU"|"DEUTSCHLAND"=>"D".to_string(),country=>country.to_string()};
    let seller_postal_city=if country_prefix.is_empty()||invoice.invoice.seller.city.trim().is_empty(){invoice.invoice.seller.city.clone()}else{format!("{}-{}",country_prefix,invoice.invoice.seller.city.trim())};
    let html=template
        .replace("<footer><div>{{seller_name}}<br>{{seller_street}}<br>{{seller_city}}</div>","<footer><div>{{sender_name}}<br>{{seller_street}}<br>{{seller_postal_city}}</div>")
        .replace("<div>Inhaber: {{account_holder}}<br>Verwendung: {{payment_reference}}<br>IBAN: {{iban}}{{bic_line}}</div></footer>","<div>{{bank_name}}<br>IBAN: {{iban}}{{bic_line}}</div></footer>")
        .replace("{{bic_entry}}</dl>","</dl>")
        .replace("<div class=\"wordmark\"><span>art</span>Messengers.de</div>","<div class=\"wordmark\">{{logo}}</div>")
        .replace("<div class=\"wordmark\">{{seller_name}}</div>","<div class=\"wordmark\">{{logo}}</div>")
        .replace("{{logo}}",&logo)
        .replace("{{bic_entry}}",&bic_entry)
        .replace("{{bic_line}}",&bic_line)
        .replace("{{seller_registration_line}}",&registration_line)
        .replace("<p>{{payment_note}}</p>",&if invoice.invoice.payment_note.trim().is_empty(){String::new()}else{format!("<p>{}</p>",escape(&invoice.invoice.payment_note))})
        .replace("bis zum {{due_date}} ", &if invoice.invoice.show_due_date { format!("bis zum {} ",escape(&due_german)) } else { String::new() })
        .replace("<dt>Verwendung:</dt><dd>{{number}}</dd>","<dt>Verwendung:</dt><dd>{{payment_reference}}</dd>")
        .replace("{{payment_reference}}",&escape(&invoice.invoice.payment_reference))
        .replace("{{payment_due}}",&if invoice.invoice.show_due_date {format!(" bis zum {}",escape(&due_german))}else{String::new()})
        .replace("{{number}}", &escape(&invoice.invoice.number))
        .replace("{{date_german}}", &escape(&date_german))
        .replace("{{date}}", &escape(&invoice.invoice.date))
        .replace("{{service_date}}", &escape(&service_german))
        .replace("{{service_meta}}", &service_meta)
        .replace("{{service_line}}", "")
        .replace("{{due_date}}", &if invoice.invoice.show_due_date {escape(&due_german)}else{String::new()})
        .replace("{{invoice_heading}}", &escape(&invoice_heading))
        .replace("{{subject}}", &escape(&invoice.invoice.subject))
        .replace("{{sender_name}}", &escape(sender_name))
        .replace("{{seller_locality}}", &escape(locality))
        .replace("{{seller_name}}", &escape(&invoice.invoice.seller.name))
        .replace("{{seller_street}}", &escape(&invoice.invoice.seller.street))
        .replace("{{seller_city}}", &escape(&invoice.invoice.seller.city))
        .replace("{{seller_postal_city}}", &escape(&seller_postal_city))
        .replace("{{seller_email}}", &escape(&invoice.invoice.seller.email))
        .replace("{{seller_phone}}", &escape(&invoice.invoice.seller.phone))
        .replace(
            "{{seller_website}}",
            &escape(&invoice.invoice.seller.website),
        )
        .replace(
            "{{seller_tax_number}}",
            &escape(&invoice.invoice.seller.tax_number),
        )
        .replace("{{seller_vat_id}}", &escape(&invoice.invoice.seller.vat_id))
        .replace("{{buyer_name}}", &format!("{}{}{}",escape(&invoice.invoice.buyer.name),if invoice.invoice.buyer.contact.is_empty(){String::new()}else{format!("<br>{}",escape(&invoice.invoice.buyer.contact))},if invoice.invoice.buyer.additional_info.is_empty(){String::new()}else{format!("<br>{}",escape(&invoice.invoice.buyer.additional_info))}))
        .replace("{{buyer_street}}", &escape(&invoice.invoice.buyer.street))
        .replace("{{buyer_city}}", &escape(&invoice.invoice.buyer.city))
        .replace(
            "{{account_holder}}",
            &escape(&invoice.invoice.account_holder),
        )
        .replace("{{iban}}", &escape(&invoice.invoice.iban))
        .replace("{{bic}}", &escape(&invoice.invoice.bic))
        .replace("{{bank_name}}", &escape(&invoice.invoice.bank_name))
        .replace("{{rows}}", &rows)
        .replace("{{net}}", &display_money(invoice.net_cents))
        .replace("{{tax_rows}}", &tax_rows)
        .replace("{{gross}}", &display_money(invoice.gross_cents));
    html.replace("</body>", &format!("<script>{}</script></body>",include_str!("pagination.js")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn implicit_due_date_only_in_xml_and_payment_reference_in_both() {
        let mut invoice=crate::model::sample();
        invoice.show_due_date=false;
        invoice.payment_reference="AM 2026-42062".into();
        let finalized=invoice.finalize().unwrap();
        let template=include_str!("../tests/fixtures/standard/invoice.html");
        let html=super::html(&finalized,template);
        assert!(!html.contains("bis zum"));
        assert!(!html.contains(&invoice.due_date));
        assert!(html.contains("AM 2026-42062"));
        let xml=crate::xml::render(&finalized);
        assert!(xml.contains("20261019"));
        assert!(xml.contains("<ram:PaymentReference>AM 2026-42062</ram:PaymentReference>"));
        invoice.show_due_date=true;
        assert!(super::html(&invoice.finalize().unwrap(),template).contains("bis zum 19.10.2026"));
    }

    #[test]
    fn invoice_identity_and_optional_payment_details() {
        let mut invoice=crate::model::sample();
        invoice.payment_note="Danke für Ihren Auftrag.".into();
        let template=include_str!("../tests/fixtures/standard/invoice.html");
        let html=super::html(&invoice.finalize().unwrap(),template);
        assert!(html.contains("<div class=\"wordmark\"><span class=\"wordmark-art\">art</span><span class=\"wordmark-rest\">Messengers.de</span></div>"));
        assert!(html.contains("<small><i></i>artMessengers.de</small>"));
        assert!(html.contains("Danke für Ihren Auftrag."));
        assert!(html.contains("<br>BIC: "));
        assert!(!html.contains("<dt>BIC:</dt>"));
        invoice.bic.clear();
        let finalized=invoice.finalize().unwrap();
        assert!(!super::html(&finalized,template).contains("<dt>BIC:</dt>"));
        assert!(!crate::xml::render(&finalized).contains("<ram:BICID>"));
    }

    #[test]
    fn tax_ids_follow_tax_number_in_footer() {
        let mut invoice=crate::model::sample();
        invoice.seller.economic_id="DE123456789-00001".into();
        let template=include_str!("../tests/fixtures/standard/invoice.html");
        let html=super::html(&invoice.finalize().unwrap(),template);
        assert!(html.contains("W-IdNr: DE123456789-00001"));
        assert!(html.contains("Steuernummer: 00/000/00000<br>USt-IdNr.: DE123456789<br>W-IdNr: DE123456789-00001"));
        invoice.seller.economic_id.clear();
        let html=super::html(&invoice.finalize().unwrap(),template);
        assert!(html.contains("USt-IdNr.: DE123456789"));
    }

    #[test]
    fn sender_subject_and_german_city_date_are_rendered() {
        let mut invoice=crate::model::sample();
        invoice.seller.alternative_name="Artmessengers".into();
        invoice.seller.city="03042 Cottbus".into();
        invoice.date="2026-10-07".into();
        invoice.subject_prefix="Honorar".into();
        invoice.subject="Ai Buddy Krams".into();
        invoice.bank_name="Deutsche Kreditbank AG".into();
        let html=super::html(&invoice.finalize().unwrap(),include_str!("../tests/fixtures/standard/invoice.html"));
        assert!(html.contains("<strong>Artmessengers</strong>"));
        assert!(html.contains("<small><i></i>artMessengers.de</small>"));
        assert!(html.contains("Cottbus, den 07.10.2026"));
        assert!(html.contains("Honorar BF-2026-0001 - Ai Buddy Krams"));
        invoice.number_in_subject=false;
        let without_number=super::html(&invoice.finalize().unwrap(),include_str!("../templates/example/invoice.html"));
        assert!(without_number.contains("<h1>Honorar - Ai Buddy Krams</h1>"));
        assert!(without_number.contains("<strong>BF-2026-0001</strong>"));
        assert!(html.contains("D-03042 Cottbus"));
        assert!(html.contains("<footer><div>Artmessengers<br>"));
        assert!(html.contains("<div>Deutsche Kreditbank AG<br>IBAN: "));
        assert!(!html.contains("<div>Inhaber: "));
    }

    #[test]
    fn copied_company_template_uses_current_footer() {
        let invoice=crate::model::sample();
        let old=include_str!("../tests/fixtures/standard/invoice.html")
            .replace("<footer><div>{{sender_name}}<br>{{seller_street}}<br>{{seller_postal_city}}</div>","<footer><div>{{seller_name}}<br>{{seller_street}}<br>{{seller_city}}</div>")
            .replace("<div>{{bank_name}}<br>IBAN: {{iban}}{{bic_line}}</div></footer>","<div>Inhaber: {{account_holder}}<br>Verwendung: {{payment_reference}}<br>IBAN: {{iban}}{{bic_line}}</div></footer>")
            .replace("<dd>{{iban}}</dd></dl>","<dd>{{iban}}</dd>{{bic_entry}}</dl>");
        let html=super::html(&invoice.finalize().unwrap(),&old);
        assert!(html.contains("<footer><div>artMessengers.de<br>"));
        assert!(!html.contains("<div>Inhaber: "));
        assert!(!html.contains("<dt>BIC:</dt>"));
    }
    #[test]
    fn service_period_and_due_date_in_both_themes() {
        let mut invoice=crate::model::sample();
        invoice.service_date="2026-08-17".into();
        invoice.service_month=true;
        for template in [include_str!("../tests/fixtures/standard/invoice.html"),include_str!("../templates/example/invoice.html")] {
            let html=super::html(&invoice.finalize().unwrap(),template);
            assert!(html.contains("Leistungszeitraum: August 2026"));
            assert!(html.contains("bis zum 19.10.2026"));
            assert!(!html.contains("{{service_meta}}"));
        }
        invoice.service_month=false;
        let html=super::html(&invoice.finalize().unwrap(),include_str!("../tests/fixtures/standard/invoice.html"));
        assert!(html.contains("Leistungszeitraum: 17.08.2026"));
    }
    #[test]
    fn buyer_additional_info_follows_contact() {
        let mut invoice=crate::model::sample();
        invoice.buyer.contact="Buchhaltung".into();
        invoice.buyer.additional_info="Gebäude B, 2. Etage".into();
        for template in [include_str!("../tests/fixtures/standard/invoice.html"),include_str!("../templates/example/invoice.html")] {
            let html=super::html(&invoice.finalize().unwrap(),template);
            assert!(html.contains("Beispielkunde GmbH<br>Buchhaltung<br>Gebäude B, 2. Etage"));
        }
        let xml=crate::xml::render(&invoice.finalize().unwrap());
        assert!(xml.contains("<ram:LineThree>Gebäude B, 2. Etage</ram:LineThree>"));
    }
}
