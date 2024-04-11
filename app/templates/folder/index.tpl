<main id="auftragsmappe">

<aside class="flright hide-mobile">
<a class="button light noline" href="/">zurück zur Übersicht</a> 
<button class="button" style="margin-left:10px" type="submit" onclick="document.getElementById('submit-btn').click()">Angaben speichern</button>
</aside>

<h1 v-cloak>{{ title || 'Neue Auftragsmappe' }} - {{ type }} <span class="highlight-sphere"><small>{{ phase }}</small></span></h1>

<nav class="form-tabs" v-cloak>
	<ul>
	<li :class="{'active' : activeTab == 'home'}" @click="setFormTab('home')"><img src="/styles/img/folder.svg"> Allgemein</li>
	<li :class="{'active' : activeTab == 'kunde'}" @click="setFormTab('kunde')"><img src="/styles/img/customers.svg"> Kundendaten</li>
	<li :class="{'active' : activeTab == 'invoice'}" @click="setFormTab('invoice')"><img src="/styles/img/documents.svg"> Finanzdaten</li>
	<li :class="{'active' : activeTab == 'events'}" @click="setFormTab('events')"><img src="/styles/flundr/img/icon-calendar.svg"> Events</li>

	</ul>
</nav>

<div class="form-container huge-shadow" :class="statusClass" v-cloak>

<form action="" method="post" @submit.prevent="overwritePrevention" class="main-form" 
ref="form" data-edit-id="<?=$formular['id'] ?? ''?>" data-username="<?=auth('firstname').' '.auth('lastname');?>" data-user-id="<?=auth('id')?>">

<section v-show="activeTab == 'home'">
<?php include(tpl('folder/allgemein'))?>
</section>

<section v-show="activeTab == 'kunde'">
<?php include(tpl('folder/kunde'))?>
</section>

<section v-show="activeTab == 'invoice'">
<?php include(tpl('folder/invoice'))?>
</section>

<section v-show="activeTab == 'events'">
<?php include(tpl('folder/events'))?>
</section>

<hr>

<fieldset class=" mt">
	<button id="submit-btn" type="submit">Angaben speichern</button>
	&nbsp;
	<a class="button light noline" href="/">zurück zur Übersicht</a> 	
</fieldset>

<?php if (isset($formular['created'])): ?>
<div class="meta-info hide-mobile" ><span style="color:#aaaaaa">angelegt: </span><?=formatDate($formular['created'], 'd.m.y')?>
<?php if ($formular['edited']): ?>
 <span style="color:#aaaaaa">| editiert: </span><?=formatDate($formular['edited'], 'd.m.y')?> <span style="color:#aaaaaa"><?=formatDate($formular['edited'], 'H:i')?></span></div>
<?php endif ?>
<?php endif ?>

</form>
</div>

</main>