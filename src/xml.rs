use crate::model::{money, FinalInvoice, Party};

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn postal(party: &Party) -> String {
    let (postcode, city) = party.city.split_once(' ').unwrap_or(("", &party.city));
    let contact=if party.contact.is_empty(){String::new()}else{format!("<ram:LineTwo>{}</ram:LineTwo>",escape(&party.contact))};
    format!("<ram:PostalTradeAddress><ram:PostcodeCode>{}</ram:PostcodeCode><ram:LineOne>{}</ram:LineOne>{contact}<ram:CityName>{}</ram:CityName><ram:CountryID>{}</ram:CountryID></ram:PostalTradeAddress>",
        escape(postcode), escape(&party.street), escape(city), escape(&party.country))
}

fn date(value: &str) -> String {
    value.replace('-', "")
}

pub fn render(final_invoice: &FinalInvoice<'_>) -> String {
    let invoice = final_invoice.invoice;
    let seller_tax_registration = if invoice.seller.vat_id.trim().is_empty() {
        format!("<ram:SpecifiedTaxRegistration><ram:ID schemeID=\"FC\">{}</ram:ID></ram:SpecifiedTaxRegistration>", escape(&invoice.seller.tax_number))
    } else {
        format!("<ram:SpecifiedTaxRegistration><ram:ID schemeID=\"VA\">{}</ram:ID></ram:SpecifiedTaxRegistration>", escape(&invoice.seller.vat_id))
    };
    let seller_identifier = if invoice.seller.vat_id.trim().is_empty() {
        format!("<ram:ID>{}</ram:ID>", escape(&invoice.seller.tax_number))
    } else { String::new() };
    let delivery = if invoice.service_date.is_empty() { "<ram:ApplicableHeaderTradeDelivery/>".into() } else {
        format!("<ram:ApplicableHeaderTradeDelivery><ram:ActualDeliverySupplyChainEvent><ram:OccurrenceDateTime><udt:DateTimeString format=\"102\">{}</udt:DateTimeString></ram:OccurrenceDateTime></ram:ActualDeliverySupplyChainEvent></ram:ApplicableHeaderTradeDelivery>",date(&invoice.service_date))
    };
    let mut lines = String::new();
    for (index, line) in final_invoice.items.iter().enumerate() {
        lines.push_str(&format!(r#"
<ram:IncludedSupplyChainTradeLineItem>
  <ram:AssociatedDocumentLineDocument><ram:LineID>{id}</ram:LineID></ram:AssociatedDocumentLineDocument>
  <ram:SpecifiedTradeProduct><ram:Name>{name}</ram:Name><ram:Description>{detail}</ram:Description></ram:SpecifiedTradeProduct>
  <ram:SpecifiedLineTradeAgreement><ram:NetPriceProductTradePrice><ram:ChargeAmount>{price}</ram:ChargeAmount></ram:NetPriceProductTradePrice></ram:SpecifiedLineTradeAgreement>
  <ram:SpecifiedLineTradeDelivery><ram:BilledQuantity unitCode="{unit}">{quantity}</ram:BilledQuantity></ram:SpecifiedLineTradeDelivery>
  <ram:SpecifiedLineTradeSettlement>
    <ram:ApplicableTradeTax><ram:TypeCode>VAT</ram:TypeCode><ram:CategoryCode>S</ram:CategoryCode><ram:RateApplicablePercent>{rate}</ram:RateApplicablePercent></ram:ApplicableTradeTax>
    <ram:SpecifiedTradeSettlementLineMonetarySummation><ram:LineTotalAmount>{net}</ram:LineTotalAmount></ram:SpecifiedTradeSettlementLineMonetarySummation>
  </ram:SpecifiedLineTradeSettlement>
</ram:IncludedSupplyChainTradeLineItem>"#,
            id = index + 1, name = escape(&line.item.description), detail = escape(&line.item.detail), price = money(line.item.unit_price_cents),
            unit = escape(&line.item.unit_code), quantity = line.item.quantity,
            rate = line.item.vat_percent, net = money(line.net_cents)));
    }
    let mut taxes = String::new();
    for (rate, (basis, amount)) in &final_invoice.tax_by_rate {
        taxes.push_str(&format!(r#"<ram:ApplicableTradeTax><ram:CalculatedAmount>{}</ram:CalculatedAmount><ram:TypeCode>VAT</ram:TypeCode><ram:BasisAmount>{}</ram:BasisAmount><ram:CategoryCode>S</ram:CategoryCode><ram:RateApplicablePercent>{}</ram:RateApplicablePercent></ram:ApplicableTradeTax>"#,
            money(*amount), money(*basis), rate));
    }
    let bic_xml = if invoice.bic.trim().is_empty() { String::new() } else {
        format!("<ram:PayeeSpecifiedCreditorFinancialInstitution><ram:BICID>{}</ram:BICID></ram:PayeeSpecifiedCreditorFinancialInstitution>", escape(&invoice.bic))
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
  <rsm:ExchangedDocumentContext><ram:GuidelineSpecifiedDocumentContextParameter><ram:ID>urn:cen.eu:en16931:2017</ram:ID></ram:GuidelineSpecifiedDocumentContextParameter></rsm:ExchangedDocumentContext>
  <rsm:ExchangedDocument><ram:ID>{number}</ram:ID><ram:TypeCode>380</ram:TypeCode><ram:IssueDateTime><udt:DateTimeString format="102">{issue_date}</udt:DateTimeString></ram:IssueDateTime></rsm:ExchangedDocument>
  <rsm:SupplyChainTradeTransaction>
    {lines}
    <ram:ApplicableHeaderTradeAgreement>
      <ram:SellerTradeParty>{seller_identifier}<ram:Name>{seller_name}</ram:Name>{seller_postal}{seller_tax_registration}</ram:SellerTradeParty>
      <ram:BuyerTradeParty><ram:Name>{buyer_name}</ram:Name>{buyer_postal}</ram:BuyerTradeParty>
    </ram:ApplicableHeaderTradeAgreement>
    {delivery}
    <ram:ApplicableHeaderTradeSettlement>
      <ram:PaymentReference>{payment_reference}</ram:PaymentReference><ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
      <ram:SpecifiedTradeSettlementPaymentMeans><ram:TypeCode>58</ram:TypeCode><ram:PayeePartyCreditorFinancialAccount><ram:IBANID>{iban}</ram:IBANID><ram:AccountName>{account_holder}</ram:AccountName></ram:PayeePartyCreditorFinancialAccount>{bic_xml}</ram:SpecifiedTradeSettlementPaymentMeans>
      {taxes}
      <ram:SpecifiedTradePaymentTerms><ram:Description>Zahlbar bis {due_date_text}</ram:Description><ram:DueDateDateTime><udt:DateTimeString format="102">{due_date}</udt:DateTimeString></ram:DueDateDateTime></ram:SpecifiedTradePaymentTerms>
      <ram:SpecifiedTradeSettlementHeaderMonetarySummation><ram:LineTotalAmount>{net}</ram:LineTotalAmount><ram:TaxBasisTotalAmount>{net}</ram:TaxBasisTotalAmount><ram:TaxTotalAmount currencyID="EUR">{tax}</ram:TaxTotalAmount><ram:GrandTotalAmount>{gross}</ram:GrandTotalAmount><ram:DuePayableAmount>{gross}</ram:DuePayableAmount></ram:SpecifiedTradeSettlementHeaderMonetarySummation>
    </ram:ApplicableHeaderTradeSettlement>
  </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>"#,
        number = escape(&invoice.number),
        payment_reference = escape(&invoice.payment_reference),
        issue_date = date(&invoice.date),
        delivery = delivery,
        due_date = date(&invoice.due_date),
        due_date_text = escape(&invoice.due_date),
        lines = lines,
        seller_name = escape(&invoice.seller.name),
        seller_postal = postal(&invoice.seller),
        seller_tax_registration = seller_tax_registration,
        seller_identifier = seller_identifier,
        buyer_name = escape(&invoice.buyer.name),
        buyer_postal = postal(&invoice.buyer),
        iban = escape(&invoice.iban.replace(' ', "")),
        account_holder = escape(&invoice.account_holder),
        bic_xml = bic_xml,
        taxes = taxes,
        net = money(final_invoice.net_cents),
        tax = money(final_invoice.tax_cents),
        gross = money(final_invoice.gross_cents)
    )
}
