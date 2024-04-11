<fieldset class="grid-5-wide">

	<label>Titel
		<input required <?=isset($formular) && $formular['title'] ? '' : 'ref="autofocusElement"'?> v-model="title" name="title" type="text" placeholder="z.B. Gartenwebseite">
	</label>

	<label>Auftragsart
		<select v-model="type" name="type">
			<option>Webdesign</option>
			<option>Software</option>
			<option>CI/CD</option>
			<option>Wartung</option>
			<option>Sonstiges</option>
		</select>
	</label>

	<label>Firma
		<select v-model="office" name="office">
			<option>Artmessengers</option>
			<option>Tiefblau</option>			
			<option>LR</option>			
		</select>
	</label>

	<label>Auftragdatum
		<input v-model="date" name="date" type="date">
	</label>

	<label>Auftragstatus
		<select v-model="status" @change="changeStatus" name="status">
			<option>geplant</option>
			<option>in Arbeit</option>
			<option>fertig</option>
			<option>abgelehnt</option>
		</select>
	</label>

</fieldset>


<fieldset class="grid-5-col">

	<label>Produktpreis in Euro
		<input type="text" v-model="price" name="price" placeholder="Endpreis für Kunden">
		<input type="hidden" v-model="revenue" name="revenue">
	</label>

	<label>Rechnungsnummer
		<input type="text" v-model="invoice_id" name="invoice_id" placeholder="z.B. 2023-42001">
	</label>

	<label>Provisionssatz in %
		<input type="text" v-model="commission" name="commission" placeholder="z.B. 15">
	</label>

	<label>Projektphase
		<select v-model="phase" @change="changePhase" name="phase">
			<option>keine</option>
			<option>Onboarding</option>
			<option>Umsetzung</option>
			<option>Auslieferung</option>
			<option>Wartung</option>
			<option>Abgeschlossen</option>
		</select>
	</label>



	<label>Rechnung
		<select v-model="invoice" @change="changeInvoice" name="invoice">
			<option>offen</option>
			<option>verschickt</option>
			<option>bezahlt</option>
			<option>abgelehnt</option>
		</select>
	</label>


</fieldset>


<div class="grid-2-1">
	<fieldset>
		<label>Projektbeschreibung
			<textarea v-model="description" name="description" class="medium" placeholder="z.B. was soll umgesetzt werden"></textarea>
		</label>
	</fieldset>

	<fieldset>
		<label>Kundenziele
			<textarea v-model="goals" name="goals" class="medium" placeholder="z.B. Neue Homepage bis März"></textarea>
		</label>

	</fieldset>
</div>

<div>


	<fieldset>
	<label>Dateianhänge zum Projekt</label>
	</fieldset>

	<div v-if="offerAttachments.length > 0" class="attachment-container">
		<figure v-for="attachment in offerAttachments" class="attachment-item">
			<div class="attachment-remove">
				<img @click="removeAttachment(attachment.id, $event)" class="icon-delete" src="/styles/flundr/img/icon-delete-black.svg">
			</div>
			<a :href="attachment.url" :target="attachment.target" :class="{'ps-thumb' : attachment.width}" :data-pswp-width="attachment.width" :data-pswp-height="attachment.height"><img :src="attachment.thumbnail" :alt="attachment.name" :title="attachment.name"></a>
			<figcaption>
				<span class="attachment-name">
					<input @change="renameAttachment(attachment.id, attachment.name)" @keypress.enter.prevent class="attachment-edit-input" v-model="attachment.name">
				</span>
				<span class="attachment-filesize">({{ attachment.size }})</span>
			</figcaption>	
		</figure>
	</div>

	<fieldset v-if="!isNewForm" class="upload-area white-inset">
		<label>
			<upload @done="reloadAttachments(), reloadEvents()" action="/auftrag/<?=$formular['id'] ?? '-'?>/upload/offer" multiple>[+] Dateien Anhängen</upload>&ensp; z.B. Rechungen, Präsentationen usw.
		</label>
	</fieldset>

	<fieldset v-else class="upload-area white-inset">
		<label>Die Upload-Funktion steht nach speichern des Auftrags zur Verfügung</label>
	</fieldset>

</div>

