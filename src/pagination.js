document.fonts.ready.then(() => {
      const body = document.body;
      const marks = [...body.querySelectorAll('.fold-mark')];
      const masthead = body.querySelector('.masthead');
      const address = body.querySelector('.address-area');
      const heading = body.querySelector('main h1');
      const table = body.querySelector('main table');
      const rows = [...table.tBodies[0].rows];
      const tableHead = table.tHead;
      const totals = body.querySelector('.totals');
      const payment = body.querySelector('.payment');
      const footer = body.querySelector('footer');
      body.classList.add('paginated');
      body.replaceChildren();

      function newSheet(first) {
        const sheet = document.createElement('section');
        sheet.className = 'sheet';
        marks.forEach(mark => sheet.append(mark.cloneNode(true)));
        if (first) sheet.append(masthead, address);
        const main = document.createElement('main');
        if (first) main.append(heading);
        const pageTable = document.createElement('table');
        pageTable.append(tableHead.cloneNode(true), document.createElement('tbody'));
        main.append(pageTable);
        sheet.append(main);
        body.append(sheet);
        return {sheet, main, table:pageTable};
      }

      let current = newSheet(true);
      for (let index = 0; index < rows.length; index++) {
        const row = rows[index];
        current.table.tBodies[0].append(row);
        if (current.sheet.scrollHeight > current.sheet.clientHeight + 1 && current.table.tBodies[0].rows.length > 1) {
          row.remove();
          current = newSheet(false);
          current.table.tBodies[0].append(row);
        }
        if (current.sheet.scrollHeight > current.sheet.clientHeight + 1) {
          if (current.sheet === body.firstElementChild) {
            row.remove(); current = newSheet(false); index--; continue;
          }
          const item = row.querySelector('td.item');
          const detail = [...item.childNodes].find(node => node.nodeType === Node.TEXT_NODE && node.textContent.trim());
          const words = detail?.textContent.match(/\S+\s*|\s+/g) || [];
          if (words.length > 1) {
            let low = 0, high = words.length;
            while (low < high) {
              const middle = Math.ceil((low + high) / 2);
              detail.textContent = words.slice(0, middle).join('');
              if (current.sheet.scrollHeight <= current.sheet.clientHeight + 1) low = middle;
              else high = middle - 1;
            }
            if (low > 0 && low < words.length) {
              detail.textContent = words.slice(0, low).join('');
              const continuation = row.cloneNode(true);
              continuation.cells[0].textContent = '';
              continuation.querySelector('td.item').textContent = words.slice(low).join('');
              [...continuation.cells].slice(2).forEach(cell => cell.textContent = '');
              rows.splice(index + 1, 0, continuation);
              current = newSheet(false);
            } else {
              detail.textContent = words.join('');
            }
          }
        }
      }
      current.main.append(totals, payment);
      current.sheet.append(footer);
      if (current.sheet.scrollHeight > current.sheet.clientHeight + 1) {
        totals.remove(); payment.remove(); footer.remove();
        current = newSheet(false);
        current.table.remove();
        current.main.append(totals, payment);
        current.sheet.append(footer);
      }
      document.documentElement.dataset.pagination = 'ready';
      window.dispatchEvent(new Event('invoice-layout-ready'));
    });
