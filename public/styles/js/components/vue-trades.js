const flundrVueOperations = Vue.defineComponent({
data() {return {
	errormessage: '',
	loading: false,
	operations: [],
	userlist: [],
	referencelist: ['Ist Zustand', 'Variante', 'Sonstiges'],
	categorylist: ['Thermische Hülle', 'Anlagentechnik', 'Sonstiges'],
	componentlist: ['Dach', 'Oberste Geschossdecke', 
		'Außenwand', 'Bodenaufbau', 'Fenster',
		'Tür', 'Heizung', 'Sonstiges'],
}},

props: [],
emits: [],

computed: {
	formID() {return this.$parent.formID},

	total() {
		let total = 0
		this.operations.forEach(item => {
			total = parseFloat(total) + parseFloat(this.parseGer(this.calcRevenue(item)))
		})

		return total.toFixed(2)

	}

},

template: `


<div class="flex small" style="justify-content:flex-end; float:right;">
<button @click.prevent="copyContent" class="light" type="button">kopieren</button>&nbsp;
<button @click.prevent="add" type="button">+</button>
</div>

<h3>Trading Historie: {{ total }} €</h3>

<div v-if="loading">Historie wird geladen...</div>
<div v-if="errormessage" class="warning">{{errormessage}}</div>

<div class="mb" v-if="operations.length == 0">Keine Einträge vorhanden</div>


<table v-if="operations.length > 0" class="fancy wide noalts"> 

<thead>
	<tr>
		<th>Aktie</th>
		<th style="width:80px">Volume</th>
		<th>Buy</th>
		<th>Sell</th>
		<th class="nowrap" style="width:120px">Revenue</th>
		<th style="width:40%">Notiz</th>
		<th style="width:120px">In</th>
		<th style="width:120px">Out</th>
		<th class="text-right">⚙</th>
	</tr>
</thead>
<tbody>

<tr v-for="item in operations">
	<td><input v-model="item.title" data-name="title" :data-id="item.id" @change="edit" type="text" placeholder="Name"></td>
	<td><input v-model="item.amount" data-name="amount" :data-id="item.id" @change="edit" type="text" placeholder="Anzahl"></td>
	<td><input v-model="item.buy" data-name="buy" :data-id="item.id" @change="edit" type="text" placeholder="Kaufpreis"></td>
	<td><input v-model="item.sell" data-name="sell" :data-id="item.id" @change="edit" type="text" placeholder="Verkaufspreis"></td>
	<td><div v-html="calcRevenue(item)"></div></td>
	<td><input v-model="item.notes" data-name="notes" :data-id="item.id" @change="edit" type="text" placeholder="Notizen"></td>
	<td><input v-model="item.indate" data-name="indate" :data-id="item.id" @change="edit" type="text" placeholder="Kaufdatum"></td>
	<td><input v-model="item.outdate" data-name="outdate" :data-id="item.id" @change="edit" type="text" placeholder="Verkaufsdatum"></td>

	<td style="text-align: right;">
		<img @click="remove(item.id)" class="icon-delete" src="/styles/flundr/img/icon-delete-black.svg">
	</td>
</tr>

</tbody>

</table>
`,

mounted: function() {
	this.$nextTick(() => {
		this.load_operation_users()
		this.load_operations()
	})
},

methods: {

	async load_operation_users() {
		const apiurl = '/api/users/backoffice'
		let response = await fetch(apiurl)
		if (!response.ok) {}
		let text; text = await response.text(); // Recieves any Server output

		try { // Parse Text as JSON
			let json = JSON.parse(text)
			this.userlist = json.map(item => item.firstname);

		} catch {
			this.errormessage = text
		}

	},

	parseGer(n) {
		if (!n) {return 0}
		n = n.replace(/\./g, '').replace(',', '.')
		return n
	},

	calcRevenue(item) {
		if (!item.sell) {return 0}
		
		let buyTotal = 0
		let sellTotal = 0

			let buy = this.parseGer(item.buy)
			let sell = this.parseGer(item.sell)

			buy = parseFloat(buy)
			sell = parseFloat(sell)

			buy = buy * parseFloat(item.amount)
			sell = sell * parseFloat(item.amount)

			buyTotal = buyTotal + buy
			sellTotal = sellTotal + sell

			return this.gnum(sellTotal - buyTotal)
	},

	gnum(value) {
		if (!value) {return 0}
		if (isNaN(value)) {return 0}
		return parseFloat(value).toLocaleString('de-DE')
	},

	async load_operations() {

		this.loading = true

		const apiurl = '/api/orders/'
		let response = await fetch(apiurl)
		if (!response.ok) {}

		let text; text = await response.text(); // Recieves any Server output

		try { // Parse Text as JSON
			let json = JSON.parse(text)
			this.operations = json

		} catch {
			this.errormessage = text
		}

		this.loading = false
	},

	async add() {
		const apiurl = '/api/orders/add'
		let response = await fetch(apiurl)

		if (!response.ok) {}
		let text; text = await response.text(); // Recieves any Server output

		try { // Parse Text as JSON
			let json = JSON.parse(text)
			let newID = json.id
			this.operations.push({
				'id': newID,
				'user' : null,
				'reference' : null,
				'category' : null,
				'component' : null,
			})

		} catch {
			this.errormessage = text
		}

			
	},

	async remove(id) {
		const apiurl = '/api/orders/' + id + '/delete'
		let response = await fetch(apiurl)
		this.operations = this.operations.filter(item => item.id !== id)
	},

	async edit(event) {

		let operationsID = event.target.dataset.id
		let key = event.target.dataset.name
		let value = event.target.value

		const apiurl = '/api/orders/' + operationsID
		let formData = new FormData()
		formData.append(key, value)

		let response = await fetch(apiurl, {body: formData, method: 'post'})

		if (!response.ok) {}
		let text; text = await response.text(); // Recieves any Server output

		try { // Parse Text as JSON
			let json = JSON.parse(text)
		} catch {
			this.errormessage = text
		}
	},


	copyContent() {

		if (!this.operations) {return}

		let output = ''
		this.operations.forEach(item => {
			output += `${item.title} | ${item.amount} | ${item.buy} | ${item.sell || '0'} | ${this.calcRevenue(item) || '0'} | ${item.notes || ''} | ${item.indate || '-'} | ${item.outdate || 'offen'}\n`
		})

		navigator.clipboard.writeText(output);
	},


}, // End Methods
})