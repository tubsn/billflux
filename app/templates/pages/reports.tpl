<main>
<h1><?=$page['title']?></h1>

<?php table_dump($report);?>


<p>Aufträge: <?=count($report ?? [])?><br>
Umsatz: <?=gnum($profit)?> € <br>
Mwst: <?=gnum($mwst)?> €<br>
Steuerlast: <?=gnum(($profit*0.3))?> € (~30%)
</p>

</main>