<main id="fl-trade-app">

<h2 class="fright">Gesamtkapital: {{gnum(final_savings)}} €</h2>
<h1>Tradeflux - Trading</h1>

<div>
{{ bop }}
</div>

<div class="grid-2-col mb">
<section class="box mb ">
	<h3>Aktie</h3>
	<div class="flex-fields">
		<label class="slide">Buy
			<input v-model="buy" @change="refreshPercentage" placeholder="Kaufpreis" type="number">
			<input v-model="buy" @change="refreshPercentage" type="range" step="1" min="0" max="1500">
		</label>

		<label class="slide">Now
			<input v-model="now" @change="refreshPercentage" placeholder="Aktueller Preis" type="number">
			<input v-model="now" @change="refreshPercentage" type="range" step="1" min="0" max="1500">
		</label>

		<label class="slide" style="width:33%">Volume
			<input v-model="volume" placeholder="Anzahl" step="1" type="number">
			<input v-model="volume" type="range" step="1" min="0" max="200">
		</label>

		<label class="slide">Wachstum Prozent
			<input v-model="percentchange" @change="refreshNow" placeholder="Wachstum in Prozent" step="0.1" type="number">
			<input v-model="percentchange" @change="refreshNow" type="range" step="0.1" min="-10" max="+10">
		</label>
	</div>

	<table class="mt fancy">
		<tr>
			<td>Rendite:</td>
			<td>{{gnum(returnofinvest)}} €</td>
		</tr>	
	</table>
</section>


<section class="box">
	<h3>Aktie 2</h3>
	<div class="flex-fields">
		<label class="slide">Buy
			<input v-model="buy2" @change="refreshPercentage2" placeholder="Kaufpreis" type="number">
			<input v-model="buy2" @change="refreshPercentage2" type="range" step="1" min="0" max="1500">
		</label>

		<label class="slide">Now
			<input v-model="now2" @change="refreshPercentage2" placeholder="Aktueller Preis" type="number">
			<input v-model="now2" @change="refreshPercentage2" type="range" step="1" min="0" max="1500">
		</label>

		<label class="slide" style="width:33%">Volume
			<input v-model="volume2" placeholder="Anzahl" step="1" type="number">
			<input v-model="volume2" type="range" step="1" min="0" max="200">
		</label>

		<label class="slide">Wachstum Prozent
			<input v-model="percentchange2" @change="refreshNow" placeholder="Wachstum in Prozent" step="0.1" type="number">
			<input v-model="percentchange2" @change="refreshNow" type="range" step="0.1" min="-10" max="+10">
		</label>
	</div>

	<table class="mt fancy">
		<tr>
			<td>Rendite:</td>
			<td>{{gnum(returnofinvest2)}} €</td>
		</tr>	
	</table>
</section>


<section class="box">
	<label>Notizfeld
		<textarea class="notes-area" v-model="notes" placeholder="z.B. für Zwischenergebnisse"></textarea>
	</label>
</section>

<section class="box">
	<h3>Anlage Potential</h3>
	<div class="flex-fields">
		<label class="slide">Kapital
			<input v-model="savings" placeholder="Verfügbares Kapital" type="number">
			<input v-model="savings" type="range" step="50" min="0" max="50000">
		</label>

	<label class="slide">Einkaufspreis
		<input v-model="price" placeholder="Preis" step="0.1" type="number">
		<input v-model="price" type="range" step="0.1" min="0" max="1500">
	</label>
	</div>

	<table class="mt fancy">
		<tr>
			<td>Anzahl Anteile:</td>
			<td>{{gnum(planed_volume)}} &emsp; <button @click="setVolume">Übernehmen</button>
			</td>
		</tr>			
	</table>

	

</section>

</div>

<hr>

<form>
<events-table @selectone="highlightone" @selecttwo="highlighttwo" class="mb"></events-table>
</form>

</main>