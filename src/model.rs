use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Party {
    pub name: String,
    pub alternative_name: String,
    pub contact:String,
    pub street: String,
    pub city: String,
    pub country: String,
    pub email: String,
    pub vat_id: String,
    pub economic_id:String,
    pub phone: String,
    pub website: String,
    pub tax_number: String,
}

pub struct Item {
    pub description: String,
    pub detail: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_code: String,
    pub unit_price_cents: i64,
    pub vat_percent: u32,
}

pub struct Invoice {
    pub number: String,
    pub date: String,
    pub service_date: String,
    pub service_month: bool,
    pub due_date: String,
    pub show_due_date: bool,
    pub payment_reference: String,
    pub payment_note:String,
    pub subject: String,
    pub subject_prefix: String,
    pub account_holder: String,
    pub iban: String,
    pub bic: String,
    pub bank_name: String,
    pub seller: Party,
    pub buyer: Party,
    pub items: Vec<Item>,
}

pub struct CalculatedItem<'a> {
    pub item: &'a Item,
    pub net_cents: i64,
}

pub struct FinalInvoice<'a> {
    pub invoice: &'a Invoice,
    pub items: Vec<CalculatedItem<'a>>,
    pub net_cents: i64,
    pub tax_cents: i64,
    pub gross_cents: i64,
    pub tax_by_rate: BTreeMap<u32, (i64, i64)>,
}

impl Invoice {
    pub fn finalize(&self) -> Result<FinalInvoice<'_>, String> {
        if self.items.is_empty() || self.number.is_empty() {
            return Err("Rechnungsnummer und Positionen sind erforderlich".into());
        }
        let mut net = 0_i64;
        let mut tax_by_rate = BTreeMap::<u32, (i64, i64)>::new();
        let mut calculated = Vec::new();
        for item in &self.items {
            if !item.quantity.is_finite() || item.quantity <= 0.0 || item.unit_price_cents < 0 || item.vat_percent > 100 {
                return Err("Ungültige Rechnungsposition".into());
            }
            let line_net = line_total(item.quantity, item.unit_price_cents)?;
            net = net.checked_add(line_net).ok_or("Betragsüberlauf")?;
            let entry = tax_by_rate.entry(item.vat_percent).or_default();
            entry.0 = entry.0.checked_add(line_net).ok_or("Betragsüberlauf")?;
            calculated.push(CalculatedItem {
                item,
                net_cents: line_net,
            });
        }
        let mut tax = 0_i64;
        for (rate, (basis, amount)) in &mut tax_by_rate {
            // EN-16931-Steueraufschlüsselung: je Steuersatz auf die gruppierte Bemessungsgrundlage runden.
            *amount = basis
                .checked_mul(i64::from(*rate))
                .and_then(|value| value.checked_add(50))
                .ok_or("Betragsüberlauf")?
                / 100;
            tax = tax.checked_add(*amount).ok_or("Betragsüberlauf")?;
        }
        let gross = net.checked_add(tax).ok_or("Betragsüberlauf")?;
        Ok(FinalInvoice {
            invoice: self,
            items: calculated,
            net_cents: net,
            tax_cents: tax,
            gross_cents: gross,
            tax_by_rate,
        })
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn sample() -> Invoice {
    Invoice {
        number: "BF-2026-0001".into(),
        date: "2026-10-05".into(),
        service_date: "2026-10-05".into(),
        service_month: false,
        due_date: "2026-10-19".into(),
        show_due_date: true,
        payment_reference:"BF-2026-0001".into(),
        payment_note:"Ich bedanke mich für die Zusammenarbeit.".into(),
        subject: "Konzeption und Gestaltung".into(),
        subject_prefix: "Rechnung".into(),
        account_holder: "artMessengers.de".into(),
        iban: "DE36 0000 0000 0000 0000 00".into(),
        bic: "XXXXXXXXXXX".into(),
        bank_name: "Musterbank".into(),
        seller: Party {
            contact:String::new(),
            name: "artMessengers.de".into(),
            alternative_name: String::new(),
            street: "Musterstraße 12".into(),
            city: "10115 Berlin".into(),
            country: "DE".into(),
            email: "rechnung@example.invalid".into(),
            vat_id: "DE123456789".into(),
            economic_id: "".into(),
            phone: "+49 000 000000".into(),
            website: "artmessengers.de".into(),
            tax_number: "00/000/00000".into(),
        },
        buyer: Party {
            contact:String::new(),
            name: "Beispielkunde GmbH".into(),
            alternative_name: String::new(),
            street: "Beispielweg 5".into(),
            city: "20095 Hamburg".into(),
            country: "DE".into(),
            email: "buchhaltung@example.invalid".into(),
            vat_id: "DE987654321".into(),
            economic_id: "".into(),
            phone: "".into(),
            website: "".into(),
            tax_number: "".into(),
        },
        items: vec![
            Item {
                description: "Konzeption".into(),
                detail: "Planung und Beratung für das Projekt".into(),
                quantity: 2.0,
                unit: "h".into(),
                unit_code: "HUR".into(),
                unit_price_cents: 12500,
                vat_percent: 19,
            },
            Item {
                description: "Gestaltung".into(),
                detail: "Ausarbeitung der vereinbarten Entwürfe".into(),
                quantity: 3.0,
                unit: "h".into(),
                unit_code: "HUR".into(),
                unit_price_cents: 8900,
                vat_percent: 19,
            },
        ],
    }
}

pub fn money(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_quantities_round_to_cents() {
        assert_eq!(line_total(1.5, 8500).unwrap(),12750);
        assert_eq!(line_total(0.145, 100).unwrap(),15);
        assert_eq!(line_total(0.5, 3).unwrap(),2);
        assert!(line_total(0.0, 100).is_err());
        assert!(line_total(f64::NAN,100).is_err());
        assert!(line_total(f64::MAX,100).is_err());
    }
    #[test]
    fn sample_totals() {
        let invoice = sample();
        let result = invoice.finalize().unwrap();
        assert_eq!(
            (result.net_cents, result.tax_cents, result.gross_cents),
            (51700, 9823, 61523)
        );
    }
    #[test]
    fn rounds_each_line_to_cent() {
        let mut invoice = sample();
        invoice.items = vec![Item {
            description: "Test".into(),
            detail: "".into(),
            quantity: 1.0,
            unit: "h".into(),
            unit_code: "HUR".into(),
            unit_price_cents: 3,
            vat_percent: 19,
        }];
        assert_eq!(invoice.finalize().unwrap().tax_cents, 1);
    }
    #[test]
    fn rounds_tax_once_per_rate() {
        let mut invoice = sample();
        invoice.items = vec![
            Item {
                description: "A".into(),
                detail: "".into(),
                quantity: 1.0,
                unit: "h".into(),
                unit_code: "HUR".into(),
                unit_price_cents: 3,
                vat_percent: 19,
            },
            Item {
                description: "B".into(),
                detail: "".into(),
                quantity: 1.0,
                unit: "h".into(),
                unit_code: "HUR".into(),
                unit_price_cents: 3,
                vat_percent: 19,
            },
        ];
        assert_eq!(invoice.finalize().unwrap().tax_cents, 1);
    }
}


pub fn line_total(quantity: f64, price: i64) -> Result<i64, String> {
    if !quantity.is_finite() || quantity <= 0.0 || price < 0 { return Err("Ungültige Rechnungsposition".into()); }
    let decimal=quantity.to_string();
    let precision=decimal.split_once('.').map_or(0,|(_,fraction)|fraction.len());
    let scale=10_i128.checked_pow(precision as u32).ok_or("Menge hat zu viele Nachkommastellen")?;
    let numerator=decimal.replace('.', "").parse::<i128>().map_err(|_|"Menge ist zu groß")?;
    let total=numerator.checked_mul(i128::from(price)).and_then(|value|value.checked_add(scale/2)).ok_or("Betragsüberlauf")?/scale;
    i64::try_from(total).map_err(|_|"Betragsüberlauf".into())
}

pub fn unit_label(unit: &str, quantity: f64) -> &str {
    match unit { "Stunden" | "Stunde" | "h" => "h", "Tage" | "Tag" if quantity == 1.0 => "Tag", "Tage" | "Tag" => "Tage", _ => unit }
}
