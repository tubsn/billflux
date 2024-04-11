<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Tests extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,Attachment,Events');
	}

	public function index() {

		$this->view->render('test');


		


	}

}
