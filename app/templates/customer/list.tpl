<main>

<a class="button flright" href="/new">Neu Anlegen</a>
<h1><?=$page['title']?></h1>

<?php if ($customers): ?>
<div class="overflow-table huge-shadow">
<table class="fancy homepage js-sortable wide">
	<thead>
	<tr>
		<th>Name</th>
		<th>Mail</th>
		<th>PLZ</th>
		<th>Location</th>
		<th>URL</th>
		<th>ID</th>
	</tr>
	</thead>

	<?php foreach ($customers as $customer): ?>
	<tr data-form-url="/auftrag/<?=$customer['id']?>/<?=slugify($customer['firm'] ?? '')?>">
		<?php if (!empty($customer['firm'])): ?>
		<td><?=$customer['firm']?></td>
		<?php else: ?>
		<td><?=$customer['firstname']?> <?=$customer['name'] ?? ''?></td>
		<?php endif ?>
		<td><a href="mailto:<?=$customer['email']?>"><?=$customer['email']?></a></td>		
		<td><?=$customer['plz'] ?? '-'?></td>
		<td><?=$customer['location'] ?? '-'?></td>
		<td><?=$customer['url'] ?? '-'?></td>
		<td><a href="/auftrag/<?=$customer['id']?>/<?=slugify($customer['firm'] ?? '')?>"><?=$customer['id']?></a></td>

	</tr>
	<?php endforeach ?>

</table>
</div>
<?php endif ?>


</main>