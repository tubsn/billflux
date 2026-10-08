#![windows_subsystem = "windows"]
#[path = "../src/renderer.rs"] mod renderer;
use std::{fs,path::PathBuf,process::{Command,Stdio},os::windows::process::CommandExt,time::{Duration,SystemTime,UNIX_EPOCH}};
fn check()->Result<String,String>{
    let root=std::env::current_dir().map_err(|e|e.to_string())?;
    let chrome=std::env::var("BILLFLUX_RENDERER").map(PathBuf::from).unwrap_or_else(|_|root.join("dist/Billflux/bin/chrome-headless-shell/chrome-headless-shell.exe"));
    let source=std::env::args().nth(1).map(PathBuf::from).unwrap_or(root.join("dist/Billflux/preview/sample.html"));
    let id=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string();
    let profile=std::env::temp_dir().join(format!("billflux-gui-test-{id}"));
    let html=renderer::prepare(&chrome,&source,&profile,&id)?;
    if !html.starts_with("<!doctype html>") || !html.contains("data-pagination=\"ready\"") || html.contains("XMLHttpRequest") {
        return Err("Ungültiges fertiges Seitendokument".into());
    }
    let output=source.parent().unwrap().join("gui-renderer-test.html");
    fs::write(&output,html).map_err(|e|e.to_string())?;
    let pdf=root.join("tmp/pdfs/gui-renderer-test.pdf");
    fs::create_dir_all(pdf.parent().unwrap()).map_err(|e|e.to_string())?;
    let mut command=Command::new(chrome);
    command.args(["--headless","--disable-gpu","--no-sandbox","--no-first-run","--no-pdf-header-footer"])
        .arg(format!("--user-data-dir={}-print",profile.display())).arg(format!("--print-to-pdf={}",pdf.display()))
        .arg(format!("file:///{}",output.to_string_lossy().replace('\\',"/").replace(' ',"%20")))
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(0x08000000);
    let status=command.status().map_err(|e|e.to_string())?;
    if !status.success(){return Err(format!("Druck fehlgeschlagen: {status}"));}
    for _ in 0..100 {
        if fs::metadata(&pdf).is_ok_and(|file|file.len()>10000){fs::remove_file(output).map_err(|e|e.to_string())?;return Ok("GUI renderer and PDF export succeeded with stdout/stderr disabled.".into());}
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("PDF-Datei fehlt".into())
}
fn main(){
    fs::create_dir_all("tmp").unwrap();
    let result=check();
    let failed=result.is_err();
    fs::write("tmp/renderer-gui-result.txt",format!("{result:?}")).unwrap();
    if failed{std::process::exit(1);}
}
