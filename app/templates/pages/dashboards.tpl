<main>


<h1 class="fright text-left"><?=APP_NAME?> - Dashboard</h1>
<h1 class="text-left">Gesamtumsatz: <?=gnum(array_sum($yearly))?>&thinsp;Euro</h1>

<style>.fancy {width:100%;}</style>

<div class="text-center box mblarge">
<p>
aktuelles Jahr: <b class="sphere"><?=gnum(end($yearly))?>&thinsp;€</b> &emsp; 
aktueller Monat: <b class=""><?=gnum(end($monthly))?>&thinsp;€</b> &emsp; 
Ø-Monatsumsatz: <b class=""><?=gnum(array_sum($yearly)/12)?>&thinsp;€</b> &emsp; 
| &emsp; 
Aufträge: <b><?=array_sum($statuses)?></b> &emsp; 
davon abgeschlossen: <b class="sphere"><?=$statuses['fertig']?></b> &emsp;
| &emsp; 
Potentieller Gesamtumsatz: <b><?=gnum(array_sum($potential))?>&thinsp;€</b> &emsp; 
Ø-Potentieller Monatsumsatz: <b class="sphere"><?=gnum(array_sum($potential)/12)?>&thinsp;€</b> &emsp; 
</p>

</div>



<div class="grid-2-back-wide">
	<figure>
		<h3 class="text-center">Umsätze nach Jahr</h3>
		<?=$revenueByYearChart?>
	</figure>

	<figure>
		<h3 class="text-center">Umsätze nach Rechnungs Monat</h3>
		<?=$revenueByMonthChart?>
		
	</figure>
</div>

<h3 class="text-center">Verteilung von Aufträgen</h3>
<div class="grid-2-back-verywide mblarge" style="grid-template-columns: 1fr 2fr 1fr;">
<?=$typeChart?>
<?=$foldersByMonthChart?>
<?=$invoiceChart?>
</div>

<div class="grid-2-col mblarge" style="max-width:1200px; margin:0 auto;">
<figure><h3>Produkte</h3><?=table_dump($types)?></figure>
<figure><h3>Rechnungen</h3><?=table_dump($invoices)?></figure>
</div>

</main>