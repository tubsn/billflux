<h1>Rechnungen und Geldeingang</h1>

<div class="grid-2-1">

<fieldset class="grid-2-1 gap-wide">

	<section>
	
		<fieldset>
		<label>Notizen zum Geldeingang
			<textarea v-model="bank.notes" name="bank[notes]" class="medium" placeholder="z.B. Hinweise zum Geldeingang, Verspätet bezahlt usw."></textarea>
		</label>
	</fieldset>

	<fieldset class="grid-2-col">

	<label>Rechnungsnummer
		<input type="text" v-model="invoice_id" name="invoice_id" placeholder="z.B. 2023-42001">
	</label>

	<label>Rechnungsdatum
		<div class="input-group">
		<input v-model="date_invoice" name="date_invoice" type="date">
		<button @click.prevent="date_invoice = today" class="light">Heute</button>
		</div>
	</label>


	</fieldset>



<!--
	<label>Nettopreis:
		<input type="text" v-model="price" name="price" placeholder="Endpreis für Kunden">
	</label>

	<label>Mwst:
		<input type="text" :value="mwstdif" placeholder="Mwst.">
	</label>
-->






	</section>

	<section>

	<label>Datum Geldeingang
		<div class="input-group">
		<input v-model="date_payment" name="date_payment" type="date">
		<button class="light" @click.prevent="date_payment = today, triggerEvent('invoice', 'Geldeingang'), reloadEvents()">Heute</button>	
		</div>	
	</label>

	<hr>

	<label>Erhaltene Summe in Euro:
		<input type="text" v-model="income" name="income" placeholder="Summe in Euro">
	</label>

	</section>

</fieldset>

<figure style="margin-left:2em; padding:0 3em; box-sizing: border-box;">
	<img class="hide-mobile" style="width:100%; margin-top:-2em;" src="/styles/img/documents.svg">
</figure>


</div>
