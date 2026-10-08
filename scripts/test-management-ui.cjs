const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const {pathToFileURL}=require('node:url');
const path=require('node:path');
(async()=>{
 const browser=await chromium.launch({channel:'chrome',headless:true});
 const page=await browser.newPage({viewport:{width:1440,height:950}});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.addInitScript(()=>{
  const party=name=>({name,contact:'Einkauf',street:'Straße 1',city:'12345 Ort',country:'DE',email:'',phone:'',website:'',tax_number:'123',vat_id:''});
  const company={id:1,party:party('Firma Eins'),template:'standard'};
  const ws={invoice_id:10,company_id:1,customer_id:3,seller:company.party,buyer:party('Kunde Alt'),number:'2026-0010',date:'2026-10-07',due_date:'',service_date:'',subject:'Original',payment_reference_prefix:'AM',payment_reference:'',account_holder:'Inhaber',iban:'DE123',bic:'TEST',bank_name:'Bank',items:[{description:'Leistung',detail:'',quantity:1,unit:'Stunden',unit_price_cents:8500,vat_percent:19}]};
  window.fixture={workspace:ws,companies:[company,{id:2,party:party('Firma Zwei'),template:'standard'}],customers:[{id:3,company_id:2,party:party('Kunde Alt')}],invoices:[{id:10,company_id:1,number:ws.number,buyer:ws.buyer.name,date:ws.date,subject:ws.subject,status:'draft'},{id:9,company_id:1,number:'2026-0009',buyer:'Zweiter Kunde',date:ws.date,subject:'Anderer Betreff',status:'issued'}],templates:['standard'],settings:{due_days:14,recent_count:5,iban:'DE123',bic:'TEST'},active_company_id:1,status:'draft'};
  window.calls=[];
  window.__TAURI__={core:{invoke:async(command,args)=>{
   window.calls.push({command,args:structuredClone(args)});
   const db=window.fixture;
   if(command==='load_app')return structuredClone(db);
   if(command==='calculate')return {lines:args.workspace.items.map(item=>({net_cents:item.quantity*item.unit_price_cents})),taxes:[],net_cents:8500,tax_cents:0,gross_cents:8500};
   if(command==='save_workspace'){db.workspace=structuredClone(args.workspace);return structuredClone(db.workspace);}
   if(command==='update_customer'){const c=db.customers.find(c=>c.id===args.id);c.party=structuredClone(args.party);return structuredClone(c);}
   if(command==='save_customer'){const c={id:4,company_id:args.companyId,party:structuredClone(args.party)};db.customers.push(c);return structuredClone(c);}
   if(command==='new_invoice'){db.workspace={...structuredClone(ws),invoice_id:12,customer_id:0,number:'2026-0012'};db.invoices.push({id:12,company_id:1,number:'2026-0012',buyer:'',date:ws.date,subject:'',status:'draft'});return structuredClone(db.workspace);}
   if(command==='duplicate_invoice'){db.workspace={...structuredClone(db.workspace),invoice_id:13,number:'2026-0013'};db.invoices.push({id:13,company_id:1,number:'2026-0013',buyer:db.workspace.buyer.name,date:ws.date,subject:'Kopie',status:'draft'});return structuredClone(db.workspace);}
   if(command==='delete_invoice'){db.invoices=db.invoices.filter(row=>row.id!==args.id);return;}
   throw Error('Unexpected command '+command);
  }}};
 });
 await page.goto(pathToFileURL(path.resolve(__dirname,'../desktop/ui/index.html')).href);
 await page.waitForFunction(()=>document.getElementById('save-status').textContent==='Entwurf');
 assert((await page.locator('#recent-invoices [data-open="10"] small').innerText()).includes('Original'));
 assert.equal(await page.locator('#recent-invoices small').first().evaluate(el=>getComputedStyle(el).textOverflow),'ellipsis');
 assert.equal(await page.locator('#invoice-customer').inputValue(),'3');
 for(const field of ['buyer.name','buyer.contact','buyer.street','buyer.city','buyer.country'])assert(await page.locator(`[data-path="${field}"]`).isHidden());
 await page.locator('#toggle-customer-fields').click();assert(await page.locator('[data-path="buyer.contact"]').isVisible());
 assert(await page.locator('[data-path=service_date]').isHidden());
 await page.locator('#toggle-invoice-fields').click();assert(await page.locator('[data-path=service_date]').isVisible());
 await page.locator('#toggle-payment-fields').click();
 assert.equal(await page.locator('[data-path=payment_reference]').inputValue(),'AM 2026-0010');
 assert(await page.locator('[data-path=bic]').isVisible());
 await page.locator('[data-path=payment_reference]').fill('Mein Auftrag 42');
 await page.locator('[data-path=payment_note]').fill('Vielen Dank für die Zusammenarbeit.');
 assert.equal(await page.evaluate(()=>data.payment_note),'Vielen Dank für die Zusammenarbeit.');
 await page.waitForFunction(()=>document.getElementById('preview').innerText.includes('Mein Auftrag 42'));
 await page.locator('#toggle-payment-fields').click();assert(await page.locator('[data-path=bic]').isHidden());
 await page.locator('[data-page=customers]').click();assert(await page.locator('[data-edit-customer="3"]').isVisible());
 await page.locator('[data-edit-customer="3"]').click();
 await page.locator('[data-customer-field=name]').fill('Kunde Neu');await page.locator('[data-customer-field=contact]').fill('Buchhaltung');
 await page.locator('#customer-form button[type=submit]').click();await page.waitForFunction(()=>!document.getElementById('customer-dialog').open);
 assert.equal(await page.evaluate(()=>data.buyer.name),'Kunde Alt');assert.equal(await page.evaluate(()=>data.buyer.contact),'Einkauf');
 await page.locator('[data-customer-invoice="3"]').click();await page.waitForFunction(()=>page==='invoice'&&data.buyer.name==='Kunde Neu');
 assert(await page.locator('[data-path="buyer.name"]').isHidden());
 assert.equal(await page.evaluate(()=>data.buyer.contact),'Buchhaltung');
 await page.evaluate(()=>flushSave());
 await page.locator('[data-page=invoices]').click();
 await page.locator('[data-sort=number]').click();
 let numbers=await page.locator('.invoice-open').allTextContents();assert.equal(numbers[0],'2026-0009');
 await page.locator('[data-sort=number]').click();numbers=await page.locator('.invoice-open').allTextContents();assert.equal(numbers[0],'2026-0012');
 await page.locator('[data-duplicate="10"]').click();await page.waitForFunction(()=>page==='invoice'&&data.invoice_id===13);
 await page.evaluate(()=>flushSave());
 await page.locator('[data-page=invoices]').click();await page.locator('[data-delete="9"]').click();assert(await page.locator('#delete-invoice-dialog').isVisible());
 await page.locator('#confirm-delete-invoice').click();await page.waitForFunction(()=>!document.getElementById('delete-invoice-dialog').open);
 assert.equal(await page.locator('[data-delete="9"]').count(),0);
 assert.deepEqual(errors,[]);
 await browser.close();console.log('Management UI tests passed: shared customers, independent snapshots, editing, custom reference, stable disclosure, sorting, duplication and deletion.');
})().catch(e=>{console.error(e);process.exit(1)});

