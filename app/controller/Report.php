<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Report extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,Events');
	}

	public function tax() {

		$start = 'first day of january this year';
		$end = 'last day of december this year';

		$data = $this->Folders->tax_year($start, $end);
		$this->view->report = $data;

		$this->view->mwst = array_sum(array_column($data, 'mwst'));
		$this->view->profit = array_sum(array_column($data, 'revenue'));
		
		$this->view->title = 'Tax Report: laufendes Jahr [' . count($data ?? []) . ']';
		$this->view->render('pages/reports');
	}

	public function tax_last_year() {

		$data = $this->Folders->tax_year();
		$this->view->report = $data;

		$this->view->mwst = array_sum(array_column($data, 'mwst'));
		$this->view->profit = array_sum(array_column($data, 'revenue'));

		$this->view->title = 'Tax Report: letztes Jahr [' . count($data ?? []) . ']';
		$this->view->render('pages/reports');

	}
}
