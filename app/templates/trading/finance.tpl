<main id="fl-trade-app">

<h2 class="fright">Gesamtkapital: {{gnum(total_invest)}}</h2>
<h1>Tradeflux - Finanzplan</h1>

<div class="grid-3-col">
<section class="box">
	<h3>Longterm (z.B. ETFs)</h3>
	<div class="flex-fields">
		<label class="slide">Invest
			<input v-model="invest_etf" placeholder="ETF Anlage" type="text">
			<input v-model="invest_etf" type="range" step="5000" min="0" max="300000">
		</label>

		<label class="slide">Wachstum / Zinsen
			<input v-model="growth_etf" placeholder="Wachstum in Prozent" step="0.1" type="number">
			<input v-model="growth_etf" type="range" step="0.1" min="-15" max="+20">
		</label>
	</div>

	<table class="mt fancy">
		<tr>
			<td>Jahr:</td>
			<td>{{gnum(yearly_etf)}} €</td>
		</tr>	
		<tr>
			<td>Monat:</td>
			<td>{{gnum(monthly_etf)}} €</td>
		</tr>
	</table>
</section>

<section class="box">
	<h3>Festgeld</h3>
	<div class="flex-fields">
		<label class="slide">Invest
			<input v-model="invest_fixed" placeholder="Festgeldeinlage" type="text">
			<input v-model="invest_fixed" type="range" step="5000" min="0" max="100000">
		</label>

	<label class="slide">Wachstum / Zinsen
		<input v-model="growth_fixed" placeholder="Wachstum in Prozent" step="0.1" type="number">
		<input v-model="growth_fixed" type="range" step="0.1" min="0" max="+6">
	</label>
	</div>

	<table class="mt fancy">
		<tr>
			<td>Jahr:</td>
			<td>{{gnum(yearly_fixed)}} €</td>
		</tr>	
		<tr>
			<td>Monat:</td>
			<td>{{gnum(monthly_fixed)}} €</td>
		</tr>
	</table>
</section>

<section class="box">
	<h3>Tagesgeld</h3>
	<div class="flex-fields">
		<label class="slide">Invest
			<input v-model="invest_daily" placeholder="Tagesgeld" type="text">
			<input v-model="invest_daily" type="range" step="500" min="0" max="100000">
		</label>

	<label class="slide">Wachstum / Zinsen
		<input v-model="growth_daily" placeholder="Wachstum in Prozent" step="0.1" type="number">
		<input v-model="growth_daily" type="range" step="0.1" min="0" max="+6">
	</label>
	</div>

	<table class="mt fancy">
		<tr>
			<td>Jahr:</td>
			<td>{{gnum(yearly_daily)}} €</td>
		</tr>	
		<tr>
			<td>Monat:</td>
			<td>{{gnum(monthly_daily)}} €</td>
		</tr>
	</table>
</section>
</div>




<div class="box mt mb income-grid">
<section>
	<h3>Laufendes Jahr</h3>
	<table class="fancy wide">
		<tr>
			<td>Sparen / Jahr:</td>
			<td>{{gnum(monthly_saving*12)}}&nbsp;€</td>
		</tr>

		<tr>
			<td>Zinsen / Jahr:</td>
			<td>{{gnum(yearly_total)}}&nbsp;€</td>
		</tr>	
		<tr>
			<td>Zinsen / Monat:</td>
			<td><b>{{gnum(monthly_total)}}&nbsp;€</b></td>
		</tr>
	</table>
</section>

<section>
	<h3>Kapitaldetails</h3>
	<table class="fancy wide">
		<tr>
			<td>Kapital Start:</td>
			<td>{{gnum(total_invest)}}&nbsp;€</td>
		</tr>	

		<tr>
			<td>Kapital nach {{timeframe}} Jahren:</td>
			<td><b>{{gnum(total_after_x_years)}}&nbsp;€</b></td>
		</tr>

	</table>
</section>

<section>
	<h3>Gesamteinnahmen</h3>
	<table class="fancy wide">

		<tr>
			<td>Spareinnahmen nach {{timeframe}} Jahren:</td>
			<td>{{gnum(monthly_saving*12 * timeframe)}}&nbsp;€</td>
		</tr>	

		<tr>
			<td>Zinseinnahmen nach {{timeframe}} Jahren:</td>
			<td>{{gnum(interest_after_x_years)}}&nbsp;€</td>
		</tr>

	</table>
</section>

<section class="last-grid">
	<h3>nach Laufzeit versteuert</h3>
	<table class="fancy wide">
		<tr>
			<td>mit Rente pro Monat:</td>
			<td>{{gnum(total_monthly_taxed_rent)}}&nbsp;€</td>
		</tr>

		<tr>
			<td>Auszahlung pro Monat:</td>
			<td><b>{{gnum(total_monthly_taxed)}}&nbsp;€</b></td>
		</tr>

	</table>
</section>

</div>

<div class="box grid-2-wide">
<label>Notizfeld
<textarea class="notes-area" v-model="notes" placeholder="z.B. für Zwischenergebnisse"></textarea>
</label>

<section>
	<div class="flex-fields">
<label class="slide">Prognosejahre
	<input v-model="timeframe" placeholder="Jahre" step="1" type="number">
	<input v-model="timeframe" type="range" step="1" min="1" max="50">
</label>
<label class="slide">Monatssparen
	<input v-model="monthly_saving" placeholder="Sparsumme" type="text">
	<input v-model="monthly_saving" type="range" step="50" min="0" max="2000">
</label>
</div>

<div class="flex-fields mt">
<label class="slide">Basiszins
	<input v-model="basic_interest_tax" placeholder="ca 2,55%" type="number">
	<input v-model="basic_interest_tax" type="range" step="0.01" min="0" max="4">
</label>

<label class="slide">jetzige Rente
	<input v-model="rent" placeholder="Rente" type="number">
	<input v-model="rent" type="range" step="5" min="300" max="2500">
</label>

</section>


</div>





<!--
<section>
<input v-model="buy" placeholder="buy" type="text">
<input v-model="volume" placeholder="volume" type="text">
</section>

<section>
<label class="slide">Aktienwachstum
<input v-model="percentchange" placeholder="Veränderung" type="number">
<input v-model="percentchange" type="range" step="0.01" min="-50" max="+50">
</label>
<input v-model="now" placeholder="now" type="text">
</section>
-->



</main>