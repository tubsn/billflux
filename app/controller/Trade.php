<?php

namespace app\controller;
use flundr\mvc\Controller;

class Trade extends Controller {

	public function __construct() {
		$this->view('DefaultLayout');
		$this->view->js =  ['/styles/js/main.js','/styles/js/tradeflux.js'];
		array_push($this->view->css, '/styles/css/tradeflux.css');
		$this->models('Positions');
	}

	public function index() {
		$this->view->render('trading/trade');
	}

	public function finance() {
		$this->view->render('trading/finance');
	}

}
