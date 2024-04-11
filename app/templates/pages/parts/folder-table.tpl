<?php if ($folders): ?>
<div class="overflow-table huge-shadow">
<table class="fancy homepage js-sortable wide">
	<thead>
	<tr>
		<th class="text-center">Datum</th>
		<th>Org.</th>
		<th>ID/R-NR.</th>
		<th>Produkt</th>
		<th>Titel</th>
		<th>Kunde</th>
		<th class="text-right">Preis</th>
		<th>Phase</th>
		<th>Rechnung</th>
		<th class="text-center">Status</th>
		<th class="text-right">⚙</th>
	</tr>
	</thead>

	<?php foreach ($folders as $folder): ?>
	<tr data-form-url="/auftrag/<?=$folder['id']?>/<?=slugify($folder['title'])?>">
		<td data-sortdate="<?=formatDate($folder['date'], 'Y-m-d', '-')?>" 
			title="<?=$folder['date']?>"
			class="text-center nowrap"><?=formatDate($folder['date'], 'M. y', '-')?></td>
		<td>
			<?php if ($folder['office'] == 'Artmessengers'): ?>AM<?php endif ?>
			<?php if ($folder['office'] == 'Tiefblau'): ?>TB<?php endif ?>
			<?php if ($folder['office'] == 'LR'): ?>LR<?php endif ?>
		</td>
		<td><?=$folder['invoice_id'] ?? $folder['id']?></td>


		<td><?=$folder['type'] ?? '-'?> <?=(empty($folder['objects'])) ? '' : '(' . count(json_decode($folder['objects'],1)) .')'?></td>
	
		<td style="min-width:250px" class="nowrap"><a href="/auftrag/<?=$folder['id']?>/<?=slugify($folder['title'] ?? 'Auftrag')?>"><?=$folder['title'] ?? 'Auftrag'?></a></td>

		<td title="<?=$folder['customer']['firm'] ?? 'Privat'?>" style="max-width:130px" class="nowrap"><a href="/auftrag/<?=$folder['id']?>/<?=slugify($folder['title'] ?? 'Auftrag')?>">
			<?=$folder['customer']['firm'] ?? 'Privat'?></a></td>
		

		<td class="text-right nowrap">
			<?php if ($folder['status'] != 'abgelehnt'): ?>
			<?=$folder['revenue'] ?? 'offen'?>&thinsp;€
			<?php else: ?>
			-
			<?php endif ?>
		</td>

		<td><?=$folder['phase']?></td>

		<td class="status">
			<select name="invoice" onchange="changeInvoiceInList(event)" data-folder-id="<?=$folder['id']?>">
				<?php if ($folder['invoice']): ?>
				<option selected><?=$folder['invoice']?></option>
				<?php endif ?>
				<?php foreach (INVOICE_TYPES as $invoice): ?>
				<?php if ($folder['invoice'] == $invoice) {continue;} ?>
				<option><?=$invoice?></option>
				<?php endforeach ?>
			</select>
		</td>

		<td class="text-center status <?=$folder['status'] ?? ''?>">
			<select name="status" onchange="changeStatusInList(event)" data-folder-id="<?=$folder['id']?>">
				<?php if ($folder['status']): ?>
				<option selected><?=$folder['status']?></option>
				<?php endif ?>
				<?php foreach (STATUS_TYPES as $status): ?>
				<?php if ($folder['status'] == $status) {continue;} ?>
				<option><?=$status?></option>
				<?php endforeach ?>
			</select>
		</td>

		<td style="text-align: right;">
			<a id="copy-campaign-<?=$folder['id']?>"title="kopieren" class="noline pointer"><img class="icon-delete" src="/styles/flundr/img/icon-copy.svg"></a>
			<a id="del-campaign-<?=$folder['id']?>" title="löschen" class="noline pointer"><img class="icon-delete" src="/styles/flundr/img/icon-delete-black.svg"></a>
			<fl-dialog selector="#copy-campaign-<?=$folder['id']?>" href="/auftrag/<?=$folder['id']?>/copy">
			<h1><?=$folder['title']?> - kopieren?</h1>
			<p>Achtung - Dateianhänge werden NICHT mit kopiert!</p>
			</fl-dialog>			
			<fl-dialog selector="#del-campaign-<?=$folder['id']?>" href="/auftrag/<?=$folder['id']?>/delete">
			<h1><?=$folder['title']?> - löschen?</h1>
			<p>Achtung die Daten werden unwideruflich gelöscht!</p>
			</fl-dialog>
		</td>

	</tr>
	<?php endforeach ?>

</table>
</div>

<?php else: ?>
	Keine Einträge
<?php endif ?>