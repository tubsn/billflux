<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Customers extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,Customers,Attachment');
	}

	public function index() {
		$this->view->customers = $this->Customers->list();
		$this->view->title = 'Kundenliste';
		$this->view->render('customer/list');
	}

}
