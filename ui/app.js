const invoke = window.__TAURI__?.core?.invoke;
const euro = cents => new Intl.NumberFormat('de-DE', {style:'currency',currency:'EUR'}).format((cents || 0) / 100);
const escapeHtml = value => String(value ?? '').replace(/[&<>"']/g, char => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
const emptyParty = () => ({name:'',alternative_name:'',contact:'',street:'',city:'',country:'DE',email:'',phone:'',website:'',tax_number:'',vat_id:'',economic_id:''});
const newItem = () => ({description:'',detail:'',quantity:1,unit:'Stunden',unit_price_cents:appData.settings?.hourly_rate_cents||0,vat_percent:19});
const defaultData = () => ({seller:{...emptyParty(),name:'Musterfirma'},buyer:emptyParty(),number:new Date().getFullYear()+'-0001',payment_reference_prefix:'',payment_reference:'',customer_id:0,date:new Date().toISOString().slice(0,10),service_date:'',service_month:false,due_date:'',subject:'',subject_prefix:appData?.settings?.subject_prefix??'Rechnung',account_holder:'',iban:'',bic:'',bank_name:'',payment_note:'Ich bedanke mich für die Zusammenarbeit.',items:[newItem()]});
let appData = {companies:[],invoices:[],customers:[],templates:['example','standard'],settings:{due_days:14,recent_count:5},active_company_id:1,status:'draft'};
let data = defaultData();
let companyDraft = null;
let dirty = false;
let totals = {lines:[],taxes:[],net_cents:0,tax_cents:0,gross_cents:0};
let calculationSequence = 0;

function getPath(path) { return path.split('.').reduce((object, key) => object?.[key], data); }
function setPath(path, value) { const keys=path.split('.'); const final=keys.pop(); const object=keys.reduce((current,key)=>current[key],data); object[final]=value; }
function setStatus(message) { const text=message==='Bereit'?(page==='invoice'?(appData.status==='issued'?'Ausgestellt':'Entwurf'):''):message;const status=document.getElementById('save-status');status.textContent=text;status.classList.toggle('hidden',!text); }
function fillFields() { document.querySelectorAll('[data-path]').forEach(input => {const path=input.dataset.path,value=path.startsWith('seller.')?(companyDraft?.party?.[path.slice(7)]??''):(path==='due_date'?effectiveDue():getPath(path)??'');if(input.type==='checkbox')input.checked=Boolean(value);else input.value=value;});const city=splitCompanyCity(data.buyer.city);document.querySelectorAll('[data-buyer-address]').forEach(input=>input.value=city[input.dataset.buyerAddress]); }
function centsFromInput(value) { const normalized=String(value).trim().replace(',', '.'); return Math.round((Number(normalized)||0)*100); }
const unitLabel = (unit, quantity) => ['Stunden','Stunde','h'].includes(unit)?'h':['Tage','Tag'].includes(unit)?(quantity===1?'Tag':'Tage'):unit;
function itemMarkup(item,index) { return `<div class="item-card" data-index="${index}"><div class="item-top"><button type="button" class="drag-handle" data-drag="${index}" aria-label="Position ${index+1} verschieben" title="Ziehen oder mit Pfeiltasten verschieben">⠿</button><strong>Position ${index+1}</strong><button type="button" class="remove-item ${data.items.length>1?'':'hidden'}" data-remove="${index}" aria-label="Position ${index+1} entfernen">Entfernen ×</button></div><div class="item-fields"><label class="description">Beschreibung<input data-item="${index}" data-key="description" value="${escapeHtml(item.description)}" placeholder="Leistung oder Produkt"></label><label class="detail">Details<textarea rows="2" data-item="${index}" data-key="detail" placeholder="Weitere Angaben">${escapeHtml(item.detail)}</textarea></label><label>Menge<input type="text" inputmode="decimal" data-item="${index}" data-key="quantity" value="${String(item.quantity).replace('.',',')}" ${item.unit==='Pauschal'?'disabled':''}></label><label>Einheit<select data-item="${index}" data-key="unit">${['Stück','Stunden','Tage','Pauschal'].map(unit=>`<option value="${unit}" ${item.unit===unit?'selected':''}>${unitLabel(unit,item.quantity)}</option>`).join('')}</select></label><label>Einzelpreis €<input type="number" min="0" step="0.01" data-item="${index}" data-key="unit_price_cents" value="${(item.unit_price_cents/100).toFixed(2)}"></label><label>USt. %<select data-item="${index}" data-key="vat_percent">${[0,7,19].map(rate=>`<option value="${rate}" ${item.vat_percent===rate?'selected':''}>${rate} %</option>`).join('')}</select></label></div></div>`; }
function renderItems() { document.getElementById('items').innerHTML=data.items.map(itemMarkup).join(''); }
function safe(value, fallback='—') { return escapeHtml(value || fallback); }
function paymentReference(){if(data.payment_reference?.trim())return data.payment_reference;return (data.payment_reference_prefix||'')+data.number;}
function sellerPostalCity(seller) { const country=String(seller.country||'').trim().toUpperCase(); const prefix=['DE','DEU','DEUTSCHLAND'].includes(country)?'D':country; return prefix&&seller.city?`${prefix}-${seller.city}`:seller.city; }
function renderLegacyPreview() {
  const seller=data.seller, buyer=data.buyer;
  document.getElementById('preview').innerHTML=`<div class="paper-header"><div class="paper-logo">${seller.name.toLowerCase()==='artmessengers.de'?'<strong>art</strong>Messengers.de':safe(seller.name,'Firma')}</div><div class="paper-contact">${safe(seller.alternative_name||seller.name)}<br>${safe(seller.street)}<br>${safe(sellerPostalCity(seller))}<br>${safe(seller.email)}</div></div><div class="paper-address"><div><small>● ${safe(seller.name)}</small><strong>${safe(buyer.name,'Empfänger')}</strong>${buyer.contact?`<br>${escapeHtml(buyer.contact)}`:''}<br>${safe(buyer.street)}<br>${safe(buyer.city)}<br>${safe(buyer.country,'DE')}</div><div class="meta"><strong>${safe(data.subject_prefix)} ${safe(data.number)}</strong><br>Datum: ${safe(data.date)}<br>${data.service_date?`Leistungsdatum: ${safe(data.service_date)}<br>`:''}${data.due_date?`Fällig: ${safe(data.due_date)}`:''}</div></div><div class="paper-subject">${safe([data.subject_prefix,data.number].filter(Boolean).join(' '))}${data.subject?' - '+escapeHtml(data.subject):''}</div><table><thead><tr><th>Pos.</th><th>Beschreibung</th><th>Menge</th><th>Einzelpreis</th><th>Gesamt</th></tr></thead><tbody>${data.items.map((item,index)=>`<tr><td>${index+1}</td><td><strong>${safe(item.description,'Neue Position')}</strong><small>${escapeHtml(item.detail)}</small></td><td>${item.unit==='Pauschal'?'—':`${String(item.quantity).replace('.',',')} ${escapeHtml(unitLabel(item.unit,item.quantity))}`}</td><td>${euro(item.unit_price_cents)}</td><td>${euro(totals.lines[index]?.net_cents)}</td></tr>`).join('')}</tbody></table><div class="paper-totals"><div><span>Netto</span><span>${euro(totals.net_cents)}</span></div>${totals.taxes.map(tax=>`<div><span>USt. ${tax.rate} %</span><span>${euro(tax.tax_cents)}</span></div>`).join('')}<div class="grand"><span>Gesamtbetrag</span><span>${euro(totals.gross_cents)}</span></div></div><div class="paper-payment"><p>Bitte überweisen Sie den Betrag${data.due_date?` bis zum ${safe(data.due_date)}`:''} auf das unten genannte Konto.</p><div>Inhaber: ${safe(data.account_holder,seller.name)}<br>Verwendung: ${escapeHtml(paymentReference())}<br>IBAN: ${safe(data.iban)}</div>${data.payment_note?`<p class="paper-note">${escapeHtml(data.payment_note)}</p>`:''}</div><div class="paper-footer"><div>${safe(seller.alternative_name||seller.name)}<br>${safe(seller.street)}<br>${safe(sellerPostalCity(seller))}</div><div>${safe(seller.email)}<br>${safe(seller.phone)}<br>${safe(seller.website)}</div><div>${safe(data.bank_name)}<br>IBAN: ${safe(data.iban)}${data.bic?`<br>BIC: ${escapeHtml(data.bic)}`:''}</div></div>`;
}
let previewSequence=0;
let previewObserver;
let previewTimer;
let pendingPreviewFrame;
let displayedPreviewHtml='';
function sizePreview(){
  const frame=document.querySelector('#preview iframe[data-current]');if(!frame||!frame.contentDocument)return;
  const width=794,height=Math.ceil(Math.max(1123,frame.contentDocument.body?.getBoundingClientRect().height||0));
  const scale=document.getElementById('preview').clientWidth/width;
  frame.style.width=width+'px';frame.style.height=height+'px';frame.style.transform=`scale(${scale})`;
  document.getElementById('preview').style.height=Math.ceil(height*scale)+'px';
}
window.addEventListener('resize',sizePreview);
function preview(){
  if(!invoke){renderLegacyPreview();return;}
  const sequence=++previewSequence;
  clearTimeout(previewTimer);pendingPreviewFrame?.remove();pendingPreviewFrame=null;
  previewTimer=setTimeout(async()=>{
    try{
      const html=await invoke('preview_invoice',{workspace:structuredClone(data)});
      if(sequence!==previewSequence||html===displayedPreviewHtml)return;
      const surface=document.getElementById('preview');
      const frame=document.createElement('iframe');frame.title='Rechnungsvorschau';frame.setAttribute('scrolling','no');
      frame.className='preview-pending';frame.setAttribute('aria-hidden','true');
      frame.style.width='794px';frame.style.height='1123px';pendingPreviewFrame=frame;
      const publish=()=>{
        if(sequence!==previewSequence||frame.contentDocument?.documentElement.dataset.pagination!=='ready')return;
        requestAnimationFrame(()=>{
          if(sequence!==previewSequence||frame.hasAttribute('data-current'))return;
          previewObserver?.disconnect();
          for(const child of [...surface.children])if(child!==frame)child.remove();
          frame.dataset.current='';frame.classList.remove('preview-pending');frame.removeAttribute('aria-hidden');
          pendingPreviewFrame=null;displayedPreviewHtml=html;
          sizePreview();previewObserver=new ResizeObserver(sizePreview);previewObserver.observe(frame.contentDocument.body);
        });
      };
      frame.onload=()=>{if(sequence!==previewSequence)return;frame.contentWindow.addEventListener('invoice-layout-ready',publish);frame.contentDocument.fonts.ready.then(publish);publish();};
      frame.srcdoc=html;surface.append(frame);
    }catch(error){if(sequence===previewSequence&&!surfaceHasPreview())renderLegacyPreview();}
  },160);
}
function surfaceHasPreview(){return Boolean(document.querySelector('#preview iframe[data-current]'));}
async function recalculate() {
  const sequence=++calculationSequence;
  try { const result=invoke ? await invoke('calculate',{workspace:data}) : localCalculation(); if(sequence===calculationSequence){totals=result;preview();} }
  catch(error) { setStatus(String(error)); }
}
function localCalculation(){const lines=data.items.map(item=>({net_cents:Math.round(item.quantity*item.unit_price_cents)}));const groups={};lines.forEach((line,index)=>{const rate=data.items[index].vat_percent;groups[rate]=(groups[rate]||0)+line.net_cents});const taxes=Object.entries(groups).map(([rate,basis_cents])=>({rate:Number(rate),basis_cents,tax_cents:Math.round(basis_cents*Number(rate)/100)}));const net_cents=lines.reduce((sum,line)=>sum+line.net_cents,0),tax_cents=taxes.reduce((sum,tax)=>sum+tax.tax_cents,0);return {lines,taxes,net_cents,tax_cents,gross_cents:net_cents+tax_cents};}
let page = 'invoice';
let restoringNavigation=false;
let autosaveTimer;
let saving = null;
let revision = 0;
let savedRevision = 0;
const todayLocal = () => { const d=new Date(); return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`; };
const activeCompany = () => appData.companies.find(c=>c.id===appData.active_company_id);
const activeInvoices = () => appData.invoices.filter(row=>row.company_id===appData.active_company_id);
const activeCustomers = () => appData.customers;
let invoiceSort=null;
let customerFieldsExpanded=false;
let invoiceFieldsExpanded=false;
function effectiveDue(days=appData.settings?.due_days||14) {
  if(data.due_date) return data.due_date;
  if(!data.date) return '';
  const date=new Date(`${data.date}T12:00:00Z`);
  if(Number.isNaN(date.valueOf())) return '';
  date.setUTCDate(date.getUTCDate()+Number(days));
  return date.toISOString().slice(0,10);
}
function numberAvailable() {
  const field=document.querySelector('[data-path="number"]');
  const taken=appData.invoices.some(row=>row.number===data.number&&row.id!==data.invoice_id);
  field.setCustomValidity(taken?'Diese Rechnungsnummer ist bereits vergeben':'');
  field.classList.toggle('invalid',taken);
  return !taken;
}
function scheduleSave() {
  clearTimeout(autosaveTimer);
  autosaveTimer=setTimeout(()=>flushSave().catch(()=>{}),700);
}
function markDirty() {
  dirty=true;revision++;
  setStatus('Speichert …');
  scheduleSave();
}
async function flushSave() {
  clearTimeout(autosaveTimer);
  if([...document.querySelectorAll('[data-key=quantity]')].some(field=>!field.reportValidity())){throw new Error('Menge prüfen');}
  if(!dirty||!invoke) return;
  if(!data.number.trim()||!numberAvailable()){setStatus('Rechnungsnummer prüfen');throw new Error('Rechnungsnummer prüfen');}
  if(saving){await saving; if(dirty)return flushSave();return;}
  const version=revision;
  saving=invoke('save_workspace',{workspace:structuredClone(data)});
  try {
    const result=await saving;
    data.invoice_id=result.invoice_id;
    if(page==='invoice'&&!restoringNavigation)history.replaceState(navigationState(),'');
    if(version===revision){dirty=false;savedRevision=version;setStatus('Gespeichert');}
    const fresh=await invoke('load_app');
    appData={...fresh,workspace:data,status:data.invoice_id===0?'draft':fresh.status};
    renderMenu();renderSearch();numberAvailable();
  } catch(error) {
    setStatus(`Speichern fehlgeschlagen: ${error}`);
    throw error;
  } finally { saving=null; }
  if(dirty) scheduleSave();
}
function renderMenu() {
  const firm=activeCompany();
  document.getElementById('current-company-name').textContent=firm?.party.name||'Firma wählen';
  document.getElementById('company-options').innerHTML=appData.companies.map(company=>`<button class="company-option ${company.id===appData.active_company_id?'selected':''}" data-switch-company="${company.id}"><span class="company-avatar">${escapeHtml(company.party.name.slice(0,1).toUpperCase())}</span><span>${escapeHtml(company.party.name)}</span>${company.id===appData.active_company_id?'<b>✓</b>':''}</button>`).join('');
  document.getElementById('recent-invoices').innerHTML=activeInvoices().slice(0,Number(appData.settings?.recent_count||5)).map(row=>`<button data-open="${row.id}" class="recent-row ${row.id===data.invoice_id&&page==='invoice'?'active':''}"><strong>${escapeHtml(row.number)}</strong><small title="${escapeHtml([row.buyer,row.subject].filter(Boolean).join(' · '))}">${escapeHtml([row.buyer,row.subject].filter(Boolean).join(' · ')||'Entwurf')}</small></button>`).join('')||'<span class="sidebar-empty">Noch keine Rechnungen</span>';
  document.querySelectorAll('.nav').forEach(button=>button.classList.toggle('active',button.dataset.page===page));
}
function renderInvoices(rows=activeInvoices(),target='invoice-list') {
  const columns=[['number','Rechnungsnummer'],['date','Datum'],['buyer','Empfänger'],['subject','Betreff'],['status','Status']];
  document.getElementById(target).innerHTML=`<div class="invoice-table-wrap"><table class="invoice-table"><thead><tr>${columns.map(([key,label])=>`<th aria-sort="${invoiceSort?.key===key?(invoiceSort.direction===1?'ascending':'descending'):'none'}"><button data-sort="${key}">${label} ${invoiceSort?.key===key?(invoiceSort.direction===1?'↑':'↓'):'↕'}</button></th>`).join('')}<th>Aktionen</th></tr></thead><tbody>${rows.length?rows.map(row=>`<tr><td><button class="invoice-open" data-open="${row.id}">${escapeHtml(row.number)}</button></td><td class="invoice-date">${escapeHtml(row.date||'—')}</td><td>${escapeHtml(row.buyer||'Ohne Empfänger')}</td><td class="invoice-subject">${escapeHtml(row.subject||'Ohne Betreff')}</td><td><span class="invoice-status ${row.status==='issued'?'issued':''}">${row.status==='issued'?'Ausgestellt':'Entwurf'}</span></td><td><div class="row-actions"><button class="button" data-duplicate="${row.id}" aria-label="Rechnung ${escapeHtml(row.number)} duplizieren">Duplizieren</button><button class="button danger" data-delete="${row.id}" aria-label="Rechnung ${escapeHtml(row.number)} löschen">Löschen</button></div></td></tr>`).join(''):'<tr><td colspan="6" class="empty-state">Keine Rechnungen gefunden.</td></tr>'}</tbody></table></div>`;
}
function renderSearch() {
  const query=document.getElementById('search-input').value.trim().toLocaleLowerCase('de-DE');
  const ranked=activeInvoices().map(row=>({row,rank:[row.number,row.subject,row.buyer].findIndex(value=>String(value||'').toLocaleLowerCase('de-DE').includes(query))})).filter(result=>result.rank>=0);
  const collator=new Intl.Collator('de-DE',{numeric:true,sensitivity:'base'});
  ranked.sort((a,b)=>invoiceSort?invoiceSort.direction*collator.compare(String(invoiceSort.key==='status'?(a.row.status==='issued'?'Ausgestellt':'Entwurf'):a.row[invoiceSort.key]||''),String(invoiceSort.key==='status'?(b.row.status==='issued'?'Ausgestellt':'Entwurf'):b.row[invoiceSort.key]||'')):a.rank-b.rank);
  renderInvoices(ranked.map(result=>result.row));
}
function selectedCustomer(){
  return activeCustomers().find(customer=>data.customer_id?customer.id===data.customer_id:['name','contact','street','city','country','email','vat_id'].every(key=>(customer.party[key]||'')===(data.buyer[key]||'')));
}
function syncCustomerFields(){
  const selected=Boolean(document.getElementById('invoice-customer').value);
  document.querySelectorAll('.customer-basic').forEach(field=>field.classList.toggle('hidden',selected&&!customerFieldsExpanded));
  document.getElementById('customer-extra-fields').classList.toggle('hidden',!customerFieldsExpanded);
  document.getElementById('save-customer-button').classList.toggle('hidden',selected&&!customerFieldsExpanded);
  const toggle=document.getElementById('toggle-customer-fields');toggle.setAttribute('aria-expanded',String(customerFieldsExpanded));toggle.textContent=customerFieldsExpanded?'weniger Felder':'alle Felder';
}
function renderCustomers() {
  const rows=activeCustomers();
  document.getElementById('customer-list').innerHTML=rows.length?rows.map(customer=>`<div class="customer-row"><span><strong>${escapeHtml(customer.party.name)}</strong><small>${escapeHtml([customer.party.contact,customer.party.street,customer.party.city].filter(Boolean).join(' · '))}</small></span><button class="button" data-edit-customer="${customer.id}">Bearbeiten</button><button class="button" data-customer-invoice="${customer.id}">Rechnung erstellen</button><button class="button danger" data-delete-customer="${customer.id}">Löschen</button></div>`).join(''):'<p class="empty-state">Noch keine Kunden gespeichert.</p>';
  const match=selectedCustomer();
  document.getElementById('invoice-customer').innerHTML='<option value="">Neuer Kunde</option>'+rows.map(customer=>`<option value="${customer.id}">${escapeHtml(customer.id===match?.id?data.buyer.name:customer.party.name)}</option>`).join('');
  document.getElementById('invoice-customer').value=match?String(match.id):'';
  document.getElementById('save-customer-button').textContent=match?'Als neuen Kunden speichern':'Kunden speichern';
  syncCustomerFields();
}
const templateLabel=name=>name==='standard'?'Artmessengers':name==='example'?'Example':name;
let selectedTemplate=null;
let templatePreviewSequence=0;
function sizeTemplatePreview(){
  const surface=document.getElementById('template-preview-surface'),frame=surface.querySelector('iframe');
  if(!frame||!frame.contentDocument)return;
  const height=Math.ceil(Math.max(1123,frame.contentDocument.body?.getBoundingClientRect().height||0));
  const scale=surface.clientWidth/794;
  frame.style.width='794px';frame.style.height=height+'px';frame.style.transform=`scale(${scale})`;
  surface.style.height=Math.ceil(height*scale)+'px';
}
window.addEventListener('resize',sizeTemplatePreview);
async function showTemplatePreview(){
  const name=selectedTemplate,sequence=++templatePreviewSequence,surface=document.getElementById('template-preview-surface');
  document.getElementById('template-preview-title').textContent=templateLabel(name);
  surface.style.height='auto';surface.innerHTML='<p class="empty-state">Vorschau wird geladen …</p>';
  try{
    const html=invoke?await invoke('preview_template',{workspace:structuredClone(data),template:name}):null;
    if(sequence!==templatePreviewSequence)return;
    if(!html){surface.innerHTML='<p class="empty-state">Die Vorschau ist in der Desktop-App verfügbar.</p>';return;}
    const frame=document.createElement('iframe');frame.title=`Vorschau ${templateLabel(name)}`;
    frame.onload=()=>{const ready=()=>{if(sequence===templatePreviewSequence)sizeTemplatePreview();};frame.contentWindow.addEventListener('invoice-layout-ready',ready);frame.contentDocument.fonts.ready.then(ready);ready();};
    frame.srcdoc=html;surface.replaceChildren(frame);
  }catch(error){if(sequence===templatePreviewSequence)surface.innerHTML=`<p class="empty-state">${escapeHtml(String(error))}</p>`;}
}
function renderTemplates() {
  const firm=activeCompany();
  if(!appData.templates.includes(selectedTemplate))selectedTemplate=firm?.template||appData.templates[0];
  document.getElementById('template-list').innerHTML=appData.templates.map(name=>`<div class="template-row ${name===selectedTemplate?'selected':''}"><button type="button" class="template-choice" data-view-template="${escapeHtml(name)}"><span class="template-icon">▧</span><span><strong>${escapeHtml(templateLabel(name))}</strong><small>${name===firm?.template?'Aktuelle Vorlage':'HTML · CSS'}</small></span></button><button type="button" class="button ${name===firm?.template?'':'primary'}" data-activate-template="${escapeHtml(name)}" ${name===firm?.template?'disabled':''}>${name===firm?.template?'Aktiv':'Aktivieren'}</button></div>`).join('');
  if(selectedTemplate)showTemplatePreview();
}
function splitCompanyCity(city){const match=String(city||'').trim().match(/^([A-Z0-9-]*\d[A-Z0-9-]*)\s+(.+)$/i);return match?{postal_code:match[1],locality:match[2]}:{postal_code:'',locality:city||''};}
function fillSettings() {
  document.getElementById('last-invoice-number').value=activeInvoices()[0]?.number||'Noch keine Rechnung';
  document.querySelectorAll('[data-setting]').forEach(input=>input.value=input.dataset.setting==='hourly_rate_cents'?((appData.settings?.hourly_rate_cents||0)/100).toFixed(2).replace('.',','):appData.settings?.[input.dataset.setting]??'');
  const city=splitCompanyCity(companyDraft?.party?.city);
  document.querySelectorAll('[data-company-address]').forEach(input=>input.value=city[input.dataset.companyAddress]);
}
function updateReferenceExample(){const prefix=document.querySelector('[data-setting=payment_reference_prefix]').value;document.getElementById('payment-reference-example').textContent='Verwendung: '+prefix+data.number;}
function syncDates(){
  document.getElementById('invoice-extra-fields').classList.toggle('hidden',!invoiceFieldsExpanded);
  const toggle=document.getElementById('toggle-invoice-fields');toggle.setAttribute('aria-expanded',String(invoiceFieldsExpanded));toggle.textContent=invoiceFieldsExpanded?'weniger Felder':'alle Felder';
}
let paymentFieldsExpanded=false;
function syncPayment(){
  const configured=['iban','account_holder','bank_name','bic'].some(key=>Boolean(appData.settings?.[key]));
  document.getElementById('payment-summary').textContent=configured?'Aus den Voreinstellungen übernommen':'Bankverbindung ergänzen';
  document.getElementById('payment-fields').classList.toggle('hidden',!paymentFieldsExpanded);
  const toggle=document.getElementById('toggle-payment-fields');toggle.setAttribute('aria-expanded',String(paymentFieldsExpanded));toggle.textContent=paymentFieldsExpanded?'weniger Felder':'alle Felder';
}
function syncUi() {
  if(!data.items?.length)data.items=[newItem()];
  if(!companyDraft||companyDraft.id!==appData.active_company_id)companyDraft=structuredClone(activeCompany());
  fillFields();renderItems();syncDates();syncPayment();renderMenu();renderInvoices();renderSearch();renderCustomers();renderTemplates();fillSettings();numberAvailable();recalculate();
  const issued=appData.status==='issued';
  document.getElementById('export-button').disabled=false;
  document.getElementById('page-title').textContent=issued?'Rechnung '+data.number:data.invoice_id?'Rechnung bearbeiten':'Neue Rechnung';
}
function showPage(next) {
  clearExportFeedback();
  page=next;setStatus(dirty&&next==='invoice'?'Noch nicht gespeichert':'Bereit');
  recordNavigation();
  if(next==='invoices')renderSearch();
  document.getElementById('create-company-button').classList.toggle('hidden',next!=='settings');
  if(next==='settings'){if(settingsRevision===settingsSavedRevision)fillSettings();updateReferenceExample();}
  for(const id of ['invoice','invoices','customers','templates','settings'])document.getElementById(id+'-page').classList.toggle('hidden',id!==next);
  if(next==='templates')requestAnimationFrame(sizeTemplatePreview);
  const titles={invoice:['RECHNUNG','Rechnung','Deine Rechnungsdaten'],invoices:['ÜBERSICHT','Rechnungen','Rechnungen der aktuellen Firma'],search:['SUCHE','Suche','Rechnungen finden'],customers:['ADRESSBUCH','Kunden','Deine Kunden'],templates:['GESTALTUNG','Templates','Vorlagen der aktuellen Firma'],settings:['VOREINSTELLUNGEN','Einstellungen','Standardwerte für neue Rechnungen'],company:['FIRMA','Firmendaten','Angaben zur aktuellen Firma']};
  const [label,title,subtitle]=titles[next];
  document.getElementById('section-label').textContent=label;
  document.getElementById('page-title').textContent=next==='invoice'?(appData.status==='issued'?'Rechnung '+data.number:data.invoice_id?'Rechnung bearbeiten':'Neue Rechnung'):title;
  document.getElementById('page-subtitle').textContent=next==='settings'?`Einstellungen für ${activeCompany()?.party.name||'die aktuelle Firma'}`:subtitle;
  document.getElementById('export-button').classList.toggle('hidden',next!=='invoice');
  document.getElementById('company-popup').classList.add('hidden');
  document.getElementById('company-switch').setAttribute('aria-expanded','false');
  renderMenu();
}
async function startInvoice(buyer) {
  await flushSettings();
  await flushSave();
  const companyId=appData.active_company_id;
  data=invoke?await invoke('new_invoice',{companyId,date:todayLocal()}):defaultData();
  data.date=todayLocal();
  if(buyer){data.buyer=structuredClone(buyer.party||buyer);data.customer_id=buyer.id||0;}
  customerFieldsExpanded=false;invoiceFieldsExpanded=false;paymentFieldsExpanded=false;
  appData.status='draft';dirty=false;revision=0;savedRevision=0;
  syncUi();showPage('invoice');markDirty();
}
async function openInvoice(id) {
  await flushSettings();
  await flushSave();
  data=await invoke('open_invoice',{id});
  customerFieldsExpanded=false;invoiceFieldsExpanded=false;paymentFieldsExpanded=false;
  appData=await invoke('load_app');
  companyDraft=structuredClone(activeCompany());
  dirty=false;revision=0;savedRevision=0;
  syncUi();showPage('invoice');setStatus(appData.status==='issued'?'Ausgestellt':'Entwurf');
}
async function switchCompany(id) {
  const keepSettings=page==='settings';
  await flushSettings();
  await flushSave();
  await invoke('select_company',{companyId:id});
  appData=await invoke('load_app');
  selectedTemplate=null;
  companyDraft=structuredClone(activeCompany());
  const latest=activeInvoices()[0];
  if(latest){data=await invoke('open_invoice',{id:latest.id});appData=await invoke('load_app');dirty=false;syncUi();showPage('invoices');setStatus('Bereit');}
  else await startInvoice();
  if(keepSettings)showPage('settings');
}
function writeItem(field) {
  const item=data.items[Number(field.dataset.item)];
  const key=field.dataset.key;
  if(key==='quantity'){const quantity=Number(field.value.replace(',','.'));field.setCustomValidity(Number.isFinite(quantity)&&quantity>0?'':'Bitte eine Menge größer als 0 eingeben');if(!field.checkValidity())return;}
  item[key]=key==='unit_price_cents'?centsFromInput(field.value):['quantity','vat_percent'].includes(key)?Number(field.value.replace(',','.')):field.value;
  if(key==='quantity'){const select=field.closest('.item-fields').querySelector('[data-key=unit]');for(const option of select.options)option.textContent=unitLabel(option.value,item.quantity);}
  if(key==='unit'){if(item.unit==='Pauschal')item.quantity=1;renderItems();}
  clearExportFeedback();recalculate();markDirty();
}
document.addEventListener('input',event=>{
  const field=event.target;
  if(field.dataset.setting==='payment_reference_prefix')updateReferenceExample();
  if(field.dataset.setting==='due_days'&&!data.due_date){document.querySelector('[data-path=due_date]').value=effectiveDue(field.value);}
  if(field.dataset.setting!==undefined)queueSettingsSave();
  if(field.dataset.companyAddress){const postal=document.querySelector('[data-company-address=postal_code]').value.trim();const locality=document.querySelector('[data-company-address=locality]').value.trim();companyDraft.party.city=[postal,locality].filter(Boolean).join(' ');queueSettingsSave();}
  if(field.dataset.buyerAddress){const postal=document.querySelector('[data-buyer-address=postal_code]').value.trim();const locality=document.querySelector('[data-buyer-address=locality]').value.trim();data.buyer.city=[postal,locality].filter(Boolean).join(' ');preview();markDirty();}
  clearExportFeedback();
  if(field.dataset.path){
    const path=field.dataset.path;
    if(path.startsWith('seller.')){companyDraft.party[path.slice(7)]=field.value;queueSettingsSave();}
    else{setPath(path,field.type==='checkbox'?field.checked:field.value);if(path==='number')numberAvailable();if(path==='date'&&!data.due_date)document.querySelector('[data-path=due_date]').value=effectiveDue();preview();markDirty();}
  }
  if(field.dataset.item!==undefined)writeItem(field);
});
document.addEventListener('change',event=>{
  const field=event.target;
  clearExportFeedback();
  if(field.dataset.path==='service_month'){data.service_month=field.checked;preview();markDirty();}
  if(field.dataset.item!==undefined)writeItem(field);
  if(field.dataset.path==='service_date'||field.dataset.path==='due_date')syncDates();
});
document.getElementById('items').addEventListener('click',event=>{
  const button=event.target.closest('[data-remove]');if(!button)return;
  if(document.activeElement?.closest('#items'))document.activeElement.blur();
  data.items.splice(Number(button.dataset.remove),1);if(!data.items.length)data.items=[newItem()];
  renderItems();recalculate();markDirty();
});
document.getElementById('add-item').addEventListener('click',()=>{data.items.push(newItem());renderItems();recalculate();markDirty();document.querySelector('.item-card:last-child input').focus();});
document.getElementById('toggle-invoice-fields').addEventListener('click',()=>{invoiceFieldsExpanded=!invoiceFieldsExpanded;syncDates();});
document.getElementById('sidebar-new').addEventListener('click',()=>startInvoice().catch(error=>setStatus(String(error))));
document.querySelectorAll('.nav').forEach(button=>button.addEventListener('click',()=>showPage(button.dataset.page)));
document.getElementById('recent-invoices').addEventListener('click',event=>{const button=event.target.closest('[data-open]');if(button)openInvoice(Number(button.dataset.open)).catch(error=>setStatus(String(error)));});
document.getElementById('invoice-list').addEventListener('click',event=>{
  const button=event.target.closest('button');if(!button)return;
  if(button.dataset.sort){const key=button.dataset.sort;invoiceSort={key,direction:invoiceSort?.key===key?-invoiceSort.direction:1};renderSearch();}
  else if(button.dataset.open)openInvoice(Number(button.dataset.open)).catch(error=>setStatus(String(error)));
  else if(button.dataset.duplicate)duplicateInvoice(Number(button.dataset.duplicate)).catch(error=>setStatus(String(error)));
  else if(button.dataset.delete)requestInvoiceDeletion(Number(button.dataset.delete));
});
document.getElementById('search-input').addEventListener('input',renderSearch);
document.getElementById('company-switch').addEventListener('click',()=>{const popup=document.getElementById('company-popup');popup.classList.toggle('hidden');document.getElementById('company-switch').setAttribute('aria-expanded',String(!popup.classList.contains('hidden')));});
document.getElementById('company-options').addEventListener('click',event=>{const button=event.target.closest('button');if(!button)return;if(button.dataset.page)showPage(button.dataset.page);else switchCompany(Number(button.dataset.switchCompany)).catch(error=>setStatus(String(error)));});
document.getElementById('invoice-customer').addEventListener('change',event=>{
  const customer=activeCustomers().find(row=>row.id===Number(event.target.value));
  clearExportFeedback();data.customer_id=customer?.id||0;data.buyer=customer?structuredClone(customer.party):emptyParty();customerFieldsExpanded=false;
  fillFields();syncCustomerFields();preview();markDirty();
});
document.getElementById('save-customer-button').addEventListener('click',async()=>{
  try{const customer=await invoke('save_customer',{companyId:appData.active_company_id,party:data.buyer});appData.customers.unshift(customer);data.customer_id=customer.id;customerFieldsExpanded=false;renderCustomers();markDirty();setStatus('Kunde gespeichert');}catch(error){setStatus(String(error));}
});
document.getElementById('customer-list').addEventListener('click',event=>{
  const button=event.target.closest('button');if(!button)return;
  if(button.dataset.editCustomer)editCustomer(Number(button.dataset.editCustomer));
  else if(button.dataset.customerInvoice){const customer=activeCustomers().find(row=>row.id===Number(button.dataset.customerInvoice));startInvoice(customer).catch(error=>setStatus(String(error)));}
  else if(button.dataset.deleteCustomer){const id=Number(button.dataset.deleteCustomer);const customer=activeCustomers().find(row=>row.id===id);if(customer&&confirm(`Kunde „${customer.party.name}“ löschen?`)){invoke('delete_customer',{id}).then(()=>{appData.customers=appData.customers.filter(row=>row.id!==id);if(data.customer_id===id){data.customer_id=0;markDirty();}renderCustomers();setStatus('Kunde gelöscht');}).catch(error=>setStatus(String(error)));}}
});
let settingsTimer,settingsRevision=0,settingsSavedRevision=0,settingsSaving=null;
function queueSettingsSave(){settingsRevision++;clearTimeout(settingsTimer);setStatus('Speichert …');settingsTimer=setTimeout(()=>flushSettings().catch(error=>setStatus(String(error))),400);}
async function flushSettings(){
  clearTimeout(settingsTimer);
  if(settingsSaving){await settingsSaving;if(settingsSavedRevision<settingsRevision)return flushSettings();return;}
  if(settingsSavedRevision===settingsRevision)return;
  const hourly=document.querySelector('[data-setting="hourly_rate_cents"]');
  const rate=Number(hourly.value.trim().replace(',','.'));
  hourly.setCustomValidity(Number.isFinite(rate)&&rate>=0&&Number.isSafeInteger(Math.round(rate*100))?'':'Bitte einen gültigen Stundensatz ab 0 eingeben');
  if([...document.querySelectorAll('#settings-page input')].some(input=>!input.checkValidity())){setStatus('Bitte die markierten Einstellungen prüfen');return;}
  const settings={};
  document.querySelectorAll('[data-setting]').forEach(input=>settings[input.dataset.setting]=input.dataset.setting==='hourly_rate_cents'?Math.round(rate*100):['due_days','recent_count'].includes(input.dataset.setting)?Number(input.value):input.value);
  const companyId=appData.active_company_id;
  const party=structuredClone(companyDraft.party);
  const version=settingsRevision;
  try{
    const company=invoke?await (settingsSaving=invoke('save_company_settings',{companyId,party,settings})):{...companyDraft,party};
    if(appData.active_company_id!==companyId)return;
    settingsSavedRevision=version;
    appData.settings=settings;if(settingsRevision===version)companyDraft=company;
    appData.companies=appData.companies.map(row=>row.id===company.id?company:row);
    if(data.company_id===companyId){
      let changed=JSON.stringify(data.seller)!==JSON.stringify(company.party);
      if(data.payment_reference_prefix!==settings.payment_reference_prefix){data.payment_reference_prefix=settings.payment_reference_prefix;changed=true;}
      if(settingsRevision===version)data.seller=structuredClone(company.party);
      for(const key of ['account_holder','iban','bic','bank_name'])if(!data[key]&&settings[key]){data[key]=settings[key];changed=true;}
      if(changed&&settingsRevision===version){fillFields();markDirty();}
    }
    renderMenu();syncPayment();preview();
    document.getElementById('page-subtitle').textContent=`Einstellungen für ${company.party.name}`;
    if(settingsRevision===version)setStatus('Gespeichert');
  }finally{settingsSaving=null;}
  if(settingsSavedRevision<settingsRevision)return flushSettings();
}
document.getElementById('template-list').addEventListener('click',async event=>{
  const choice=event.target.closest('[data-view-template]');
  if(choice){selectedTemplate=choice.dataset.viewTemplate;renderTemplates();return;}
  const button=event.target.closest('[data-activate-template]');
  if(!button||button.disabled)return;
  try{
    const company=structuredClone(activeCompany());company.template=button.dataset.activateTemplate;
    await invoke('save_company',{company});
    appData.companies=appData.companies.map(row=>row.id===company.id?company:row);
    selectedTemplate=company.template;renderTemplates();preview();setStatus('Vorlage aktiviert');
  }catch(error){setStatus(String(error));}
});
document.getElementById('create-company-button').addEventListener('click',()=>{
  document.getElementById('create-company-error').textContent='';document.getElementById('new-company-name').value='';document.getElementById('create-company-dialog').showModal();document.getElementById('new-company-name').focus();
});
document.getElementById('create-company-form').addEventListener('submit',async event=>{
  event.preventDefault();const input=document.getElementById('new-company-name'),name=input.value.trim();if(!name){input.focus();return;}
  event.submitter.disabled=true;
  try{await flushSettings();await flushSave();const created=await invoke('create_company',{name});document.getElementById('create-company-dialog').close();await switchCompany(created.id);showPage('settings');setStatus('Firma angelegt');}
  catch(error){document.getElementById('create-company-error').textContent=String(error);}
  finally{event.submitter.disabled=false;}
});
document.getElementById('export-button').addEventListener('click',async()=>{
  const button=document.getElementById('export-button'),resultPanel=document.getElementById('export-result');
  clearExportFeedback();
  const issues=invoiceIssues();if(issues.length){showInvoiceIssues(issues);return;}
  if(!invoke){resultPanel.textContent='Der Export ist nur in der Desktop-App verfügbar.';resultPanel.classList.remove('hidden');return;}
  button.disabled=true;button.textContent='PDF wird erstellt …';resultPanel.classList.add('hidden');setStatus('PDF wird geprüft …');
  const exportedId=data.invoice_id;
  try{await flushSave();const result=await invoke('export_invoice',{workspace:data});if(!result){setStatus('Export abgebrochen');return;}appData=await invoke('load_app');data=appData.workspace;dirty=false;syncUi();if(page!=='invoice')return;setStatus('Validiert');resultPanel.innerHTML=`<strong>ZUGFeRD-PDF erfolgreich validiert.</strong><br>PDF: ${escapeHtml(result.pdf_path)}<br>Prüfbericht: ${escapeHtml(result.report_path)}`;resultPanel.classList.remove('hidden');}
  catch(error){if(page==='invoice'&&(data.invoice_id===exportedId||exportedId===0)){setStatus('Export fehlgeschlagen');resultPanel.textContent=String(error);resultPanel.classList.add('error');resultPanel.classList.remove('hidden');resultPanel.scrollIntoView({block:'nearest'});}}
  finally{button.disabled=false;button.textContent='PDF Exportieren';}
});
(async()=>{
  try{
    if(invoke){appData=await invoke('load_app');data=appData.workspace;}
    else{data=JSON.parse(localStorage.getItem('billflux-workspace'))||defaultData();data.company_id=1;appData.companies=[{id:1,party:data.seller,template:'example'}];appData.active_company_id=1;}
    companyDraft=structuredClone(activeCompany());
    if(!data.date&&appData.status!=='issued')data.date=todayLocal();
    syncUi();showPage('invoice');setStatus('Bereit');
  }catch(error){setStatus(`Laden fehlgeschlagen: ${error}`);renderItems();recalculate();}
})();


function closeCompanyPopup(){
  document.getElementById('company-popup').classList.add('hidden');
  document.getElementById('company-switch').setAttribute('aria-expanded','false');
}
document.addEventListener('click',event=>{if(!event.target.closest('.company-switch-wrap'))closeCompanyPopup();});
document.addEventListener('keydown',event=>{if(event.key==='Escape')closeCompanyPopup();});
document.getElementById('toggle-customer-fields').addEventListener('click',()=>{customerFieldsExpanded=!customerFieldsExpanded;syncCustomerFields();});
document.getElementById('toggle-payment-fields').addEventListener('click',()=>{paymentFieldsExpanded=!paymentFieldsExpanded;syncPayment();});
function clearExportFeedback(){
  const panel=document.getElementById('export-result');
  if(panel.classList.contains('error'))setStatus(dirty?'Noch nicht gespeichert':'Bereit');
  panel.setAttribute('role','status');panel.classList.add('hidden');panel.classList.remove('error');panel.textContent='';
  document.querySelectorAll('.export-invalid').forEach(field=>field.classList.remove('export-invalid'));
}
function invoiceIssues(){
  const issues=[];
  for(const [path,label] of [['number','Rechnungsnummer'],['date','Rechnungsdatum'],['seller.name','Firmenname'],['seller.street','Firmenanschrift'],['seller.city','Firmenort'],['buyer.name','Kundenname'],['buyer.street','Kundenanschrift'],['buyer.city','Kundenort'],['iban','IBAN']]){
    if(!String(getPath(path)||'').trim())issues.push({path,message:label+' fehlt'+(path.startsWith('seller.')?' (unter Einstellungen → Firmendaten ergänzen)':'')});
  }
  if(!data.seller.vat_id.trim()&&!data.seller.tax_number.trim())issues.push({path:'seller.tax_number',message:'USt-IdNr. oder Steuernummer fehlt (unter Einstellungen → Firmendaten ergänzen)'});
  if(!numberAvailable())issues.push({path:'number',message:'Diese Rechnungsnummer ist bereits vergeben'});
  for(const field of document.querySelectorAll('#invoice-page input')){
    if(!field.checkValidity())issues.push({field,message:field.closest('label').firstChild.textContent.trim()+': '+field.validationMessage});
  }
  data.items.forEach((item,index)=>{
    if(!item.description.trim())issues.push({field:document.querySelector(`[data-item="${index}"][data-key="description"]`),message:`Beschreibung für Position ${index+1} fehlt`});
    if(item.vat_percent===0)issues.push({field:document.querySelector(`[data-item="${index}"][data-key="vat_percent"]`),message:`Position ${index+1}: 0 % Umsatzsteuer wird im Export noch nicht unterstützt`});
  });
  return issues;
}
function showInvoiceIssues(issues){
  const panel=document.getElementById('export-result');
  panel.innerHTML='<strong>Bitte vor dem Export ergänzen oder korrigieren:</strong><ul>'+issues.map(issue=>`<li>${escapeHtml(issue.message)}</li>`).join('')+'</ul>';
  panel.classList.remove('hidden');panel.classList.add('error');panel.setAttribute('role','alert');
  for(const issue of issues){
    const field=issue.field||document.querySelector(`[data-path="${issue.path}"]`);if(!field)continue;
    field.classList.add('export-invalid');
    if(field.closest('#payment-fields')){paymentFieldsExpanded=true;syncPayment();}
    if(field.dataset.path?.startsWith('buyer.')){customerFieldsExpanded=true;syncCustomerFields();}
    if(['service_date','due_date'].includes(field.dataset.path)){invoiceFieldsExpanded=true;syncDates();}
  }
  setStatus('Angaben prüfen');panel.scrollIntoView({block:'nearest'});
}


function moveItem(from,to){
  if(from===to||from<0||to<0||from>=data.items.length||to>=data.items.length)return;
  if(document.activeElement?.closest('#items'))document.activeElement.blur();
  const [item]=data.items.splice(from,1);data.items.splice(to,0,item);
  clearExportFeedback();renderItems();recalculate();markDirty();
}
let draggedItem=null;
const itemContainer=document.getElementById('items');
let dragPointer=null,dropItem=null,dragY=0,dragX=0,dragFrame=null;
function markDrop(x,y){
  dragY=y;dragX=x;
  const cards=[...itemContainer.querySelectorAll('.item-card')];
  const card=cards.find(card=>{const rect=card.getBoundingClientRect();return x>=rect.left&&x<=rect.right&&y>=rect.top&&y<=rect.bottom;});
  dropItem=card?Number(card.dataset.index):null;
  cards.forEach(card=>card.classList.toggle('drop-target',Number(card.dataset.index)===dropItem&&dropItem!==draggedItem));
}
function scrollDrag(){
  if(draggedItem===null)return;
  if(dragY<70)window.scrollBy(0,-14);else if(dragY>window.innerHeight-70)window.scrollBy(0,14);
  markDrop(dragX,dragY);dragFrame=requestAnimationFrame(scrollDrag);
}
function endDrag(commit){
  const from=draggedItem,to=dropItem;draggedItem=null;dropItem=null;
  cancelAnimationFrame(dragFrame);document.body.classList.remove('reordering');
  itemContainer.querySelectorAll('.dragging,.drop-target').forEach(el=>el.classList.remove('dragging','drop-target'));
  if(dragPointer){const {handle,id}=dragPointer;dragPointer=null;if(handle.hasPointerCapture(id))handle.releasePointerCapture(id);}
  if(commit&&from!==null&&to!==null)moveItem(from,to);
}
itemContainer.addEventListener('pointerdown',event=>{
  const handle=event.target.closest('[data-drag]');if(!handle||event.button!==0)return;
  event.preventDefault();draggedItem=Number(handle.dataset.drag);dropItem=draggedItem;dragY=event.clientY;dragX=event.clientX;
  dragPointer={handle,id:event.pointerId};handle.setPointerCapture(event.pointerId);handle.closest('.item-card').classList.add('dragging');document.body.classList.add('reordering');scrollDrag();
});
itemContainer.addEventListener('pointermove',event=>{if(draggedItem!==null){event.preventDefault();markDrop(event.clientX,event.clientY);}});
itemContainer.addEventListener('pointerup',()=>{if(draggedItem!==null)endDrag(true);});
itemContainer.addEventListener('pointercancel',()=>endDrag(false));
itemContainer.addEventListener('lostpointercapture',()=>{if(dragPointer)endDrag(false);});
itemContainer.addEventListener('dragstart',event=>event.preventDefault());
itemContainer.addEventListener('keydown',event=>{
  const handle=event.target.closest('[data-drag]');if(!handle||!['ArrowUp','ArrowDown'].includes(event.key))return;
  event.preventDefault();const from=Number(handle.dataset.drag),to=from+(event.key==='ArrowUp'?-1:1);moveItem(from,to);itemContainer.querySelector(`[data-drag="${Math.max(0,Math.min(to,data.items.length-1))}"]`)?.focus();
});
function navigationState(){return {billflux:true,page,invoiceId:data.invoice_id||0,companyId:appData.active_company_id};}
function recordNavigation(){
  if(restoringNavigation)return;
  const state=navigationState();
  if(!history.state?.billflux)history.replaceState(state,'');
  else if(JSON.stringify(history.state)!==JSON.stringify(state))history.pushState(state,'');
}
window.addEventListener('popstate',async event=>{
  const state=event.state;if(!state?.billflux)return;
  restoringNavigation=true;
  try{
    await flushSave();
    if(invoke&&state.companyId!==appData.active_company_id){
      await invoke('select_company',{companyId:state.companyId});appData=await invoke('load_app');companyDraft=structuredClone(activeCompany());
    }
    if(invoke&&state.page==='invoice'&&state.invoiceId&&state.invoiceId!==data.invoice_id){await openInvoice(state.invoiceId);}
    else{syncUi();showPage(state.page);}
  }catch(error){setStatus(String(error));}finally{restoringNavigation=false;}
});
document.addEventListener('keydown',event=>{
  if(event.altKey&&['ArrowLeft','ArrowRight'].includes(event.key)){event.preventDefault();event.key==='ArrowLeft'?history.back():history.forward();}
  else if(event.key==='BrowserBack'||event.key==='BrowserForward'){event.preventDefault();event.key==='BrowserBack'?history.back():history.forward();}
});
let editingCustomer=null;
function editCustomer(id=0){
  editingCustomer=activeCustomers().find(customer=>customer.id===id)||null;
  const party=editingCustomer?.party||emptyParty();
  document.getElementById('customer-dialog-title').textContent=editingCustomer?'Kunde bearbeiten':'Kunde anlegen';
  document.querySelectorAll('[data-customer-field]').forEach(input=>input.value=party[input.dataset.customerField]||'');
  const city=splitCompanyCity(party.city);document.querySelectorAll('[data-customer-address]').forEach(input=>input.value=city[input.dataset.customerAddress]);
  document.getElementById('customer-error').textContent='';document.getElementById('customer-dialog').showModal();
}
document.getElementById('new-customer').addEventListener('click',()=>editCustomer());
document.querySelectorAll('[data-close-dialog]').forEach(button=>button.addEventListener('click',()=>document.getElementById(button.dataset.closeDialog).close()));
document.getElementById('customer-form').addEventListener('submit',async event=>{
  event.preventDefault();const button=event.submitter;button.disabled=true;
  const party={...emptyParty(),...editingCustomer?.party};
  document.querySelectorAll('[data-customer-field]').forEach(input=>party[input.dataset.customerField]=input.value);
  party.city=[document.querySelector('[data-customer-address=postal_code]').value.trim(),document.querySelector('[data-customer-address=locality]').value.trim()].filter(Boolean).join(' ');
  try{
    const customer=editingCustomer?await invoke('update_customer',{id:editingCustomer.id,party}):await invoke('save_customer',{companyId:appData.active_company_id,party});
    appData.customers=appData.customers.filter(row=>row.id!==customer.id);appData.customers.unshift(customer);
    renderCustomers();document.getElementById('customer-dialog').close();setStatus('Kunde gespeichert');
  }catch(error){document.getElementById('customer-error').textContent=String(error);}finally{button.disabled=false;}
});
async function duplicateInvoice(id){
  await flushSave();data=await invoke('duplicate_invoice',{id,date:todayLocal()});
  appData=await invoke('load_app');companyDraft=structuredClone(activeCompany());dirty=false;
  customerFieldsExpanded=false;invoiceFieldsExpanded=false;paymentFieldsExpanded=false;syncUi();showPage('invoice');setStatus('Entwurf');
}
let invoiceToDelete=null;
function requestInvoiceDeletion(id){
  const invoice=appData.invoices.find(row=>row.id===id);if(!invoice)return;
  invoiceToDelete=id;document.getElementById('delete-invoice-description').textContent=`Rechnung ${invoice.number} für ${invoice.buyer||'diesen Empfänger'} löschen?`;
  document.getElementById('delete-invoice-error').textContent='';document.getElementById('delete-invoice-dialog').showModal();
}
document.getElementById('confirm-delete-invoice').addEventListener('click',async()=>{
  const button=document.getElementById('confirm-delete-invoice'),id=invoiceToDelete;button.disabled=true;
  try{
    if(data.invoice_id===id){clearTimeout(autosaveTimer);if(saving)await saving;dirty=false;}else await flushSave();
    await invoke('delete_invoice',{id});appData=await invoke('load_app');
    if(data.invoice_id===id){data=appData.workspace;customerFieldsExpanded=false;invoiceFieldsExpanded=false;}
    companyDraft=structuredClone(activeCompany());syncUi();showPage('invoices');document.getElementById('delete-invoice-dialog').close();setStatus('Rechnung gelöscht');
  }catch(error){document.getElementById('delete-invoice-error').textContent=String(error);}finally{button.disabled=false;}
});
