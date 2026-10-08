const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {pathToFileURL}=require('node:url');
(async()=>{
 const browser=await chromium.launch({channel:'chrome',headless:true});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:1000}});
  await page.goto(pathToFileURL(path.resolve('desktop/ui/index.html')).href);
  await page.waitForFunction(()=>document.getElementById('save-status').textContent==='Entwurf');
  await page.evaluate(()=>showPage('invoice'));
  const field=page.locator('textarea[data-key=detail]').first();
  assert.equal(await field.getAttribute('rows'),'2');
  const detail='Straße & <Details>\nZweite Zeile\n\nWeitere Angaben';
  await field.fill(detail);
  await page.waitForFunction(value=>data.items[0].detail===value,detail);
  await page.waitForFunction(value=>document.querySelector('#preview td small')?.textContent===value,detail);
  assert.equal(await page.locator('#preview td small').first().evaluate(el=>getComputedStyle(el).whiteSpace),'pre-wrap');
  await page.evaluate(()=>renderItems());
  assert.equal(await field.inputValue(),detail);
  assert.equal((await field.locator('..').innerText()).trim(),'Details');
  await field.scrollIntoViewIfNeeded();
  fs.mkdirSync('tmp/details-check',{recursive:true});
  await field.locator('..').screenshot({path:'tmp/details-check/field.png'});
  const template=fs.readFileSync('templates/standard/invoice.html','utf8');
  const css=fs.readFileSync('templates/standard/style.css','utf8').replaceAll('url("fonts/', 'url("'+pathToFileURL(path.resolve('fonts')).href+'/');
  await page.goto('about:blank');
  page.on('pageerror',error=>console.error(error));
  for(const count of [3,120]){
   const lines=Array.from({length:count},(_,i)=>`Detailzeile ${i+1}: Straße & Leistung`).join('\n');
   const escaped=lines.replaceAll('&','&amp;');
   const row=`<tr><td>1</td><td class="item" style="white-space:pre-wrap"><strong>Leistung</strong><br>${escaped}</td><td>19 %</td><td>100 €</td><td>1 h</td><td>100 €</td></tr>`;
   const html=template.replace('{{rows}}',row).replace(/\{\{\w+\}\}/g,'').replace('<link rel="stylesheet" href="style.css">',`<style>${css}</style>`);
   fs.writeFileSync('tmp/details-check/layout.html',html);
   await page.goto(pathToFileURL(path.resolve('tmp/details-check/layout.html')).href);
   await page.waitForFunction(()=>document.documentElement.dataset.pagination==='ready');
   const actual=await page.locator('td.item').evaluateAll(cells=>cells.map(cell=>[...cell.childNodes].filter(n=>n.nodeType===Node.TEXT_NODE).map(n=>n.textContent).join('')).join(''));
   assert.equal(actual,lines);
   assert(await page.locator('.sheet').count()>=(count===120?2:1));
   assert(await page.locator('.sheet').evaluateAll(sheets=>sheets.every(sheet=>sheet.scrollHeight<=sheet.clientHeight+1)));
  }
  console.log('Details: multiline editing, rerender, preview and pagination passed.');
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
