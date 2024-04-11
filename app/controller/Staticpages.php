<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;
use app\models\AphexChart;

class Staticpages extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,Customers,Attachment,Events');
	}

	public function vorlagen() {
		if (!auth_groups('Billflux')) {return;}
		$this->view->title = 'Vorlagen';
		$this->view->render('pages/vorlagen');
	}	

	public function dashboards() {

 		array_push($this->view->framework, 'https://cdn.jsdelivr.net/npm/apexcharts');
		//$this->view->events = $this->Events->find('hochgeladen');

		$this->view->potential = $this->Folders->potential_by('%Y-%m');

		$invoices = $this->Folders->amount_by('invoice');
		$invoiceamount = new AphexChart($invoices);
		$invoiceamount->name = 'Rechungen';
		$invoiceamount->height = 300;
		$invoiceamount->template = 'charts/default_donut_chart';
		$this->view->invoices = $invoices;
		$this->view->invoiceChart = $invoiceamount->generate();

		$types = $this->Folders->amount_by('type');
		$typeamount = new AphexChart($types);
		$typeamount->name = 'Produkte';
		$typeamount->height = 300;
		$typeamount->template = 'charts/default_donut_chart';
		$this->view->types = $types;
		$this->view->typeChart = $typeamount->generate();

		$statuses = $this->Folders->amount_by('status');
		$status = new AphexChart($statuses);
		$status->name = 'Statis';
		$status->height = 300;
		$status->template = 'charts/default_donut_chart';
		$this->view->statuses = $statuses;
		$this->view->statusChart = $status->generate();

		$monthly = $this->Folders->revenue_by('%Y-%m');
		$revenueByMonth = new AphexChart($monthly);
		$revenueByMonth->name = 'Umsätze';
		//$revenueByMonth->template = 'charts/default_line_chart';			
		$revenueByMonth->showValues = true;		
		$revenueByMonth->height = 300;
		$this->view->monthly = $monthly;
		$this->view->revenueByMonthChart = $revenueByMonth->generate();

		$yearly = $this->Folders->revenue_by('%Y');
		$revenueByYear = new AphexChart($yearly);
		$revenueByYear->name = 'Umsätze';
		$revenueByYear->showValues = true;		
		$revenueByYear->height = 300;
		$this->view->yearly = $yearly;	
		$this->view->revenueByYearChart = $revenueByYear->generate();

		$foldersMonthly = $this->Folders->folders_by('%Y-%m');
		$foldersByMonth = new AphexChart($foldersMonthly);
		$foldersByMonth->name = 'Umsätze';
		$foldersByMonth->showValues = false;		
		//$foldersByMonth->template = 'charts/default_line_chart';		
		$foldersByMonth->height = 300;
		$this->view->foldersMonthly = $foldersMonthly;	
		$this->view->foldersByMonthChart = $foldersByMonth->generate();	

		/*
		$test = new AphexChart();
		$test->dimension = ['Monat1', 'Monat2', 'Monat3'];
		$test->metric = [[100,200,300], [234,354,122]];
		$test->name = ['Soll','Ist'];
		$this->view->testChart = $test;
		*/

		$this->view->title = 'Dashboards';
		$this->view->render('pages/dashboards');
	}	

	public function invoices() {
		$this->view->events = $this->Events->find('bezahlt');
		$this->view->title = 'Upload Events';
		$this->view->render('pages/invoices');
	}	


	public function kontakte() {
		$this->view->title = 'Kontakte';
		$this->view->render('pages/kontakte');
	}

}
