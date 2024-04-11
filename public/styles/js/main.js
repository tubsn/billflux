const emsAPP = Vue.createApp({
	data() {return {
		formID: null,
		title : '',
		edited : '',
		activeTab : 'home',
		events : [],
		eventFilter : 'system',
		date : '',
		date_invoice : '',
		date_payment : '',
		status: 'geplant',
		phase: 'keine',
		type: 'Software',
		invoice_id: '',
		invoice: 'offen',
		price: '',
		income: '',
		commission: '',
		office: 'Artmessengers',
		description: '',
		goals: '',
		customer: [],
		bank: [],
		issuer: '',
		attachments: '',
		loading: false,
		editCheckInterval: null,
	}},

	
	components: {
		'upload': flundrVueUpload,
		'fl-dialog': flundrVueDialog,
	},

	computed: {

		isNewForm() {if (!this.formID) {return true}},
		isClientForm() {
			if (!this.formID) {return}
			if (this.formID.length > 17 && this.formID.length < 22) {return true}
		},

		statusClass() {return this.status.replace(' ', '-')},
		mwstdif() {
			let price = this.price.toLocaleString('de-DE');
			let income = this.income.toLocaleString('en-US');
			
			console.log(parseFloat(income))
			console.log(price)

			return income - price
			//return price - income
		},

		revenue() {
			if (this.commission) {
				return parseFloat(this.price * (this.commission /100)).toFixed(2);
			}
			return this.price
		},

		analyticsAttachments() {
			if (this.attachments.length == 0) {return []}
			return this.attachments.filter(attachment => attachment.origin == 'analytics');
		},

		offerAttachments() {
			if (this.attachments.length == 0) {return []}
			return this.attachments.filter(attachment => attachment.origin == 'offer');
		},

		customerAttachments() {
			if (this.attachments.length == 0) {return []}
			return this.attachments.filter(
				attachment => attachment.origin == 'customer' || attachment.origin == 'funding' ||
				attachment.origin == 'attorney' || attachment.origin == 'authorization');
		},

		today() {return new Date().toISOString().split('T')[0]},

		customerShort() {
			if (this.customer.firm) {return this.customer.firm}
			if (this.customer.firstname && this.customer.name) {return this.customer.firstname + ' ' + this.customer.name}
			return 'Kunde'			
		},

		offerAmount() {if(this.offerAttachments && this.offerAttachments != 0) {return '(' + this.offerAttachments.length + ')'}},
		customerAmount() {if(this.customerAttachments && this.customerAttachments != 0) {return '(' + this.customerAttachments.length + ')'}},

	},

	mounted: function() {
		this.setFormID()
		this.setCalculatedDefaults()
		this.importFormData()
		if (location.hash) {this.setFormTab(location.hash.substr(1),'')}
		this.requiredCheck()
		if (this.$refs.autofocusElement) {this.$refs.autofocusElement.focus()}
	},

	watch: {
		/*phase(newPhase, oldPhase) { }, */
	},

	methods: {

		setFormID() {
			if (!this.$refs.form) {return null}
			this.formID = this.$refs.form.dataset.editId
		},
		submitForm() {this.$refs.form.submit()},
		setCalculatedDefaults() {this.date = this.today},


		async pingEditedTime() {
			let editTime;
			await fetch('/api/folder/'+this.formID+'/edited')
			.then(response => {return response.json()})
			.then(data => {editTime = data.edited})		
			return editTime
		},

		async hasBeenEdited() {
			let pingedEdit = Date.parse(await this.pingEditedTime())/1000
			let originalEdit = Date.parse(this.edited)/1000

			if (!originalEdit) {return}
			if (!pingedEdit) {return}

			if (originalEdit != pingedEdit) {return true}
			return false
		},

		async overwritePrevention() {
			if (this.isNewForm) {this.$refs.form.submit(); return;}
			
			if (await this.hasBeenEdited()) {
				let forceSave = confirm(`Achtung der Auftrag wurde im Hintergrund editiert.
				Wirklich speichern? (vorherige Änderungen werden überschrieben)`)
				if (forceSave) {this.$refs.form.submit();}
				return
			}

			this.$refs.form.submit()
		},



		async alertNewEdit() {
			if (await this.hasBeenEdited()) {
				clearInterval(this.editCheckInterval)
				let reload = confirm('Achtung Auftrag wurde im Hintergrund editiert! Seite neuladen?')
				if (reload) {location.reload()}
			}
		},

		async changeStatus(newStatus) {
			if (!newStatus.target) {this.status = newStatus}			
			let formData = new FormData()
			formData.append('status', this.status)
			await fetch('/api/folder/'+this.formID+'/status', {body: formData, method: 'post'})
			this.reloadEvents()
			this.renewCurrentEditDate()
		},

		async changePhase(newPhase) {
			if (!newPhase.target) {this.phase = newPhase}
			let formData = new FormData()
			formData.append('phase', this.phase)
			await fetch('/api/folder/'+this.formID+'/status', {body: formData, method: 'post'})
			this.reloadEvents()
			this.renewCurrentEditDate()
		},

		async changeInvoice(newInvoice) {
			if (!newInvoice.target) {this.invoice = newInvoice}
			let formData = new FormData()
			formData.append('invoice', this.invoice)
			await fetch('/api/folder/'+this.formID+'/status', {body: formData, method: 'post'})
			this.reloadEvents()
			this.renewCurrentEditDate()
		},

		async triggerEvent(event, payload = null) {
			let formData = new FormData()
			formData.append('event', event)
			if (payload) {formData.append('payload', payload)}
			await fetch('/api/folder/'+this.formID+'/event', {body: formData, method: 'post'})
			if (!this.isClientForm) {this.reloadEvents()}
		},	

		copyInvoiceAdress() {
			this.customer.inv_firm = this.customer.firm
			this.customer.inv_co = this.customer.co
			this.customer.inv_firstname = this.customer.firstname
			this.customer.inv_name = this.customer.name
			this.customer.inv_plz = this.customer.plz
			this.customer.inv_location = this.customer.location
			this.customer.inv_sublocation = this.customer.sublocation
			this.customer.inv_street = this.customer.street
			this.customer.inv_streetnum = this.customer.streetnum
		},

		copyInvoiceToClipboard() {

			let firm = this.customer.inv_firm || ''
			let co = this.customer.inv_co || ''
			if (co != '') {co = '\n' + 'c/o ' + co}
			if (firm) {co = co + '\n'}
			let name = `${this.customer.inv_firstname} ${this.customer.inv_name}`
			let location = `${this.customer.inv_plz} ${this.customer.inv_location}`
			if (this.customer.inv_sublocation) {location = `${location} ${this.customer.inv_sublocation}`}

			let adress = `${this.customer.inv_street} ${this.customer.inv_streetnum}`
			let invoice = `${firm}${co}${name}\n${adress}\n${location}` 
			
			navigator.clipboard.writeText(invoice);
		},


		requiredCheck() {
			let requiredInputs = document.querySelectorAll('[required]');
			requiredInputs.forEach(input => {
				input.addEventListener('invalid', (event) => {
					this.setFormTab('home');
					let element = event.target;
					setTimeout(function(){element.focus();}, 20);
				})

			});
		},

		setFormTab(tabName) {
			if(tabName == this.activeTab) {return;}

			let tabs = document.querySelectorAll('.form-tabs li');
			this.activeTab = tabName;

			let url = '#' + this.activeTab;
			if (this.activeTab == 'home') {url = window.location.href.split('#')[0];}			
			history.replaceState(null, null, url);
		},

		async importFormData() {
			let id = this.formID; 
			if (!id) {return;}
		
			let response = await fetch('/api/folder/'+id);
			let formdata = await response.json();

			this.importForm(formdata);
			if (this.title != '') {this.editCheckInterval = setInterval(this.alertNewEdit, 5000);}

			this.initPhotoSwipe()
		},

		importForm(data) {
			for (let [key, value] of Object.entries(data)) {
				if (value == null) {continue;}
				this[key] = value;
			}

		},

		async reloadEvents() {
			let id = this.formID;
			if (!id) {return;}
			let response = await fetch('/api/folder/'+id+'/events');
			let formdata = await response.json();
			this.events = formdata;
		},

		async reloadAttachments() {

			let id = this.formID;
			if (!id) {return;}
		
			let response = await fetch('/api/folder/'+id+'/attachments');
			let formdata = await response.json();

			this.attachments = formdata;
			this.renewCurrentEditDate();

		},

		async removeAttachment(id, $event) {

			if (!confirm('Anhang ID:'+id+' wirklich löschen?')) {return}

			clearInterval(this.editCheckInterval);

			let container = event.target.parentElement.parentElement
			container.onanimationend = (event) => {
				container.classList.remove('fade-out-right')
				this.attachments = this.attachments.filter(attachment => attachment.id != id)
			};

			container.classList.add('fade-out-right');
			await fetch('/attachment/'+this.formID+'/remove-attachment/'+id)
			this.renewCurrentEditDate()

			this.editCheckInterval = setInterval(this.alertNewEdit, 5000);

		},

		async renameAttachment(id, name) {
			let formData = new FormData()
			formData.append('name', name)

			let apipath = '/attachment/'
			if (this.isClientForm) {apipath += 'client/'+this.formID+'/'}
			
			await fetch(apipath + id, {body: formData, method: 'post'})

		},

		async renewCurrentEditDate() {
			console.log('Ping Date renewed')
			this.edited = await this.pingEditedTime()
		},


		initPhotoSwipe() {
			this.$nextTick(() => {
				window.lightbox = new PhotoSwipeLightbox({
					gallery: '.ps-thumb',
					//children: 'a',
					showHideAnimationType: 'zoom',
					imageClickAction: 'close',
					//initialZoomLevel: .7,
					pswpModule: PhotoSwipe,
				});
				window.lightbox.init();
			})
		},

		async generateCommunicationText() {

			await this.askAi()
		},

		async extractAddressAi(rawAddress) {

			this.loading = true
			let formData = new FormData()
			formData.append('address', rawAddress)
			let response = await fetch('/ai/extractaddress', {body: formData, method: 'post'});
			let address = await response.json();

			if (!address) {return}

			if (address.firstname) {this.customer.firstname = address.firstname}
			if (address.lastname) {this.customer.name = address.lastname}
			if (address.street) {this.customer.street = address.street}
			if (address.streetnum) {this.customer.streetnum = address.streetnum}
			if (address.plz) {this.customer.plz = address.plz}
			if (address.location) {this.customer.location = address.location}
			if (address.firm) {this.customer.firm = address.firm; this.customer.type = 'Firma'}
			if (address.co) {this.customer.co = address.co}
			if (address.telephone) {this.customer.telephone = address.telephone}
			if (address.email) {this.customer.email = address.email}
			if (address.salutation) {this.customer.salutation = address.salutation}

			this.loading = false
		},


		async askAi() {

			this.loading = true

			let inputPrompt = this.$refs.emailpreview.value

			let formData = new FormData()
			formData.append('question', inputPrompt)

			let response = await fetch('/ai/texts', {method: "POST", body: formData})
			if (!response.ok) {this.showError('API Network Connection Error: ' + response.status); return}

			response = await response.text()
			let json // no Idea why but it has to be defined first
			try {json = JSON.parse(response);}
			catch (error) {
				this.emailpreview = 'PHP Error: ' + response
				this.loading = false
				return
			}
		
			// PHP Api Handling Errors
			if (json.error) {this.showError(json.error); return}

			this.$refs.emailpreview.value = json
			this.loading = false

		},

	}, // End Methods
}).mount('#auftragsmappe')