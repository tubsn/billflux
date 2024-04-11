<h1>Auftragshistorie</h1>
<div class="grid-2-col gap-wide">

<section>
	<table class="fancy wide">
	<tr>
		<th>Kategorie</th>
		<th>Event</th>
		<th>Datum</th>
		<th>Uhrzeit</th>
		<th>User</th>
	</tr>

	<tr v-for="event in events">
	<td>{{event.category}}</td>
	<td>{{event.content}}</td>
	<td>{{event.day}}</td>
	<td>{{event.time}} Uhr</td>
	<td>{{event.user_id}}</td>
	</tr>
	</table>
</section>

</div>

