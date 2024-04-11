<fieldset class="grid-2-1">
<div>

<fieldset class="grid-2-col">
	<label>Firmenname
		<input v-model="customer.firm" name="customer[firm]" placeholder="Firmenname" type="text">
	</label>

	<label>Care of
		<input v-model="customer.co" name="customer[co]" placeholder="c/o" type="text">
	</label>
</fieldset>

<fieldset class="grid-3-col">
	<label>E-Mail
		<input v-model="customer.email" name="customer[email]" placeholder="mail@example.de" type="text">
	</label>
	<label>Telefon
		<input v-model="customer.telephone" name="customer[telephone]" placeholder="+49 xxx xxxxxxxx" type="text">
	</label>

	<label>Internet
		<input v-model="customer.url" name="customer[url]" placeholder="https://www.example.de" type="text">
	</label>
</fieldset>


<fieldset class="grid-customer">
	<label>Anrede
	<select v-model="customer.salutation" name="customer[salutation]">
		<option value="0">Bitte wählen</option>
		<option>Herr</option>
		<option>Frau</option>
		<option>Divers</option>
	</select>
	</label>

	<label>Vorname
		<input v-model="customer.firstname" name="customer[firstname]" placeholder="Vorname" type="text">
	</label>
	<label>Nachname
		<input v-model="customer.name" name="customer[name]" placeholder="Name" type="text">
	</label>
</fieldset>



<fieldset>
	<fieldset class="grid-2-1">
		<label>Straße
			<input v-model="customer.street" name="customer[street]" placeholder="Straße" type="text">
		</label>

		<label>Hausnummer
			<input v-model="customer.streetnum" name="customer[streetnum]" placeholder="Hausnummer" type="text">
		</label>
	</fieldset>

	<fieldset class="grid-3-center-wide">
		<label>PLZ
			<input v-model="customer.plz" name="customer[plz]" placeholder="Postleitzahl" type="text">
		</label>

		<label>Ort
			<input v-model="customer.location" name="customer[location]" placeholder="Ortsangabe" type="text">
		</label>

		<label>Ortsteil
			<input v-model="customer.sublocation" name="customer[sublocation]" placeholder="(optional)" type="text">
		</label>
	</fieldset>

</fieldset>

<fieldset class="mblarge">
<label>Notizen zum Kundenkontakt
	<textarea v-model="customer.info" name="customer[info]" placeholder="z.B. Besonderheiten aus Gesprächen"></textarea>
</label>
</fieldset>



<details class="blue-inset" :open="customer.inv_firm || customer.inv_name || customer.inv_plz">
	<summary>Abweichende Rechnungsadresse
		<div class="fright" style="font-size:0.8em">
			<button v-if="customer.inv_firm || customer.inv_name || customer.inv_plz" class="light" @click.prevent="copyInvoiceToClipboard" type="button">in Zwischenablage </button>&ensp;
			<button class="light" @click.prevent="copyInvoiceAdress" type="button">aus Anschrift übernehmen</button>
		</div>
	</summary>

	<fieldset class="grid-4-wide">
		<label>Firma
			<input v-model="customer.inv_firm" name="customer[inv_firm]" placeholder="Firmenname" type="text">
		</label>
		<label>Care of
			<input v-model="customer.inv_co" name="customer[inv_co]" placeholder="c/o" type="text">
		</label>		
		<label>Vorname
			<input v-model="customer.inv_firstname" name="customer[inv_firstname]" placeholder="Vorname" type="text">
		</label>
		<label>Nachname
			<input v-model="customer.inv_name" name="customer[inv_name]" placeholder="Name" type="text">
		</label>
	</fieldset>

	<fieldset>
		<fieldset class="grid-3-center-wide">
			<label>PLZ
				<input v-model="customer.inv_plz" name="customer[inv_plz]" placeholder="Postleitzahl" type="text">
			</label>

			<label>Ort
				<input v-model="customer.inv_location" name="customer[inv_location]" placeholder="Ortsangabe" type="text">
			</label>

			<label>Ortsteil
				<input v-model="customer.inv_sublocation" name="customer[inv_sublocation]" placeholder="(optional)" type="text">
			</label>
		</fieldset>

		<fieldset class="grid-2-1">
			<label>Straße
				<input v-model="customer.inv_street" name="customer[inv_street]" placeholder="Straße" type="text">
			</label>

			<label>Hausnummer
				<input v-model="customer.inv_streetnum" name="customer[inv_streetnum]" placeholder="Hausnummer" type="text">
			</label>
		</fieldset>
	</fieldset>
</details>



</div>



<figure style="margin-left:2em;padding:3em; padding-top:1em; box-sizing: border-box;">

	<div class="grey-inset text-center mb">

		<fl-dialog @ok="extractAddressAi" addonclasses="" headline="Addressdaten auslesen">Addresse Importieren<div v-if="loading" class="loadIndicator white"><div></div><div></div><div></div></div>
		</fl-dialog>

		<h3>{{ customerShort }}</h3>
		<a v-if="customer.email" :href="'mailto:'+customer.email">{{ customer.email}}</a> 
		<span v-if="!customer.email">Bitte Kundendaten vervollständigen!</span> 
		<br>{{ customer.plz }} {{ customer.location }} {{ customer.street ? '|' : '' }} {{ customer.street }} {{ customer.streetnum }}
	</div>

	<img class="hide-mobile" style="width:100%; margin-top:2em;" src="/styles/img/teamwork.svg">

</figure>

</fieldset>