<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Search extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,ChatGPTApi');
	}


	public function search() {
		$query = $_GET['q'] ?? null;
		$this->view->query = $query;
		$this->view->folders = $this->Folders->find($query);
		$this->view->results = count($this->view->folders ?? []);

		// KI Search
		//if (str_contains($query,'?')) {$this->ki_search(); return;}

		$this->view->title = 'Suche: ' . $query . ' ['.$this->view->results.']';		
		$this->view->render('pages/index');
	}


	public function ki_search() {

		$folders = $this->Folders->list_mediaplans();
		$folders = json_encode($folders,1);

		$query = $_GET['q'] ?? null;
		$this->view->query = $query;

		$question = $query . "\n\n" . $folders;

		$this->ChatGPTApi->wipe_history();
		$ans = $this->ChatGPTApi->ask($question);

		$this->view->response = $ans['answer'];
		$this->view->render('ki-search');

	}

}
