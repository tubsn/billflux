const flTradeAPP = Vue.createApp({
	data() {return {
		timeframe: null,
		basic_interest_tax: null,
		rent: null,
		monthly_saving: null,
		invest_etf: null,
		invest_fixed: null,
		invest_daily: null,
		growth_etf: null,
		growth_fixed: null,
		growth_daily: null,
		
		buy: null,
		buy2: null,
		now: null,
		now2: null,
		volume: null,
		volume2: null,
		percentchange: null,
		percentchange2: null,
		savings: null,
		price: null,
		revenue: null,
		notes: null,

		bop: null,

		persistentFields: [
			'invest_etf', 'invest_daily', 'invest_fixed', 
			'growth_etf', 'growth_daily', 'growth_fixed', 'monthly_saving',
			'timeframe', 'notes', 'buy', 'buy2', 'now', 'now2', 'price', 
			'savings', 'volume', 'volume2', 'basic_interest_tax', 'rent'
		],

	}},

	components: {'events-table': flundrVueOperations,},
	watch: {},
	computed: {

		// 2,55% pro Jahr
		// bei entnahme 1000€ Frei und 30,5% = * 0.305

		returnofinvest() {return (parseFloat(this.now) - parseFloat(this.buy)) * this.volume},
		returnofinvest2() {return (parseFloat(this.now2) - parseFloat(this.buy2)) * this.volume2},
		planed_volume() {return (parseFloat(this.savings) / parseFloat(this.price)).toFixed(0)},
		final_savings() {return (parseFloat(this.savings) + parseFloat(this.returnofinvest)).toFixed(2)},
		
		yearly_etf() {
			let yearly = (this.invest_etf * (this.growth_etf/100))
			let etf_pretax = (this.invest_etf * (this.basic_interest_tax/100) * 0.7)
			etf_pretax = etf_pretax * 0.305
			return (yearly - etf_pretax)
		},

		yearly_fixed() {
			let yearly = (this.invest_fixed * (this.growth_fixed/100))
			return yearly - (yearly * 0.305)
		},

		yearly_daily() {
			let yearly = (this.invest_daily * (this.growth_daily/100))
			return yearly - (yearly * (this.basic_interest_tax/100))
		},

		monthly_etf() {return (this.yearly_etf / 12)},	
		monthly_fixed() {return (this.yearly_fixed / 12)},	
		monthly_daily() {return (this.yearly_daily / 12)},	
		monthly_total() {return (this.monthly_etf + this.monthly_fixed + this.monthly_daily)},
		monthly_taxed() {return this.monthly_total - (this.monthly_total * 0.305)},

		total_after_x_years() {
			
			let total = 0
			let savings = 0

			let etf = parseFloat(this.invest_etf)
			let fixed = parseFloat(this.invest_fixed)
			let daily = parseFloat(this.invest_daily)
			
			for (let i = 0; i < this.timeframe; i++) {
			
				let etf_pretax = (etf * (this.basic_interest_tax/100) * 0.7)
				etf_pretax = etf_pretax * 0.305

				etf = (etf * (this.growth_etf/100)) + etf
				etf = etf - etf_pretax
				etf = etf + (this.monthly_saving*12)
				
				fixed = (fixed * (this.growth_fixed/100)) + fixed
				daily = (daily * (this.growth_daily/100)) + daily

			}

			total = etf + fixed + daily + savings

			return total
		},

		total_monthly_taxed() {
			let total = this.total_after_x_years
			interest = (total * (this.growth_etf/100))
			interest = interest / 12
			interest = interest - (interest * 0.305)
			return interest
		},

		total_monthly_taxed_rent() {
			return this.total_monthly_taxed + parseFloat(this.rent)
		},

		total_after_x_years_taxed() {
			return this.total_after_x_years - (this.total_after_x_years * 0.305)
		},

		yearly_total() {return (parseFloat(this.yearly_etf) + parseFloat(this.yearly_fixed) + parseFloat(this.yearly_daily)).toFixed(2)},


		yearly_total_zins() {
			if (this.yearly_total == 0.00) {return 0}
			if (!this.yearly_total) {return 0}
			if (isNaN(this.yearly_total)) {return 0}
			return (this.yearly_total - (this.yearly_total*0.0255)).toFixed(2)
		},

		interest_after_x_years() {
			let savings = this.monthly_saving * 12 * this.timeframe
			return this.total_after_x_years - savings - this.total_invest
		},

		yearly_taxed() {
			if (this.yearly_total == 0.00) {return 0}
			if (!this.yearly_total) {return 0}
			if (isNaN(this.yearly_total)) {return 0}			
			return (((this.yearly_total-1000) - (this.yearly_total-1000)*0.305)+1000).toFixed(2)
		},		
	

		total_invest() {return parseFloat(this.invest_etf || 0) + parseFloat(this.invest_daily || 0) + parseFloat(this.invest_fixed || 0)},
		
		total_growth() {return this.yearly_etf + this.yearly_daily + this.yearly_fixed},
	},

	mounted: function() {
		this.makeFieldsPersistent()
	},

	methods: {

		gnum(value) {
			if (!value) {return 0}
			if (isNaN(value)) {return 0}
			return parseFloat(value).toLocaleString('de-DE', {maximumFractionDigits: 2, minimumFractionDigits: 2})
		},

		makeFieldsPersistent() {
			let fields = this.persistentFields

			// Load last settings from Localstorage
			fields.forEach(field => {this[field] = localStorage[field] || null})
			
			// Add Fields to Watch()
			fields.forEach(field => {
				this.$watch(() => this[field], (value) => {
					if (value) {localStorage[field] = value}
				});
			})
		},

		setVolume() {this.volume = this.planed_volume},

		refreshPercentage() {
			if (this.now == 0) {return 0}
			this.percentchange = ((this.now / this.buy * 100)-100).toFixed(2)
		},

		refreshPercentage2() {
			if (this.now == 0) {return 0}
			this.percentchange2 = ((this.now2 / this.buy2 * 100)-100).toFixed(2)
		},

		refreshNow() {
			this.now = (this.buy * this.percentchange).toFixed(2)
		},


		highlightone(item) {
			this.buy = this.parseGer(item.buy)
			this.now = this.buy
			this.volume = this.parseGer(item.amount)
			this.refreshPercentage()
			//this.bop = item
		},

		highlighttwo(item) {
			this.buy2 = this.parseGer(item.buy)
			this.now2 = this.buy2
			this.volume2 = this.parseGer(item.amount)
			this.refreshPercentage2()
			//this.bop = item
		},

		parseGer(n) {
			if (!n) {return 0}
			n = n.replace(/\./g, '').replace(',', '.')
			return n
		},

	}, // End Methods
}).mount('#fl-trade-app')