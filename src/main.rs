mod export;
mod model;
mod view;
mod xml;

use std::{env, fs, path::PathBuf, process::Command};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let executable_dir = env::current_exe()?
        .parent()
        .ok_or("EXE-Verzeichnis nicht gefunden")?
        .to_path_buf();
    let base = if executable_dir
        .join("templates/standard/invoice.html")
        .is_file()
    {
        executable_dir
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    };
    let preview = base.join("preview");
    fs::create_dir_all(&preview)?;
    fs::create_dir_all(base.join("database"))?;
    fs::create_dir_all(base.join("output"))?;
    let invoice = model::sample();
    let final_invoice = invoice.finalize()?;
    let template = fs::read_to_string(base.join("templates/standard/invoice.html"))?;
    let html = view::html(&final_invoice, &template);
    let html_path = preview.join("sample.html");
    let xml_path = preview.join("sample.xml");
    fs::write(&html_path, html)?;
    fs::copy(
        base.join("templates/standard/style.css"),
        preview.join("style.css"),
    )?;
    for (family, names) in [
        ("Fira_Sans", ["FiraSans-Regular.ttf", "FiraSans-Bold.ttf"]),
        (
            "Fira_Sans_Condensed",
            [
                "FiraSansCondensed-Regular.ttf",
                "FiraSansCondensed-Bold.ttf",
            ],
        ),
    ] {
        let destination = preview.join("fonts").join(family);
        fs::create_dir_all(&destination)?;
        for name in names {
            let template_font = base
                .join("templates/standard/fonts")
                .join(family)
                .join(name);
            let source = if template_font.is_file() {
                template_font
            } else {
                base.join("fonts").join(family).join(name)
            };
            fs::copy(source, destination.join(name))?;
        }
    }
    fs::write(&xml_path, xml::render(&final_invoice))?;
    println!("Vorschau: {}", html_path.display());
    println!("XML-Entwurf: {}", xml_path.display());
    println!("Gesamt: {} EUR", model::money(final_invoice.gross_cents));
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--pdf" || arg == "--zugferd") {
        let bundled_chrome = base.join("vendor/chrome/chrome.exe");
        let chrome = if bundled_chrome.is_file() {
            bundled_chrome
        } else {
            PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe")
        };
        let pdf_path = preview.join("sample.pdf");
        let html_url = format!(
            "file:///{}",
            html_path
                .to_string_lossy()
                .replace('\\', "/")
                .replace(' ', "%20")
        );
        let status = Command::new(chrome)
            .arg("--headless")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--no-pdf-header-footer")
            .arg(format!("--print-to-pdf={}", pdf_path.display()))
            .arg(html_url)
            .status()?;
        if !status.success() || !pdf_path.exists() {
            return Err("Chrome konnte keine PDF erzeugen".into());
        }
        println!("PDF-Vorschau: {}", pdf_path.display());
        if args.iter().any(|arg| arg == "--zugferd") {
            let validated = export::create(&base, &preview, &pdf_path, &xml_path, &format!("PROTOTYP-{}", invoice.number))?;
            println!("Validierter Prototyp: {}", validated.display());
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Fehler: {error}");
        std::process::exit(1);
    }
}
