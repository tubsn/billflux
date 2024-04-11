<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Clientview extends Controller {

	public function __construct() {
		//if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}
		$this->view('BlankLayout');
		$this->models('Folders,Attachment,Events');
	}

	public function index($token) {

		$folder = $this->Folders->get_by_token($token);
		if (!$folder) {throw new \Exception("Auftragsmappe nicht Gefunden", 404);}
		$this->view->formular = $folder;

		$template = 'clientview/onboarding';

		switch ($folder['phase']) {
			case 'Projektphase': $template = 'clientview/projekt'; break;
			case 'Ergebnisphase': $template = 'clientview/ergebnis'; break;
			case 'Abschlussphase': $template = 'clientview/funding'; break;
		}

		//if ($folder['status'] == 'fertig') {$template = 'clientview/dankeseite';}
		if ($folder['status'] == 'abgelehnt') {$template = 'clientview/schadeseite';}

		$this->view->title = 'Mein Auftrag - ' . $folder['phase'] . ' ID: ' . $folder['id'];
		$this->view->render($template);
	}

	public function update($token) {

		$folder = $this->Folders->get_by_token($token);

		$_POST = empty_to_null($_POST);
		$data['customer'] = $_POST['customer'];
		$data['object'] = $_POST['object'];
		$data['hull'] = $_POST['hull'];
		$data['renovation'] = $_POST['renovation'];
		$data['usages'] = $_POST['usages'];

		$dataChanged = $this->Folders->update($data, $folder['id']);
		if ($dataChanged) {$this->Events->customer('Projektdaten editiert', $folder['id']);}

		$this->view->redirect('/meinprojekt/' . $token);

	}

	public function update_status($token) {
		$this->Folders->token_as_index();
		if (isset($_POST['phase']) && !empty($_POST['phase'])) {
			$data = ['phase' => $_POST['phase']];
			$this->Folders->update($data, $token);
		}

		if (isset($_POST['status']) && !empty($_POST['status'])) {
			$data = ['status' => $_POST['status']];
			$this->Folders->update($data, $token);
			$folderID = $this->Folders->get($token,['id'])[0];
		}

	}

	public function trigger_event($token) {

		//if (!isset($_POST['event']) || empty($_POST['event'])) {return;}
		$event = $_POST['event'] ?? null;
		$payload = $_POST['payload'] ?? null;

		$folderID = $this->Folders->id_by_token($token);
		if (strtolower($event) == 'download') {
			if (is_null($payload)) {return;}
			$this->Events->download($payload, $folderID);
		}
		$this->view->json('done');

	}

	public function folder_json($token) {
		$data = $this->Folders->get_by_token($token);
		$this->view->json($data);
	}

	public function folder_attachments($token) {
		$data = $this->Folders->get_by_token($token);
		if (empty($data)) {throw new \Exception("Not found", 404);}

		$this->view->json($data['attachments']);
	}

	public function upload($token, $origin = 'customer') {
		$folder = $this->Folders->get_by_token($token);
		if (!$folder) {throw new \Exception("Not found", 404);}
		$uploaded = $this->Attachment->upload($_FILES, $folder['id'], $origin);
		
		if (!empty($uploaded['stored'])) {
			$storedIDs = array_column($uploaded['stored'],'id');
			$storedIDs = implode(',', $storedIDs);

			$message = 'Dateien hochgeladen IDs: ' . $storedIDs;
			if ($origin == 'attorney') {$message = 'Vollmacht hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'authorization') {$message = 'Ermächtigung hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'funding') {$message = 'Förderungsantrag hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'objects') {$message = 'Objektdaten hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'customer') {$message = 'Kundenformulare hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'invoice') {$message = 'Rechnungen hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'analytics') {$message = 'Ergebnis hochgeladen IDs: ' . $storedIDs;}
			$this->Events->customer($message, $folder['id']);		
		}

		$this->view->json($uploaded);

	}


	public function update_attachment($token, $attachmentID) {

		$folder = $this->Folders->get_by_token($token);
		if (!$folder) {throw new \Exception("Not found", 404);}

		$folderAttachmentIDs = array_column($folder['attachments'],'id');
		if (!in_array($attachmentID, $folderAttachmentIDs)) {throw new \Exception("Not found", 404);}

		if (empty($_POST['name'])) {return;}
		$data = ['name' => $_POST['name']];
		$this->Attachment->update($data, $attachmentID);
	}

	public function remove_attachment($token, $attachmentID) {

		$folder = $this->Folders->get_by_token($token);
		if (!$folder) {throw new \Exception("Not found", 404);}

		$folderAttachmentIDs = array_column($folder['attachments'],'id');
		if (!in_array($attachmentID, $folderAttachmentIDs)) {throw new \Exception("Not found", 404);}

		$this->Folders->remove_attachment($folder['id'], $attachmentID);
		$this->Events->customer('Datei gelöscht ID: ' . $attachmentID, $folder['id']);

	}

	public function folder_edited($token) {

		$this->Folders->token_as_index();
		$data = $this->Folders->get($token,['edited']);
		if (empty($data)) {throw new \Exception("Not found", 404);}
		
		if (empty($data['edited'])) {
			$this->view->json(['edited' => null]);
			return;
		}
		$this->view->json(['edited' => $data['edited']]);
	}

}
