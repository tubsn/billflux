use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, path::Path, process::{Command, Stdio}, time::{Duration, Instant}};

const CALLBACK: &str = r#"<script>window.addEventListener('invoice-layout-ready',()=>{const request=new XMLHttpRequest();request.open('POST','./ready',false);request.setRequestHeader('Content-Type','text/html;charset=utf-8');request.send(document.documentElement.outerHTML);},{once:true});</script>"#;

fn reply(stream:&mut TcpStream,status:&str,content_type:&str,body:&[u8])->Result<(),String>{
    write!(stream,"HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",body.len()).map_err(|e|e.to_string())?;
    stream.write_all(body).map_err(|e|e.to_string())
}

fn request(stream:&mut TcpStream)->Result<(String,String,Vec<u8>),String>{
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;
    let mut bytes=Vec::new();let mut buffer=[0;8192];
    let header_end=loop {
        let count=stream.read(&mut buffer).map_err(|e|e.to_string())?;
        if count==0{return Err("Leere Browseranfrage".into());}
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(end)=bytes.windows(4).position(|part|part==b"\r\n\r\n"){break end+4;}
        if bytes.len()>16384{return Err("Browseranfrage ist zu groß".into());}
    };
    let header=String::from_utf8_lossy(&bytes[..header_end]);
    let mut first=header.lines().next().unwrap_or("").split_whitespace();
    let method=first.next().unwrap_or("").to_string();
    let path=first.next().unwrap_or("").to_string();
    let length=header.lines().filter_map(|line|line.split_once(':')).find(|(name,_)|name.eq_ignore_ascii_case("content-length"))
        .map(|(_,value)|value.trim().parse::<usize>()).transpose().map_err(|e|e.to_string())?.unwrap_or(0);
    if length>16*1024*1024{return Err("Rechnungsvorschau ist zu groß".into());}
    while bytes.len()<header_end+length {
        let count=stream.read(&mut buffer).map_err(|e|e.to_string())?;
        if count==0{return Err("Browseranfrage wurde unterbrochen".into());}
        bytes.extend_from_slice(&buffer[..count]);
    }
    Ok((method,path,bytes[header_end..header_end+length].to_vec()))
}

// Die Seite meldet ihren fertigen DOM über Loopback zurück. Das funktioniert
// auch dann, wenn Chromes Windows-Starter keine Konsolenausgabe weiterreicht.
pub fn prepare(chrome:&Path,html_path:&Path,profile:&Path,token:&str)->Result<String,String>{
    let listener=TcpListener::bind("127.0.0.1:0").map_err(|e|e.to_string())?;
    listener.set_nonblocking(true).map_err(|e|e.to_string())?;
    let prefix=format!("/{token}/");
    let url=format!("http://127.0.0.1:{}{prefix}current.html",listener.local_addr().map_err(|e|e.to_string())?.port());
    let source=fs::read_to_string(html_path).map_err(|e|e.to_string())?.replace("</head>",&format!("{CALLBACK}</head>"));
    let folder=html_path.parent().ok_or("Vorlagenverzeichnis fehlt")?;
    let mut command=Command::new(chrome);
    command.args(["--headless","--disable-gpu","--no-sandbox","--no-first-run","--dump-dom","--virtual-time-budget=10000"])
        .arg(format!("--user-data-dir={}",profile.display())).arg(url)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child=command.spawn().map_err(|e|e.to_string())?;
    let start=Instant::now();
    let result=(||{
        while start.elapsed()<Duration::from_secs(45){
            let mut stream=match listener.accept(){
                Ok((stream,_))=>stream,
                Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>{std::thread::sleep(Duration::from_millis(20));continue;},
                Err(error)=>return Err(error.to_string()),
            };
            let Ok((method,path,body))=request(&mut stream) else{continue;};
            let Some(resource)=path.strip_prefix(&prefix) else{reply(&mut stream,"404 Not Found","text/plain",b"")?;continue;};
            if method=="POST" && resource=="ready" {
                let rendered=String::from_utf8(body).map_err(|e|e.to_string())?;
                if !rendered.contains("data-pagination=\"ready\""){reply(&mut stream,"400 Bad Request","text/plain",b"")?;continue;}
                reply(&mut stream,"200 OK","text/plain",b"OK")?;
                return Ok(format!("<!doctype html>\n{}",rendered.replace(CALLBACK,"")));
            }
            if method!="GET" {reply(&mut stream,"405 Method Not Allowed","text/plain",b"")?;continue;}
            if resource=="current.html" {reply(&mut stream,"200 OK","text/html;charset=utf-8",source.as_bytes())?;continue;}
            let content_type=match resource {
                "style.css"=>"text/css;charset=utf-8",
                "fonts/Fira_Sans/FiraSans-Regular.ttf"|"fonts/Fira_Sans/FiraSans-Bold.ttf"|
                "fonts/Fira_Sans_Condensed/FiraSansCondensed-Regular.ttf"|"fonts/Fira_Sans_Condensed/FiraSansCondensed-Bold.ttf"=>"font/ttf",
                _=>{reply(&mut stream,"404 Not Found","text/plain",b"")?;continue;},
            };
            match fs::read(folder.join(resource)) {
                Ok(content)=>reply(&mut stream,"200 OK",content_type,&content)?,
                Err(_)=>reply(&mut stream,"404 Not Found","text/plain",b"")?,
            }
        }
        Err("Die Rechnungsvorlage hat den Seitenaufbau nicht innerhalb von 45 Sekunden abgeschlossen.".into())
    })();
    // Nur der eigens gestartete Renderer wird beendet, niemals ein Benutzerbrowser.
    let _=child.kill();let _=child.wait();
    result
}
