const {chromium}=require('playwright');
const fs=require('node:fs');
const path=require('node:path');
const {pathToFileURL}=require('node:url');
const {spawnSync}=require('node:child_process');
const assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const output=path.join(root,'tmp/pdfs');
const chrome=path.join(root,'dist/Billflux/vendor/chrome-headless-shell/chrome-headless-shell.exe');
const template=fs.readFileSync(path.join(root,'templates/standard/invoice.html'),'utf8');
const script=`<script>${fs.readFileSync(path.join(root,'src/pagination.js'),'utf8')}</script>`;
const css=fs.readFileSync(path.join(root,'templates/standard/style.css'),'utf8').replaceAll('url("fonts/',`url("${pathToFileURL(path.join(root,'dist/Billflux/preview/fonts/')).href}`);
const fixture={logo:'<span>art</span>Messengers.de',seller_name:'artMessengers.de',seller_street:'Musterstraße 12',seller_city:'10115 Berlin',seller_email:'rechnung@example.invalid',seller_phone:'+49 000 000000',seller_website:'example.invalid',seller_tax_number:'00/000/00000',seller_registration_line:'<br>W-IdNr: DE123456789-00001',buyer_name:'Beispielkunde GmbH<br>Einkauf',buyer_street:'Beispielweg 5',buyer_city:'20095 Hamburg',number:'2026-0002',date:'2026-10-07',subject:'Layoutprüfung',net:'500,00',gross:'595,00',tax_rows:'<div><span>19% MwSt.:</span><span>95,00 €</span></div>',account_holder:'Beispielinhaber',payment_reference:'2026-0002',iban:'DE00 0000 0000 0000 0000 00',bic_entry:'<dt>BIC:</dt><dd>TESTDE00XXX</dd>',bic_line:'<br>BIC: TESTDE00XXX',payment_note:'Ich bedanke mich für die Zusammenarbeit.',rows:Array.from({length:5},(_,i)=>`<tr><td>${i+1}</td><td class="item"><strong>Leistungsposition ${i+1}</strong><br>Abstimmung und Umsetzung</td><td class="number">19 %</td><td class="number">100,00 €</td><td class="number">1 h</td><td class="number">100,00 €</td></tr>`).join('')};
const source=process.argv[2]?fs.readFileSync(path.resolve(process.argv[2]),'utf8'):template.replace(/\{\{(\w+)\}\}/g,(_,key)=>fixture[key]||'');
const html=source.replace(/<script>[\s\S]*?<\/script>/g,'').replace('</body>',`${script}</body>`).replace('<link rel="stylesheet" href="style.css">',`<style>${css}</style>`);
fs.mkdirSync(output,{recursive:true});
const input=path.join(output,'layout-input.html');fs.writeFileSync(input,html);
function run(args){const result=spawnSync(chrome,args,{windowsHide:true,encoding:'utf8',maxBuffer:20*1024*1024});assert.equal(result.status,0,result.stderr);return result.stdout;}
(async()=>{
 const flags=['--headless','--disable-gpu','--no-sandbox','--no-first-run',`--user-data-dir=${path.join(output,'layout-chrome-profile')}`];
 const rendered=run([...flags,'--dump-dom','--virtual-time-budget=2000',pathToFileURL(input).href]);
 assert(rendered.includes('data-pagination="ready"'),'Export must have completed pagination');
 assert(!rendered.includes('<script>'),'Print document must be static');
 const staticFile=path.join(output,'layout-static.html');fs.writeFileSync(staticFile,rendered);
 const pdf=path.join(output,'layout-regression.pdf');
 run([...flags,'--no-pdf-header-footer',`--print-to-pdf=${pdf}`,pathToFileURL(staticFile).href]);
 assert(fs.statSync(pdf).size>10000);
 const browser=await chromium.launch({executablePath:chrome,headless:true});
 const page=await browser.newPage({viewport:{width:794,height:1123}});
 await page.goto(pathToFileURL(staticFile).href);await page.evaluate(()=>document.fonts.ready);
 const geometry=await page.evaluate(()=>[...document.querySelectorAll('.sheet')].map(sheet=>{
   const bounds=sheet.getBoundingClientRect(), main=sheet.querySelector('main').getBoundingClientRect(),footer=sheet.querySelector('footer')?.getBoundingClientRect();
   return {left:main.left-bounds.left,right:bounds.right-main.right,bottom:bounds.bottom-main.bottom,footerBottom:footer?bounds.bottom-footer.bottom:null,overlap:footer?main.bottom-footer.top:0};
 }));
 for(const sheet of geometry){assert(sheet.left>75&&sheet.right>75,JSON.stringify(sheet));assert(sheet.bottom>=55,JSON.stringify(sheet));assert(sheet.overlap<1,JSON.stringify(sheet));}
 assert(Math.abs(geometry.at(-1).footerBottom-15*96/25.4)<2);
 const pdfPages=Number(spawnSync('pdfinfo',[pdf],{encoding:'utf8'}).stdout.match(/Pages:\s+(\d+)/)[1]);
 assert.equal(pdfPages,geometry.length,'No extra blank or overflowing print pages');
 await page.screenshot({path:path.join(output,'layout-page.png'),fullPage:true});
 await page.close();
 const ui=await browser.newPage({viewport:{width:2551,height:1296}});
 await ui.addInitScript(({html})=>{
   const party={name:'artMessengers.de',contact:'',street:'Straße 1',city:'12345 Ort',country:'DE',email:'',phone:'',website:'',tax_number:'123',vat_id:'',economic_id:''};
   const workspace={invoice_id:1,company_id:1,customer_id:0,seller:party,buyer:{...party,name:'ZVW'},number:'2026-0002',date:'2026-10-07',service_date:'',due_date:'',subject:'Rechnung für Ai Buddy Update',account_holder:'',iban:'',bic:'',bank_name:'',payment_note:'',payment_reference:'',payment_reference_prefix:'',items:[{description:'Installation Ai Buddy',detail:'Hetzner Server',quantity:4,unit:'Stunden',unit_price_cents:12500,vat_percent:19}]};
   window.previewCalls=0;
   window.__TAURI__={core:{invoke:async(command,args)=>{
     if(command==='preview_invoice'){window.previewCalls++;await new Promise(resolve=>setTimeout(resolve,220));return html.replace('</title>',` ${args.workspace.subject}</title>`);}
     if(command==='load_app')return {workspace,companies:[{id:1,party,template:'standard'}],customers:[],invoices:[],templates:['standard'],settings:{recent_count:5,due_days:14},active_company_id:1,status:'draft'};
     if(command==='calculate')return {lines:[{net_cents:50000}],taxes:[],net_cents:50000,tax_cents:9500,gross_cents:59500};
     throw Error(command);
   }}};
 },{html});
 await ui.goto(pathToFileURL(path.join(root,'ui/index.html')).href);
 await ui.waitForFunction(()=>document.querySelector('#preview iframe[data-current]')?.contentDocument?.documentElement.dataset.pagination==='ready');
 await ui.evaluate(()=>{window.originalPreview=document.querySelector('#preview iframe[data-current]');window.previewGaps=0;new MutationObserver(()=>{if(!document.querySelector('#preview iframe[data-current]'))window.previewGaps++;}).observe(document.getElementById('preview'),{childList:true});});
 const beforeCalls=await ui.evaluate(()=>window.previewCalls);
 await ui.locator('[data-path=subject]').fill('A');await ui.locator('[data-path=subject]').fill('Aktualisiert');
 assert(await ui.evaluate(()=>document.querySelector('#preview iframe[data-current]')===window.originalPreview));
 await ui.waitForFunction(()=>document.querySelector('#preview iframe[data-current]')?.contentDocument?.title.includes('Aktualisiert'));
 assert.equal(await ui.evaluate(()=>window.previewGaps),0,'Visible preview must not disappear during updates');
 assert.equal(await ui.evaluate(()=>window.previewCalls),beforeCalls+1,'Typing should be debounced');
 for(const width of [2551,1440,1100]){
   await ui.setViewportSize({width,height:1296});
   await ui.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
   const size=await ui.evaluate(()=>{const p=document.querySelector('#preview'),f=p.querySelector('iframe'),body=f.contentDocument.body;return {surface:p.getBoundingClientRect().width,frame:f.getBoundingClientRect().width,height:p.clientHeight,expected:body.getBoundingClientRect().height*p.clientWidth/794,scroll:f.contentDocument.documentElement.scrollWidth,viewport:f.contentWindow.innerWidth};});
   assert(Math.abs(size.surface-size.frame)<2,JSON.stringify(size));
   assert(Math.abs(size.height-size.expected)<3,JSON.stringify(size));
   assert(size.scroll<=size.viewport+1,JSON.stringify(size));
 }
 await ui.setViewportSize({width:2551,height:1296});
 await ui.screenshot({path:path.join(output,'layout-ui.png')});
 await browser.close();console.log(`Layout verified: ${pdfPages} A4 pages, margins, final footer, static Chrome export and responsive preview.`);
})().catch(error=>{console.error(error);process.exit(1)});
