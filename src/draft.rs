use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Party {
    pub name: String, pub contact: String, pub additional_info:String, pub street: String, pub city: String, pub country: String,
    pub email: String, pub phone: String, pub website: String, pub tax_number: String,
    pub vat_id: String, pub economic_id: String, pub alternative_name: String,
}
impl Default for Party {
    fn default() -> Self { Self { name: String::new(), contact:String::new(), additional_info:String::new(), street: String::new(), city: String::new(), country: "DE".into(), email: String::new(), phone: String::new(), website: String::new(), tax_number: String::new(), vat_id: String::new(), economic_id:String::new(), alternative_name:String::new() } }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Item { pub description: String, pub detail: String, pub quantity: f64, pub unit: String, pub unit_price_cents: i64, pub vat_percent: u32 }
impl Default for Item {
    fn default() -> Self { Self { description: String::new(), detail: String::new(), quantity: 1.0, unit: "Stunden".into(), unit_price_cents: 0, vat_percent: 19 } }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    pub invoice_id: i64, pub customer_id:i64, pub payment_reference:String, pub company_id: i64,
    pub seller: Party, pub buyer: Party, pub number: String, pub date: String,
    pub service_date: String, pub service_month: bool, #[serde(default="default_true")] pub number_in_subject: bool, pub due_date: String, pub subject: String, #[serde(default="String::new")] pub subject_prefix: String,
    pub payment_reference_prefix:String, pub payment_note:String, pub account_holder: String, pub iban: String, pub bic: String, pub bank_name: String,
    pub items: Vec<Item>,
}
fn default_true() -> bool { true }
impl Default for Workspace {
    fn default() -> Self { Self { invoice_id: 0, customer_id:0, payment_reference:String::new(), company_id: 0, seller: Party { name: "Musterfirma".into(), ..Party::default() }, buyer: Party::default(), number: format!("{}-0001",chrono::Local::now().format("%Y")), date: String::new(), service_date: String::new(), service_month:false, number_in_subject:true, due_date: String::new(), subject: String::new(), subject_prefix:String::new(), payment_reference_prefix:String::new(), payment_note:"Ich bedanke mich für die Zusammenarbeit.".into(), account_holder: String::new(), iban: String::new(), bic: String::new(), bank_name: String::new(), items: vec![Item::default()] } }
}

#[derive(Serialize)]
pub struct LineTotal { pub net_cents: i64 }
#[derive(Serialize)]
pub struct TaxTotal { pub rate: u32, pub basis_cents: i64, pub tax_cents: i64 }
#[derive(Serialize)]
pub struct Totals { pub lines: Vec<LineTotal>, pub taxes: Vec<TaxTotal>, pub net_cents: i64, pub tax_cents: i64, pub gross_cents: i64 }

pub fn calculate(data: &Workspace) -> Result<Totals, String> {
    use std::collections::BTreeMap;
    let mut lines = Vec::new();
    let mut groups = BTreeMap::<u32, i64>::new();
    let mut net = 0_i64;
    for item in &data.items {
        if !item.quantity.is_finite() || item.quantity <= 0.0 || item.unit_price_cents < 0 || item.vat_percent > 100 { return Err("Ungültige Rechnungsposition".into()); }
        let line = crate::model::line_total(item.quantity, item.unit_price_cents)?;
        net = net.checked_add(line).ok_or("Betragsüberlauf")?;
        let basis = groups.entry(item.vat_percent).or_default();
        *basis = basis.checked_add(line).ok_or("Betragsüberlauf")?;
        lines.push(LineTotal { net_cents: line });
    }
    let mut taxes = Vec::new();
    let mut tax_sum = 0_i64;
    for (rate, basis) in groups {
        let tax = basis.checked_mul(i64::from(rate)).and_then(|x| x.checked_add(50)).ok_or("Betragsüberlauf")? / 100;
        tax_sum = tax_sum.checked_add(tax).ok_or("Betragsüberlauf")?;
        taxes.push(TaxTotal { rate, basis_cents: basis, tax_cents: tax });
    }
    Ok(Totals { lines, taxes, net_cents: net, tax_cents: tax_sum, gross_cents: net.checked_add(tax_sum).ok_or("Betragsüberlauf")? })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn groups_tax_by_rate() {
        let mut d = Workspace::default();
        d.items = vec![Item { quantity: 2.0, unit_price_cents: 1000, ..Item::default() }, Item { quantity: 1.0, unit_price_cents: 500, ..Item::default() }];
        let t = calculate(&d).unwrap();
        assert_eq!((t.net_cents, t.tax_cents, t.gross_cents), (2500, 475, 2975));
    }
}
