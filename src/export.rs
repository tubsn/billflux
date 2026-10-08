use quick_xml::{events::Event, Reader};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(windows)]
fn hide_window(command:&mut Command){use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
#[cfg(not(windows))]
fn hide_window(_: &mut Command){}

fn require(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Werkzeug fehlt: {}", path.display()).into())
    }
}

fn java_path(tools: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let folder = tools.join("jre11");
    let jre = fs::read_dir(&folder)?
        .next()
        .ok_or("Java 11 fehlt im Werkzeugordner")??
        .path();
    let java = jre.join("bin/java.exe");
    require(&java)?;
    Ok(java)
}

fn normalize_metadata(java: &Path, mustang: &Path, preview: &Path, source: &Path, output: &Path, xml: &Path) -> Result<(), Box<dyn Error>> {
    let helper = preview.join("metadata-helper");
    fs::create_dir_all(&helper)?;
    fs::write(helper.join("BillfluxPdfMetadata.class"), include_bytes!("java/BillfluxPdfMetadata.class"))?;
    let classpath = std::env::join_paths([helper.as_path(), mustang])?;
    let mut command = Command::new(java);
    command.arg("-cp").arg(classpath).arg("BillfluxPdfMetadata").arg(source).arg(output).arg(xml);
    hide_window(&mut command);
    let result = command.output()?;
    if !result.status.success() || !output.is_file() {
        return Err(format!("PDF-Metadaten konnten nicht gesetzt werden: {}", String::from_utf8_lossy(&result.stderr)).into());
    }
    Ok(())
}

fn report_valid(report: &str) -> Result<bool, Box<dyn Error>> {
    let mut reader = Reader::from_str(report);
    let mut path = Vec::<String>::new();
    let mut pdf = false;
    let mut xml = false;
    let mut overall = false;
    loop {
        match reader.read_event()? {
            Event::Start(element) => path.push(element.name().as_ref().to_owned()),
            Event::Empty(element) => {
                if element.name().as_ref() == "summary" {
                    let valid = element
                        .attributes()
                        .filter_map(Result::ok)
                        .any(|attribute| {
                            attribute.key.as_ref() == "status"
                                && attribute.value.as_ref() == "valid"
                        });
                    match path.as_slice() {
                        [root, section] if root == "validation" && section == "pdf" => pdf = valid,
                        [root, section] if root == "validation" && section == "xml" => xml = valid,
                        [root] if root == "validation" => overall = valid,
                        _ => {}
                    }
                }
            }
            Event::End(_) => {
                path.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(pdf && xml && overall)
}

#[allow(dead_code)]
pub fn create(
    base: &Path,
    preview: &Path,
    pdf: &Path,
    xml: &Path,
    output_stem: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    create_to(base,preview,pdf,xml,output_stem,None)
}

pub fn create_to(base:&Path,preview:&Path,pdf:&Path,xml:&Path,output_stem:&str,destination:Option<&Path>) -> Result<PathBuf,Box<dyn Error>> {
    let tools = if base.join("vendor/Mustang-CLI-2.26.0.jar").is_file() {
        base.join("vendor")
    } else {
        base.join(".tools")
    };
    let gs = tools.join("gs/Library/bin/gswin64c.exe");
    let mustang = tools.join("Mustang-CLI-2.26.0.jar");
    let definition = tools.join("PDFA_def.ps");
    let icc = tools.join("srgb.icc");
    for file in [&gs, &mustang, &definition, &icc] {
        require(file)?;
    }
    let java = java_path(&tools)?;
    let local_definition = preview.join("PDFA_local.ps");
    let icc_path = icc.to_string_lossy().replace('\\', "/");
    fs::write(
        &local_definition,
        fs::read_to_string(&definition)?.replace("(srgb.icc)", &format!("({icc_path})")),
    )?;
    let pdfa = preview.join("sample-pdfa.pdf");
    if pdfa.exists() {
        fs::remove_file(&pdfa)?;
    }
    let mut gs_command=Command::new(gs);
    gs_command
        .args([
            "-dBATCH",
            "-dNOPAUSE",
            "-sDEVICE=pdfwrite",
            "-dPDFA=3",
            "-dPDFACompatibilityPolicy=1",
            "-sColorConversionStrategy=RGB",
            "-dEmbedAllFonts=true",
        ])
        .arg(format!("-sOutputFile={}", pdfa.display()))
        .arg(format!("--permit-file-read={icc_path}"))
        .arg(&local_definition)
        .arg(pdf);
    hide_window(&mut gs_command);
    let gs_result=gs_command.output()?;
    if !gs_result.status.success() || !pdfa.is_file() {
        return Err(format!(
            "PDF/A-Konvertierung fehlgeschlagen: {}",
            String::from_utf8_lossy(&gs_result.stderr)
        )
        .into());
    }
    let raw_candidate = preview.join("sample-zugferd-raw.pdf");
    if raw_candidate.exists() { fs::remove_file(&raw_candidate)?; }
    let candidate = preview.join("sample-zugferd-candidate.pdf");
    if candidate.exists() {
        fs::remove_file(&candidate)?;
    }
    let mut combine_command=Command::new(&java);
    combine_command
        .arg("-jar")
        .arg(&mustang)
        .args(["--action", "combine", "--source"])
        .arg(&pdfa)
        .arg("--source-xml")
        .arg(xml)
        .arg("--out")
        .arg(&raw_candidate)
        .args([
            "--format",
            "zf",
            "--version",
            "2",
            "--profile",
            "e",
            "--no-additional-attachments",
            "--disable-file-logging",
        ])
        ;
    hide_window(&mut combine_command);
    let combine=combine_command.output()?;
    if !combine.status.success() || !raw_candidate.is_file() {
        return Err(format!(
            "XML-Einbettung fehlgeschlagen: {}",
            String::from_utf8_lossy(&combine.stderr)
        )
        .into());
    }
    normalize_metadata(&java, &mustang, preview, &raw_candidate, &candidate, xml)?;
    fs::remove_file(&raw_candidate)?;
    let mut validation_command=Command::new(&java);
    validation_command
        .arg("-Dfile.encoding=UTF-8")
        .arg("-jar")
        .arg(&mustang)
        .args(["--action", "validate", "--source"])
        .arg(&candidate)
        .arg("--disable-file-logging")
        ;
    hide_window(&mut validation_command);
    let validation=validation_command.output()?;
    let report = String::from_utf8_lossy(&validation.stdout).into_owned();
    fs::write(preview.join("validation.xml"), &report)?;
    if !validation.status.success() || !report_valid(&report)? {
        let detail = report.split("<error ").nth(1)
            .and_then(|entry| entry.split_once('>'))
            .and_then(|(_, rest)| rest.split_once("</error>"))
            .map(|(message, _)| message.chars().take(500).collect::<String>())
            .unwrap_or_else(|| "Bitte Prüfbericht ansehen".into());
        return Err(format!("PDF oder XML ist nicht valide: {detail} (Bericht: preview/validation.xml)").into());
    }
    let output = base.join("output");
    fs::create_dir_all(&output)?;
    let final_pdf = destination.map(Path::to_path_buf).unwrap_or_else(||output.join(format!("{output_stem}.pdf")));
    fs::copy(&candidate, &final_pdf)?;
    fs::copy(
        preview.join("validation.xml"),
        output.join(format!("{output_stem}.validation.xml")),
    )?;
    Ok(final_pdf)
}

#[cfg(test)]
mod tests {
    use super::report_valid;
    #[test]
    fn requires_all_three_valid_summaries() {
        assert!(report_valid("<validation><pdf><summary status=\"valid\"/></pdf><xml><summary status=\"valid\"/></xml><summary status=\"valid\"/></validation>").unwrap());
        assert!(!report_valid("<validation><pdf><summary status=\"invalid\"/></pdf><xml><summary status=\"valid\"/></xml><summary status=\"valid\"/></validation>").unwrap());
    }
}
