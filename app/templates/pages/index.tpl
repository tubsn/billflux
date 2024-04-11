<main>

<a class="button flright" href="/new">Neu Anlegen</a>

<h1>Aktuelle Aufträge [<?=count($folders)?>] 
<?php if (isset($potential)): ?>- Potential: <?=gnum($potential)?>&thinsp;€<?php endif ?>
</h1>
	
<?php include(tpl('pages/parts/folder-table'))?>

<?php if (isset($archived)): ?>
<hr style="margin-top:2em; margin-bottom:1em;">
<h3>Archivierte Aufträge [<?=count($archived)?>] - Einnahmen: <?=gnum($paid)?>&thinsp;€</h3>

<?php $folders = $archived;?>
<?php include(tpl('pages/parts/folder-table'))?>
<?php endif ?>

</main>