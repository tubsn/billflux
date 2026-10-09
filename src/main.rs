#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod draft;
mod model;
mod view;
mod xml;
mod export;
mod production;
mod renderer;
mod store;

#[tauri::command]
fn load_workspace() -> Result<draft::Workspace, String> { store::load() }
#[tauri::command]
fn load_app() -> Result<store::AppData,String> { store::load_app() }
#[tauri::command]
fn load_statistics(company_id:i64) -> Result<store::StatisticsData,String> { store::statistics(company_id) }
#[tauri::command]
fn select_company(company_id:i64) -> Result<(),String> { store::select_company(company_id) }
#[tauri::command]
fn save_settings(company_id:i64,settings:store::Settings) -> Result<(),String> { store::save_settings(company_id,settings) }
#[tauri::command]
fn save_company_settings(company_id:i64,party:draft::Party,settings:store::Settings) -> Result<store::Company,String> { store::save_company_settings(company_id,party,settings) }
#[tauri::command]
fn save_customer(company_id:i64,party:draft::Party) -> Result<store::Customer,String> { store::save_customer(company_id,party) }

#[tauri::command]
fn update_customer(id:i64,party:draft::Party) -> Result<store::Customer,String> { store::update_customer(id,party) }
#[tauri::command]
fn delete_customer(id:i64) -> Result<(),String> { store::delete_customer(id) }
#[tauri::command]
fn duplicate_invoice(id:i64,date:String) -> Result<draft::Workspace,String> { store::duplicate_invoice(id,&date) }
#[tauri::command]
fn delete_invoice(id:i64) -> Result<(),String> { store::delete_invoice(id) }
#[tauri::command]
fn save_workspace(workspace: draft::Workspace) -> Result<draft::Workspace, String> { store::save(&workspace) }
#[tauri::command]
fn create_company(name:String) -> Result<store::Company,String> { store::create_company(name) }
#[tauri::command]
fn save_company(company:store::Company) -> Result<store::Company,String> { store::save_company(company) }
#[tauri::command]
fn new_invoice(company_id:i64,date:String) -> Result<draft::Workspace,String> { store::new_invoice(company_id,&date) }
#[tauri::command]
fn open_invoice(id:i64) -> Result<draft::Workspace,String> { store::open_invoice(id) }

#[tauri::command]
fn preview_invoice(workspace:draft::Workspace) -> Result<String,String> { production::preview_document(workspace) }
#[tauri::command]
fn preview_template(workspace:draft::Workspace,template:String) -> Result<String,String> { production::preview_template(workspace,template) }
#[tauri::command]
fn calculate(workspace: draft::Workspace) -> Result<draft::Totals, String> {
    draft::calculate(&workspace)
}

#[tauri::command]
async fn export_invoice(window: tauri::Window, workspace: draft::Workspace) -> Result<Option<production::ExportResult>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        production::validate_export(&workspace)?;
        let saved=store::save(&workspace)?;
        let destination=rfd::FileDialog::new().set_parent(&window).set_title("PDF Exportieren")
            .add_filter("PDF", &["pdf"]).set_file_name(format!("{}.pdf",workspace.number)).save_file();
        let Some(destination)=destination else{return Ok(None);};
        let result=production::create_to(saved.clone(),Some(destination))?;
        store::mark_issued(saved.invoice_id,&result.pdf_path,&result.report_path)?;
        Ok(Some(result))
    })
        .await.map_err(|e| e.to_string())?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_workspace, load_app, load_statistics, select_company, save_settings, save_company_settings, save_customer, update_customer, delete_customer, duplicate_invoice, delete_invoice, save_workspace, create_company, save_company, new_invoice, open_invoice, preview_invoice, preview_template, calculate, export_invoice])
        .run(tauri::generate_context!())
        .expect("Billflux konnte nicht gestartet werden");
}
