<main>
<h1>Übersicht von Rechnungs Events</h1>



<style>.fancy {width:100%;}
</style>

<div class="grid-2-col gap-wide">
<?php foreach ($events as $group => $event): ?>

<section>
<h3><?=$group?></h3>
<?=table_dump($event)?>
</section>
<?php endforeach ?>
</div>

</main>