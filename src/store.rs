use crate::draft::{Party, Workspace};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Company { pub id: i64, pub party: Party, pub template: String }
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings { pub invoice_prefix:String, pub subject_prefix:String, pub payment_reference_prefix:String, pub hourly_rate_cents:i64, pub next_invoice_number:String, pub account_holder:String, pub iban:String, pub bic:String, pub bank_name:String, pub due_days:u32, pub recent_count:u32 }
impl Default for Settings { fn default()->Self{Self{invoice_prefix:String::new(),subject_prefix:"Rechnung".into(),payment_reference_prefix:String::new(),hourly_rate_cents:0,next_invoice_number:String::new(),account_holder:String::new(),iban:String::new(),bic:String::new(),bank_name:String::new(),due_days:14,recent_count:5}} }
#[derive(Clone, Serialize, Deserialize)]
pub struct Customer { pub id:i64, pub company_id:i64, pub party:Party }
#[derive(Serialize)]
pub struct InvoiceRow { pub id: i64, pub company_id: i64, pub number: String, pub subject:String, pub buyer: String, pub date: String, pub status: String }

#[derive(Serialize)]
pub struct StatisticInvoice { pub id:i64, pub number:String, pub date:String, pub status:String, pub customer_id:i64, pub customer:String, pub item_count:usize, pub net_cents:i64, pub tax_cents:i64, pub gross_cents:i64, pub hours:f64, pub hourly_net_cents:i64 }

#[derive(Serialize)]
pub struct StatisticsData { pub invoices:Vec<StatisticInvoice>, pub skipped:usize, pub incomplete:usize }

pub fn statistics(company_id:i64) -> Result<StatisticsData,String> {
    let conn=db()?;
    statistics_from_connection(&conn,company_id)
}
fn statistics_from_connection(conn:&Connection,company_id:i64) -> Result<StatisticsData,String> {
    let mut stmt=conn.prepare("SELECT id,number,status,data FROM invoices WHERE company_id=?1 AND deleted_at IS NULL ORDER BY id DESC").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![company_id],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).map_err(|e|e.to_string())?;
    let mut invoices=Vec::new(); let mut skipped=0; let mut incomplete=0;
    for row in rows {
        let (id,number,status,json)=row.map_err(|e|e.to_string())?;
        let Ok(data)=serde_json::from_str::<Workspace>(&json) else {skipped+=1;continue};
        if data.buyer.name.trim().is_empty() || data.items.is_empty() {incomplete+=1;continue;}
        let Ok(total)=crate::draft::calculate(&data) else {skipped+=1;continue};
        let mut hours=0.0; let mut hourly_net_cents=0;
        for (item,line) in data.items.iter().zip(&total.lines) {
            if matches!(item.unit.trim().to_lowercase().as_str(),"h"|"std"|"std."|"stunde"|"stunden") {
                hours+=item.quantity; hourly_net_cents+=line.net_cents;
            }
        }
        invoices.push(StatisticInvoice{id,number,date:data.date,status,customer_id:data.customer_id,customer:data.buyer.name,item_count:data.items.len(),net_cents:total.net_cents,tax_cents:total.tax_cents,gross_cents:total.gross_cents,hours,hourly_net_cents});
    }
    Ok(StatisticsData{invoices,skipped,incomplete})
}
#[derive(Serialize)]
pub struct AppData { pub workspace: Workspace, pub companies: Vec<Company>, pub invoices: Vec<InvoiceRow>, pub customers:Vec<Customer>, pub templates: Vec<String>, pub settings:Settings, pub active_company_id:i64, pub status: String }

fn base() -> Result<PathBuf,String> {
    #[cfg(test)]
    if let Ok(path)=std::env::var("BILLFLUX_TEST_BASE") { return Ok(PathBuf::from(path)); }
    let exe=std::env::current_exe().map_err(|e|e.to_string())?;
    let dir=exe.parent().ok_or("Programmverzeichnis fehlt")?;
    if dir.join("templates").is_dir(){Ok(dir.to_path_buf())} else {Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")))}
}
fn update_export_paths(conn:&Connection,old:&str,new:&str) -> Result<(),String> {
    for field in ["pdf_path","report_path"] {
        conn.execute(&format!("UPDATE invoices SET {field}=?2 || substr({field},length(?1)+1) WHERE substr({field},1,length(?1))=?1"),params![old,new]).map_err(|e|e.to_string())?;
    }
    Ok(())
}
fn db() -> Result<Connection,String> {
    let dir=base()?.join("database");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let path=dir.join("billflux.sqlite");
    let backup=dir.join("billflux.v1-backup.sqlite");
    if path.is_file()&&!backup.exists(){std::fs::copy(&path,&backup).map_err(|e|e.to_string())?;}
    let conn=Connection::open(path).map_err(|e|e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys=ON;
      CREATE TABLE IF NOT EXISTS workspace(id INTEGER PRIMARY KEY CHECK(id=1),data TEXT NOT NULL,updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
      CREATE TABLE IF NOT EXISTS companies(id INTEGER PRIMARY KEY,data TEXT NOT NULL,template TEXT NOT NULL DEFAULT 'standard');
      CREATE TABLE IF NOT EXISTS company_settings(company_id INTEGER PRIMARY KEY REFERENCES companies(id),data TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS customers(id INTEGER PRIMARY KEY,company_id INTEGER NOT NULL REFERENCES companies(id),data TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS invoices(id INTEGER PRIMARY KEY,company_id INTEGER NOT NULL REFERENCES companies(id),number TEXT NOT NULL UNIQUE,status TEXT NOT NULL CHECK(status IN ('draft','issued')),data TEXT NOT NULL,pdf_path TEXT NOT NULL DEFAULT '',report_path TEXT NOT NULL DEFAULT '');
      CREATE TABLE IF NOT EXISTS app_state(id INTEGER PRIMARY KEY CHECK(id=1),active_id INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS selected_company(id INTEGER PRIMARY KEY CHECK(id=1),company_id INTEGER NOT NULL REFERENCES companies(id));").map_err(|e|e.to_string())?;
    let has_deleted=conn.prepare("PRAGMA table_info(invoices)").map_err(|e|e.to_string())?.query_map([],|r|r.get::<_,String>(1)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?.iter().any(|column|column=="deleted_at");
    if !has_deleted{conn.execute("ALTER TABLE invoices ADD COLUMN deleted_at TEXT",[]).map_err(|e|e.to_string())?;}
    let count:i64=conn.query_row("SELECT COUNT(*) FROM companies",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if count==0 {
        conn.execute_batch("BEGIN IMMEDIATE").map_err(|e|e.to_string())?;
        let old:Option<String>=conn.query_row("SELECT data FROM workspace WHERE id=1",[],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        let mut draft:Workspace=old.as_ref().map(|s|serde_json::from_str(s)).transpose().map_err(|e|e.to_string())?.unwrap_or_default();
        let names=templates()?;
        let preferred=if old.is_some(){"standard"}else{"example"};
        let template=available_template(preferred,&names)?;
        conn.execute("INSERT INTO companies(data,template) VALUES(?1,?2)",params![serde_json::to_string(&draft.seller).map_err(|e|e.to_string())?,template]).map_err(|e|e.to_string())?;
        draft.company_id=conn.last_insert_rowid();
        let pdf=base()?.join("logs").join(format!("{}.pdf",draft.number));
        let status=if pdf.is_file(){"issued"}else{"draft"};
        conn.execute("INSERT INTO invoices(company_id,number,status,data,pdf_path) VALUES(?1,?2,?3,?4,?5)",params![draft.company_id,draft.number,status,serde_json::to_string(&draft).map_err(|e|e.to_string())?,if pdf.is_file(){pdf.display().to_string()}else{String::new()}]).map_err(|e|e.to_string())?;
        conn.execute("INSERT INTO app_state(id,active_id) VALUES(1,?1)",params![conn.last_insert_rowid()]).map_err(|e|e.to_string())?;
        conn.execute("INSERT INTO selected_company(id,company_id) VALUES(1,?1)",params![draft.company_id]).map_err(|e|e.to_string())?;
        conn.execute_batch("COMMIT").map_err(|e|e.to_string())?;
    }
    conn.execute("INSERT OR IGNORE INTO selected_company(id,company_id) SELECT 1,company_id FROM invoices ORDER BY id DESC LIMIT 1",[]).map_err(|e|e.to_string())?;
    let root=base()?;
    if !root.join("output").exists() && root.join("logs").is_dir() {
        update_export_paths(&conn,&root.join("output").display().to_string(),&root.join("logs").display().to_string())?;
    }
    Ok(conn)
}
pub fn templates() -> Result<Vec<String>,String> {
    let mut names=Vec::new();
    for e in std::fs::read_dir(base()?.join("templates")).map_err(|e|e.to_string())? {
        let e=e.map_err(|e|e.to_string())?;
        if e.path().join("invoice.html").is_file() && e.path().join("style.css").is_file(){names.push(e.file_name().to_string_lossy().into_owned());}
    }
    names.sort();Ok(names)
}
fn available_template(current:&str,names:&[String]) -> Result<String,String> {
    if names.iter().any(|name|name==current){Ok(current.into())}
    else{names.first().cloned().ok_or("Keine Rechnungsvorlage gefunden".into())}
}
pub fn company(id:i64) -> Result<Company,String> {
    let conn=db()?;
    let (data,template):(String,String)=conn.query_row("SELECT data,template FROM companies WHERE id=?1",params![id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    let selected=available_template(&template,&templates()?)?;
    if selected!=template {conn.execute("UPDATE companies SET template=?1 WHERE id=?2",params![selected,id]).map_err(|e|e.to_string())?;}
    Ok(Company{id,party:serde_json::from_str(&data).map_err(|e|e.to_string())?,template:selected})
}
pub fn settings(company_id:i64) -> Result<Settings,String> {
    let conn=db()?;
    let json:Option<String>=conn.query_row("SELECT data FROM company_settings WHERE company_id=?1",params![company_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    json.map(|value|serde_json::from_str(&value).map_err(|e|e.to_string())).unwrap_or_else(||Ok(Settings::default()))
}
fn validate_settings(value:&Settings) -> Result<(),String> {
    if value.hourly_rate_cents<0{return Err("Stundensatz darf nicht negativ sein".into());}
    if value.due_days==0||value.due_days>365{return Err("Zahlungsziel muss zwischen 1 und 365 Tagen liegen".into());}
    if value.recent_count==0||value.recent_count>20{return Err("Anzahl letzter Rechnungen muss zwischen 1 und 20 liegen".into());}
    number_start(value,"2026-10-07")?;
    Ok(())
}
pub fn save_settings(company_id:i64,value:Settings) -> Result<(),String> {
    validate_settings(&value)?;
    company(company_id)?;
    db()?.execute("INSERT INTO company_settings(company_id,data) VALUES(?1,?2) ON CONFLICT(company_id) DO UPDATE SET data=excluded.data",params![company_id,serde_json::to_string(&value).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn save_company_settings(company_id:i64,party:Party,value:Settings) -> Result<Company,String> {
    validate_settings(&value)?;
    if party.name.trim().is_empty(){return Err("Firmenname fehlt".into());}
    let mut conn=db()?;
    let tx=conn.transaction().map_err(|e|e.to_string())?;
    tx.execute("UPDATE companies SET data=?1 WHERE id=?2",params![serde_json::to_string(&party).map_err(|e|e.to_string())?,company_id]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO company_settings(company_id,data) VALUES(?1,?2) ON CONFLICT(company_id) DO UPDATE SET data=excluded.data",params![company_id,serde_json::to_string(&value).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    company(company_id)
}
pub fn select_company(company_id:i64) -> Result<(),String> {
    company(company_id)?;
    db()?.execute("INSERT INTO selected_company(id,company_id) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET company_id=excluded.company_id",params![company_id]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn save_customer(company_id:i64,party:Party) -> Result<Customer,String> {
    if party.name.trim().is_empty(){return Err("Kundenname fehlt".into());}
    company(company_id)?;
    let conn=db()?;
    conn.execute("INSERT INTO customers(company_id,data) VALUES(?1,?2)",params![company_id,serde_json::to_string(&party).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    Ok(Customer{id:conn.last_insert_rowid(),company_id,party})
}
pub fn delete_customer(id:i64) -> Result<(),String> {
    if db()?.execute("DELETE FROM customers WHERE id=?1",params![id]).map_err(|e|e.to_string())?==0 {return Err("Kunde nicht gefunden".into());}
    Ok(())
}
pub fn load_app() -> Result<AppData,String> {
    let conn=db()?;
    let names=templates()?;
    let mut stmt=conn.prepare("SELECT id,data,template FROM companies ORDER BY id").map_err(|e|e.to_string())?;
    let rows=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).map_err(|e|e.to_string())?
        .collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let companies=rows.into_iter().map(|(id,json,template)|{let selected=available_template(&template,&names)?;if selected!=template {conn.execute("UPDATE companies SET template=?1 WHERE id=?2",params![selected,id]).map_err(|e|e.to_string())?;} Ok(Company{id,party:serde_json::from_str(&json).map_err(|e|e.to_string())?,template:selected})}).collect::<Result<Vec<_>,String>>()?;
    let mut stmt=conn.prepare("SELECT id,company_id,number,status,data FROM invoices WHERE deleted_at IS NULL ORDER BY id DESC").map_err(|e|e.to_string())?;
    let records=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let invoices=records.iter().map(|(id,company_id,number,status,json)|{let d:Workspace=serde_json::from_str(json).map_err(|e|e.to_string())?;Ok(InvoiceRow{id:*id,company_id:*company_id,number:number.clone(),subject:d.subject,buyer:d.buyer.name,date:d.date,status:status.clone()})}).collect::<Result<Vec<_>,String>>()?;
    let mut stmt=conn.prepare("SELECT id,company_id,data FROM customers ORDER BY id DESC").map_err(|e|e.to_string())?;
    let customers=stmt.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?))).map_err(|e|e.to_string())?
        .map(|row|{let(id,company_id,json)=row.map_err(|e|e.to_string())?;Ok(Customer{id,company_id,party:serde_json::from_str(&json).map_err(|e|e.to_string())?})}).collect::<Result<Vec<_>,String>>()?;
    let active:i64=conn.query_row("SELECT active_id FROM app_state WHERE id=1",[],|r|r.get(0)).unwrap_or(0);
    let active_company_id:i64=conn.query_row("SELECT company_id FROM selected_company WHERE id=1",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    let current=records.iter().find(|r|r.0==active).or_else(||records.iter().find(|r|r.1==active_company_id));
    let (workspace,status)=if let Some(current)=current {
        let mut workspace:Workspace=serde_json::from_str(&current.4).map_err(|e|e.to_string())?;
        workspace.invoice_id=current.0;workspace.company_id=current.1;
        (workspace,current.3.clone())
    }else{(new_invoice(active_company_id,&chrono::Local::now().format("%Y-%m-%d").to_string())?,"draft".into())};
    Ok(AppData{workspace,companies,invoices,customers,templates:names,settings:settings(active_company_id)?,active_company_id,status})
}
pub fn load() -> Result<Workspace,String>{Ok(load_app()?.workspace)}
pub fn save(data:&Workspace) -> Result<Workspace,String> {
    if data.number.trim().is_empty(){return Err("Rechnungsnummer fehlt".into());}
    if !data.number.chars().all(|c|c.is_ascii_alphanumeric()||c=='-'||c=='_'){return Err("Rechnungsnummer darf nur Buchstaben, Zahlen, - und _ enthalten".into());}
    if data.company_id==0{return Err("Bitte zuerst eine eigene Firma wählen".into());}
    let mut conn=db()?;let tx=conn.transaction().map_err(|e|e.to_string())?;
    let other:Option<i64>=tx.query_row("SELECT id FROM invoices WHERE number=?1 AND id<>?2",params![data.number.trim(),data.invoice_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    if other.is_some(){return Err("Diese Rechnungsnummer ist bereits vergeben".into());}
    let previous:Option<String>=tx.query_row("SELECT number FROM invoices WHERE id=?1 AND deleted_at IS NULL",params![data.invoice_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    if data.invoice_id>0 && previous.is_none(){return Err("Rechnung nicht gefunden".into());}
    if previous.as_deref()!=Some(data.number.as_str()) && base()?.join("logs").join(format!("{}.pdf",data.number)).is_file(){return Err("Zu dieser Rechnungsnummer existiert bereits eine PDF im Ausgabeordner".into());}
    let mut saved=data.clone();
    if saved.invoice_id==0 {
        tx.execute("INSERT INTO invoices(company_id,number,status,data) VALUES(?1,?2,'draft',?3)",params![saved.company_id,saved.number,serde_json::to_string(&saved).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
        saved.invoice_id=tx.last_insert_rowid();
    }
    tx.execute("UPDATE invoices SET company_id=?1,number=?2,status=CASE WHEN data<>?3 THEN 'draft' ELSE status END,data=?3 WHERE id=?4",params![saved.company_id,saved.number,serde_json::to_string(&saved).map_err(|e|e.to_string())?,saved.invoice_id]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO app_state(id,active_id) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET active_id=excluded.active_id",params![saved.invoice_id]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;Ok(saved)
}
pub fn create_company(name:String) -> Result<Company,String> {
    let name=name.trim();if name.is_empty(){return Err("Firmenname fehlt".into());}
    let party=Party{name:name.into(),..Party::default()};let conn=db()?;
    conn.execute("INSERT INTO companies(data) VALUES(?1)",params![serde_json::to_string(&party).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    let id=conn.last_insert_rowid();
    let template=format!("firma-{id}");
    let names=templates()?;
    let source_name=if names.iter().any(|name|name=="standard"){"standard"}else{names.first().ok_or("Keine Rechnungsvorlage gefunden")?};
    let source=base()?.join("templates").join(source_name);
    let destination=base()?.join("templates").join(&template);
    copy_template(&source,&destination)?;
    conn.execute("UPDATE companies SET template=?1 WHERE id=?2",params![template,id]).map_err(|e|e.to_string())?;
    Ok(Company{id,party,template})
}
fn copy_template(source:&std::path::Path,destination:&std::path::Path) -> Result<(),String> {
    std::fs::create_dir_all(destination).map_err(|e|e.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;
        let target=destination.join(entry.file_name());
        if entry.path().is_dir(){copy_template(&entry.path(),&target)?;}
        else{std::fs::copy(entry.path(),target).map_err(|e|e.to_string())?;}
    }
    Ok(())
}
pub fn save_company(company:Company) -> Result<Company,String> {
    if company.party.name.trim().is_empty(){return Err("Firmenname fehlt".into());}
    if !templates()?.contains(&company.template){return Err("Vorlage nicht gefunden".into());}
    if db()?.execute("UPDATE companies SET data=?1,template=?2 WHERE id=?3",params![serde_json::to_string(&company.party).map_err(|e|e.to_string())?,company.template,company.id]).map_err(|e|e.to_string())?==0{return Err("Firma nicht gefunden".into());}
    Ok(company)
}
pub fn new_invoice(company_id:i64,date:&str) -> Result<Workspace,String> {
    let firm=company(company_id)?;
    let preferences=settings(company_id)?;
    let conn=db()?;
    let start=number_start(&preferences,date)?;
    let mut stmt=conn.prepare("SELECT number FROM invoices").map_err(|e|e.to_string())?;
    let mut used=stmt.query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let output=base()?.join("logs");
    if output.is_dir(){
        for entry in std::fs::read_dir(output).map_err(|e|e.to_string())?{
            let path=entry.map_err(|e|e.to_string())?.path();
            if path.extension().is_some_and(|ext|ext.eq_ignore_ascii_case("pdf")){
                if let Some(stem)=path.file_stem(){used.push(stem.to_string_lossy().into_owned());}
            }
        }
    }
    let number=next_number(&start,&used)?;
    select_company(company_id)?;
    Ok(Workspace{invoice_id:0,company_id,seller:firm.party,number,date:date.into(),subject_prefix:preferences.subject_prefix,payment_reference_prefix:preferences.payment_reference_prefix,
        items:vec![crate::draft::Item{unit_price_cents:preferences.hourly_rate_cents,..crate::draft::Item::default()}],
        account_holder:preferences.account_holder,iban:preferences.iban,bic:preferences.bic,bank_name:preferences.bank_name,..Workspace::default()})
}
pub fn open_invoice(id:i64) -> Result<Workspace,String> {
    let conn=db()?;let (json,company_id):(String,i64)=conn.query_row("SELECT data,company_id FROM invoices WHERE id=?1 AND deleted_at IS NULL",params![id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO app_state(id,active_id) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET active_id=excluded.active_id",params![id]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO selected_company(id,company_id) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET company_id=excluded.company_id",params![company_id]).map_err(|e|e.to_string())?;
    let mut data:Workspace=serde_json::from_str(&json).map_err(|e|e.to_string())?;data.invoice_id=id;data.company_id=company_id;Ok(data)
}
pub fn mark_issued(id:i64,pdf:&str,report:&str) -> Result<(),String> {
    if db()?.execute("UPDATE invoices SET status='issued',pdf_path=?1,report_path=?2 WHERE id=?3",params![pdf,report,id]).map_err(|e|e.to_string())?==0{return Err("Rechnung konnte nicht als ausgestellt markiert werden".into());}Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn statistics_separate_companies_and_deleted_invoices() {
        let conn=Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE invoices(id INTEGER PRIMARY KEY,company_id INTEGER,number TEXT,status TEXT,data TEXT,deleted_at TEXT);").unwrap();
        let mut invoice=Workspace::default();
        invoice.buyer.name="Kunde".into();
        invoice.date="2026-02-01".into();
        invoice.items[0].quantity=1.5;
        invoice.items[0].unit_price_cents=101;
        let json=serde_json::to_string(&invoice).unwrap();
        for (company,status,deleted) in [(1,"issued",None),(1,"draft",None),(1,"issued",Some("2026-01-01")),(2,"issued",None)] {
            conn.execute("INSERT INTO invoices(company_id,number,status,data,deleted_at) VALUES(?1,'R',?2,?3,?4)",params![company,status,json,deleted]).unwrap();
        }
        let result=statistics_from_connection(&conn,1).unwrap();
        assert_eq!(result.invoices.len(),2);
        let issued=result.invoices.iter().find(|row|row.status=="issued").unwrap();
        assert_eq!(issued.net_cents,152);
        assert_eq!(issued.hours,1.5);
        assert_eq!(result.incomplete,0);
        invoice.buyer.name.clear();
        conn.execute("INSERT INTO invoices(company_id,number,status,data) VALUES(1,'R','draft',?1)",params![serde_json::to_string(&invoice).unwrap()]).unwrap();
        let result=statistics_from_connection(&conn,1).unwrap();
        assert_eq!(result.invoices.len(),2);
        assert_eq!(result.incomplete,1);
    }
    #[test]
    fn moving_output_to_logs_updates_saved_invoice_paths() {
        let conn=Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE invoices(pdf_path TEXT,report_path TEXT);").unwrap();
        conn.execute("INSERT INTO invoices VALUES(?1,?2)",params![r"D:\Billflux\output\RE-1.pdf",r"D:\Billflux\output\RE-1.validation.xml"]).unwrap();
        update_export_paths(&conn,r"D:\Billflux\output",r"D:\Billflux\logs").unwrap();
        let (pdf,report):(String,String)=conn.query_row("SELECT pdf_path,report_path FROM invoices",[],|row|Ok((row.get(0)?,row.get(1)?))).unwrap();
        assert_eq!(pdf,r"D:\Billflux\logs\RE-1.pdf");
        assert_eq!(report,r"D:\Billflux\logs\RE-1.validation.xml");
    }
    #[test]
    fn missing_template_uses_first_available_theme() {
        let names=vec!["artmessengers".into(),"example".into(),"tiefblau".into()];
        assert_eq!(available_template("standard",&names).unwrap(),"artmessengers");
        assert_eq!(available_template("tiefblau",&names).unwrap(),"tiefblau");
        assert!(available_template("standard",&[]).is_err());
    }
    #[test]
    #[ignore = "benötigt BILLFLUX_EXPORT_TEST_DB"]
    fn migrates_existing_database_copy() {
        let source=std::env::var("BILLFLUX_EXPORT_TEST_DB").unwrap();
        let root=std::env::temp_dir().join(format!("billflux-migration-check-{}",std::process::id()));
        std::fs::create_dir_all(root.join("database")).unwrap();
        std::fs::create_dir_all(root.join("templates/standard")).unwrap();
        std::fs::write(root.join("templates/standard/invoice.html"),"").unwrap();
        std::fs::write(root.join("templates/standard/style.css"),"").unwrap();
        let original=Connection::open(&source).unwrap();
        let old:String=original.query_row("SELECT data FROM workspace WHERE id=1",[],|r|r.get(0)).unwrap();
        drop(original);
        std::fs::copy(source,root.join("database/billflux.sqlite")).unwrap();
        std::env::set_var("BILLFLUX_TEST_BASE",&root);
        let app=load_app().unwrap();
        let old:Workspace=serde_json::from_str(&old).unwrap();
        assert_eq!(app.workspace.seller.name,old.seller.name);
        assert_eq!(app.workspace.buyer.name,old.buyer.name);
        assert_eq!(app.workspace.items.len(),old.items.len());
        assert!(root.join("database/billflux.v1-backup.sqlite").is_file());
        std::env::remove_var("BILLFLUX_TEST_BASE");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn date_patterns_and_separate_prefixes() {
        let mut settings=Settings::default();
        assert_eq!(number_start(&settings,"2026-10-07").unwrap(),"2026-0001");
        settings.next_invoice_number="YYYY-MM-0001".into();
        settings.invoice_prefix="INV-".into();
        settings.payment_reference_prefix="AM".into();
        assert_eq!(number_start(&settings,"2027-01-02").unwrap(),"INV-2027-01-0001");
        let used=vec!["INV-2027-01-0042".into()];
        assert_eq!(next_number(&number_start(&settings,"2027-01-02").unwrap(),&used).unwrap(),"INV-2027-01-0043");
        assert_eq!(next_number(&number_start(&settings,"2027-02-02").unwrap(),&used).unwrap(),"INV-2027-02-0001");
    }
    #[test]
    fn start_number_respects_highest_used_number() {
        let used=vec!["RE-2026-0009".into(),"RE-2026-0020".into(),"OTHER-0099".into()];
        assert_eq!(next_number("RE-2026-0010",&used).unwrap(),"RE-2026-0021");
        assert_eq!(next_number("RE-2026-0042",&used).unwrap(),"RE-2026-0042");
        assert_eq!(next_number("RE-2027-0001",&used).unwrap(),"RE-2027-0001");
        assert!(next_number("invalid",&used).is_err());
    }
    #[test]
    fn migrates_legacy_and_rejects_duplicate_number() {
        let root=std::env::temp_dir().join(format!("billflux-store-test-{}",std::process::id()));
        std::fs::create_dir_all(root.join("database")).unwrap();
        std::fs::create_dir_all(root.join("templates/standard")).unwrap();
        std::fs::write(root.join("templates/standard/invoice.html"),"").unwrap();
        std::fs::write(root.join("templates/standard/style.css"),"").unwrap();
        let legacy=Connection::open(root.join("database/billflux.sqlite")).unwrap();
        legacy.execute_batch("CREATE TABLE workspace(id INTEGER PRIMARY KEY,data TEXT NOT NULL);").unwrap();
        let old=Workspace::default();
        legacy.execute("INSERT INTO workspace(id,data) VALUES(1,?1)",params![serde_json::to_string(&old).unwrap()]).unwrap();
        drop(legacy);
        std::env::set_var("BILLFLUX_TEST_BASE",&root);
        let app=load_app().unwrap();
        assert_eq!((app.companies.len(),app.invoices.len(),app.workspace.number.as_str()),(1,1,old.number.as_str()));
        let second=create_company("Zweite Firma".into()).unwrap();
        assert_eq!(second.template,format!("firma-{}",second.id));
        assert!(root.join("templates").join(&second.template).join("invoice.html").is_file());
        save_settings(second.id,Settings{account_holder:"Zweite Firma".into(),iban:"DE123".into(),due_days:21,..Settings::default()}).unwrap();
        let customer=save_customer(second.id,Party{name:"Kundin".into(),contact:"Einkauf".into(),..Party::default()}).unwrap();
        save_settings(second.id,Settings{next_invoice_number:"RE-2026-0042".into(),..settings(second.id).unwrap()}).unwrap();
        save_company_settings(second.id,Party{name:"Zweite Firma aktualisiert".into(),..second.party.clone()},Settings{hourly_rate_cents:9550,..settings(second.id).unwrap()}).unwrap();
        assert_eq!(settings(app.active_company_id).unwrap().hourly_rate_cents,0);
        let mut invalid=settings(second.id).unwrap();invalid.hourly_rate_cents=-1;
        assert!(save_company_settings(second.id,Party{name:"Nicht speichern".into(),..Party::default()},invalid).is_err());
        assert_eq!(company(second.id).unwrap().party.name,"Zweite Firma aktualisiert");
        assert_eq!(settings(second.id).unwrap().hourly_rate_cents,9550);
        let mut next=new_invoice(second.id,"2026-10-07").unwrap();
        assert_eq!(next.items[0].unit_price_cents,9550);
        assert_eq!(next.number,"RE-2026-0042");
        next.buyer=customer.party.clone();next.customer_id=customer.id;next.payment_reference="Individueller Auftrag".into();
        next.items[0].quantity=1.5;
        next.items[0].unit_price_cents=8500;
        let fractional=save(&next).unwrap();
        assert_eq!(open_invoice(fractional.invoice_id).unwrap().items[0].quantity,1.5);
        update_customer(customer.id,Party{name:"Neuer Kundenname".into(),contact:"Buchhaltung".into(),..customer.party.clone()}).unwrap();
        select_company(app.active_company_id).unwrap();
        assert_eq!(load_app().unwrap().customers[0].party.name,"Neuer Kundenname");
        let old_invoice=open_invoice(fractional.invoice_id).unwrap();
        assert_eq!(old_invoice.buyer.name,"Kundin");assert_eq!(old_invoice.buyer.contact,"Einkauf");
        assert_eq!(old_invoice.payment_reference,"Individueller Auftrag");
        assert_eq!(new_invoice(second.id,"2026-10-07").unwrap().number,"RE-2026-0043");
        next.invoice_id=0;
        assert_eq!((next.account_holder.as_str(),next.iban.as_str()),("Zweite Firma","DE123"));
        assert_eq!(load_app().unwrap().customers.len(),1);
        next.number=app.workspace.number.clone();
        assert!(save(&next).err().unwrap().contains("bereits vergeben"));
        next.number="NEW-2026-0001".into();
        let saved=save(&next).unwrap();
        assert_eq!(load_app().unwrap().invoices.len(),3);
        mark_issued(saved.invoice_id,"test.pdf","test.xml").unwrap();
        assert!(save(&saved).is_ok());
        assert_eq!(load_app().unwrap().status,"issued");
        let mut edited=saved.clone();edited.subject="Bearbeitet".into();
        std::fs::create_dir_all(root.join("logs")).unwrap();
        std::fs::write(root.join("logs").join(format!("{}.pdf",edited.number)),"old export").unwrap();
        save(&edited).unwrap();
        assert_eq!(open_invoice(edited.invoice_id).unwrap().subject,"Bearbeitet");
        assert_eq!(load_app().unwrap().status,"draft");
        mark_issued(edited.invoice_id,"updated.pdf","updated.xml").unwrap();
        assert_eq!(load_app().unwrap().status,"issued");
        let duplicated=duplicate_invoice(edited.invoice_id,"2026-10-08").unwrap();
        assert_ne!(duplicated.number,edited.number);assert_ne!(duplicated.invoice_id,edited.invoice_id);
        assert_eq!(duplicated.date,"2026-10-08");assert_eq!(duplicated.buyer.name,edited.buyer.name);
        assert_eq!(duplicated.items[0].quantity,edited.items[0].quantity);assert!(duplicated.payment_reference.is_empty());
        let reserved=duplicated.number.clone();delete_invoice(duplicated.invoice_id).unwrap();
        assert!(open_invoice(duplicated.invoice_id).is_err());assert!(save(&duplicated).is_err());
        assert_ne!(new_invoice(second.id,"2026-10-08").unwrap().number,reserved);
        for row in load_app().unwrap().invoices {delete_invoice(row.id).unwrap();}
        let empty=load_app().unwrap();assert!(empty.invoices.is_empty());assert_eq!(empty.workspace.invoice_id,0);
        std::env::remove_var("BILLFLUX_TEST_BASE");
        std::fs::remove_dir_all(root).unwrap();
    }
}


fn number_parts(number:&str) -> Result<(&str,u64,usize),String> {
    if number.is_empty() || !number.chars().all(|c|c.is_ascii_alphanumeric()||c=='-'||c=='_') { return Err("Rechnungsnummer darf nur Buchstaben, Zahlen, - und _ enthalten".into()); }
    let offset=number.trim_end_matches(|c:char|c.is_ascii_digit()).len();
    let suffix=&number[offset..];
    let value=suffix.parse::<u64>().map_err(|_|"Die Startnummer muss mit einer laufenden Zahl enden")?;
    Ok((&number[..offset],value,suffix.len()))
}
fn next_number(start:&str,used:&[String]) -> Result<String,String> {
    let (prefix,mut next,width)=number_parts(start)?;
    for number in used {
        if let Ok((other,value,_))=number_parts(number){
            if other==prefix && value>=next {next=value.checked_add(1).ok_or("Nummernkreis ausgeschöpft")?;}
        }
    }
    Ok(format!("{prefix}{next:0width$}"))
}

fn number_start(settings:&Settings,date:&str) -> Result<String,String> {
    let date=chrono::NaiveDate::parse_from_str(date,"%Y-%m-%d").map_err(|_|"Ungültiges Rechnungsdatum")?;
    let pattern=if settings.next_invoice_number.is_empty(){"YYYY-0001"}else{&settings.next_invoice_number};
    if !settings.invoice_prefix.chars().all(|c|c.is_ascii_alphanumeric()||c=='-'||c=='_'){return Err("Rechnungsnummer-Präfix darf nur Buchstaben, Zahlen, - und _ enthalten".into());}
    let number=format!("{}{}",settings.invoice_prefix,pattern.replace("YYYY",&date.format("%Y").to_string()).replace("MM",&date.format("%m").to_string()));
    number_parts(&number)?;
    Ok(number)
}


pub fn update_customer(id:i64,party:Party) -> Result<Customer,String> {
    if party.name.trim().is_empty(){return Err("Firmenname / Name fehlt".into());}
    let conn=db()?;
    let company_id=conn.query_row("SELECT company_id FROM customers WHERE id=?1",params![id],|r|r.get(0)).map_err(|e|e.to_string())?;
    conn.execute("UPDATE customers SET data=?1 WHERE id=?2",params![serde_json::to_string(&party).map_err(|e|e.to_string())?,id]).map_err(|e|e.to_string())?;
    Ok(Customer{id,company_id,party})
}
pub fn duplicate_invoice(id:i64,date:&str) -> Result<Workspace,String> {
    let conn=db()?;
    let (json,company_id):(String,i64)=conn.query_row("SELECT data,company_id FROM invoices WHERE id=?1 AND deleted_at IS NULL",params![id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    let source:Workspace=serde_json::from_str(&json).map_err(|e|e.to_string())?;
    let mut copy=new_invoice(company_id,date)?;
    copy.buyer=source.buyer;copy.customer_id=source.customer_id;copy.subject=source.subject;copy.subject_prefix=source.subject_prefix;copy.items=source.items;
    save(&copy)
}
pub fn delete_invoice(id:i64) -> Result<(),String> {
    let mut conn=db()?;let tx=conn.transaction().map_err(|e|e.to_string())?;
    if tx.execute("UPDATE invoices SET deleted_at=CURRENT_TIMESTAMP WHERE id=?1 AND deleted_at IS NULL",params![id]).map_err(|e|e.to_string())?==0{return Err("Rechnung nicht gefunden".into());}
    tx.execute("DELETE FROM app_state WHERE active_id=?1",params![id]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;Ok(())
}
