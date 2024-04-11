<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;
use flundr\rendering\TemplateEngine ;

class Export extends Controller {

	public function __construct() {
		$this->view('DefaultLayout');
		$this->models('Folders,PDFGenerator');
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}
	}

	public function pdf() {
		$this->PDFGenerator->test();
	}


	public function invoice($id) {

		$this->view('BlankLayout');

		$folder = $this->Folders->get($id);
		$objects = json_decode($folder['objects'],1);
		$customer = json_decode($folder['customer'],1);

		$this->view->folder = $folder;
		$this->view->objects = $objects;
		$this->view->customer = $customer;
		
		$viewData['folder'] = $folder;
		$viewData['objects'] = $objects;
		$viewData['customer'] = $customer;

		$tpl = new TemplateEngine('export/invoice' ,$viewData);
		dd($tpl->burn());

		//$this->view->burn('export/invoice');

	}


}
