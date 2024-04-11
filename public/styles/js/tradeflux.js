const flTradeAPP = Vue.createApp({
	data() {return {
		timeframe: null,
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
	}},

	components: {
		'events-table': flundrVueOperations,
	},

	computed: {

		returnofinvest() {return (parseFloat(this.now) - parseFloat(this.buy)) * this.volume},
		returnofinvest2() {return (parseFloat(this.now2) - parseFloat(this.buy2)) * this.volume2},
		planed_volume() {return (parseFloat(this.savings) / parseFloat(this.price)).toFixed(0)},
		final_savings() {return (parseFloat(this.savings) + parseFloat(this.returnofinvest)).toFixed(2)},
		
		yearly_etf() {return ((this.invest_etf/100) * this.growth_etf).toFixed(2)},
		monthly_etf() {return (((this.invest_etf/100) * this.growth_etf) / 12).toFixed(2)},	

		yearly_fixed() {return ((this.invest_fixed/100) * this.growth_fixed).toFixed(2)},
		monthly_fixed() {return (((this.invest_fixed/100) * this.growth_fixed) / 12).toFixed(2)},	

		yearly_daily() {return ((this.invest_daily/100) * this.growth_daily).toFixed(2)},
		monthly_daily() {return (((this.invest_daily/100) * this.growth_daily) / 12).toFixed(2)},	

		yearly_total() {return (parseFloat(this.yearly_etf) + parseFloat(this.yearly_fixed) + parseFloat(this.yearly_daily)).toFixed(2)},
		monthly_total() {return (parseFloat(this.monthly_etf) + parseFloat(this.monthly_fixed) + parseFloat(this.monthly_daily)).toFixed(2)},

		yearly_taxed() {
			if (this.yearly_total == 0.00) {return 0}
			if (!this.yearly_total) {return 0}
			if (isNaN(this.yearly_total)) {return 0}			
			return (((this.yearly_total-1000) - (this.yearly_total-1000)*0.305)+1000).toFixed(2)
		},		
		monthly_taxed() {
			return (this.yearly_taxed/12).toFixed(2)
		},		

		total_invest() {return parseFloat(this.invest_etf || 0) + parseFloat(this.invest_daily || 0) + parseFloat(this.invest_fixed || 0)},
		total_growth() {
			return (parseFloat(this.yearly_taxed) + parseFloat(this.total_invest) +(12*parseFloat(this.monthly_saving))).toFixed(2)
		},
	},

	mounted: function() {
		this.loadHistory()
	},

	watch: {
		notes(value) {localStorage.notes = value}, 
		timeframe(value) {if (value) {localStorage.timeframe = value}}, 
		monthly_saving(value) {if (value) {localStorage.monthly_saving = value}}, 
		invest_etf(value) {if (value) {localStorage.invest_etf = value}}, 
		invest_daily(value) {if (value) {localStorage.invest_daily = value}}, 
		invest_fixed(value) {if (value) {localStorage.invest_fixed = value}}, 
		growth_etf(value) {if (value) {localStorage.growth_etf = value}}, 
		growth_daily(value) {if (value) {localStorage.growth_daily = value}}, 
		growth_fixed(value) {if (value) {localStorage.growth_fixed = value}}, 
		
		buy(value) {if (value) {localStorage.buy = value}}, 
		buy2(value) {if (value) {localStorage.buy2 = value}}, 
		price(value) {if (value) {localStorage.price = value}}, 
		savings(value) {if (value) {localStorage.savings = value}}, 
		now(value) {if (value) {localStorage.now = value}}, 
		now2(value) {if (value) {localStorage.now2 = value}}, 
		volume(value) {if (value) {localStorage.volume = value}}, 
		volume2(value) {if (value) {localStorage.volume2 = value}}, 
	},

	methods: {

		gnum(value) {
			if (!value) {return 0}
			if (isNaN(value)) {return 0}
			return parseFloat(value).toLocaleString('de-DE')
		},

		loadHistory() {
			this.invest_etf = localStorage.invest_etf || null
			this.invest_daily = localStorage.invest_daily || null
			this.invest_fixed = localStorage.invest_fixed || null
			this.growth_etf = localStorage.growth_etf || null
			this.growth_daily = localStorage.growth_daily || null
			this.growth_fixed = localStorage.growth_fixed || null
			this.monthly_saving = localStorage.monthly_saving || null
			this.timeframe = localStorage.timeframe || null
			this.notes = localStorage.notes || null
			
			this.buy = localStorage.buy || null
			this.buy2 = localStorage.buy2 || null
			this.now = localStorage.now || null
			this.now2 = localStorage.now2 || null
			this.price = localStorage.price || null
			this.savings = localStorage.savings || null
			this.volume = localStorage.volume || null
			this.volume2 = localStorage.volume2 || null
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


	}, // End Methods
}).mount('#fl-trade-app')