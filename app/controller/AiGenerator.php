<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;
//use flundr\auth\JWTAuth;
//use flundr\cache\RequestCache;
//use flundr\utility\Session;

class AiGenerator extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->view->title = 'AI Generator';
		$this->models('ChatGPT');
	}

	public function engines() {
		$engines = $this->ChatGPT->list_engines();
		dd($engines);
	}

	public function extract_address() {
		$addressString = $_POST['address'] ?? null;

		$prompt = 'Extrahiere mir aus folgenden Impressum Angaben die Adressdaten für eine Rechnung und gebe diese als Json Array wieder 
Falls weitere Daten wie z.B. Firmendaten, Telefonnummer oder E-Mail Adresse vorhanden sind ergänze diese. Der Geschäftsführer kann Rechnungsempfänger für firstname und lastname sein. Nutze immer folgende Feldnamen:
firm, salutation, firstname, lastname, street, streetnum, plz, location, co (für Care of), telephone, email' . "\n\nImpressumsangaben:\n\n";

		$response = $this->ChatGPT->direct($prompt . $addressString);
		$array = json_decode($response,1);
		$this->view->json($array);
	}

	public function email() {
		$question = $_POST['question'] ?? null;
		$response = $this->ChatGPT->direct($question);
		$this->view->json($response);
	}



	public function fun_fact() {
		$cache = new RequestCache('funfact', 60*60);
		$funfact = $cache->get();
		if (empty($funfact)) {
			$date = date('d.F');
			$question = 'Mach lustigen Witz zum heutigen Tag ('.$date.'). Maximal 30 Wörter. Orientiere dich am Humor von Dave Chappelle. Themenbereich Naturwissenschaft';
			$funfact = $this->ChatGPT->direct($question);
			$cache->save($funfact);
		}

		return $funfact;
	}




}
