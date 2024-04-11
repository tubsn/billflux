<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Folder extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,Attachment,Events');
	}

	public function index() {

		$folders = $this->Folders->list();

		$activefolders = array_filter($folders, function($folder){
			if ($folder['status'] == 'abgelehnt') {return null;}
			if ($folder['invoice'] != 'bezahlt') {return $folder;}
		});
		
		$archievedFolders = array_filter($folders, function($folder){
			if ($folder['invoice'] == 'bezahlt') {return $folder;}
		});

		$paidFolders = array_filter($folders, function($folder){
			if ($folder['invoice'] == 'bezahlt') {return $folder;}
		});

		$this->view->potential = array_sum(array_column($activefolders,'revenue'));
		$this->view->paid = array_sum(array_column($paidFolders,'revenue'));
		$this->view->folders = $activefolders ?? [];
		$this->view->archived = $archievedFolders ?? [];
		$this->view->title = 'Aktuelle Aufträge';
		$this->view->render('pages/index');
	}

	public function category($name) {
		$this->view->folders = $this->Folders->type($name);
		if (!$this->view->folders) {throw new \Exception("Keine Projekte gefunden", 404);}
		$this->view->title = APP_NAME_SHORT . ' Projekte: ' . urldecode(ucfirst($name));
		$this->view->render('pages/index');
	}

	public function edit($id, $slug = null) {
		$this->view->formular = $this->Folders->get($id);
		if (!$this->view->formular) {throw new \Exception("Auftragsmappe nicht Gefunden", 404);}
		$this->view->title = APP_NAME_SHORT . ': ' . $this->view->formular['title'];
		$this->view->render('folder/index');
	}

	public function update_status($id) {
		if (isset($_POST['phase']) && !empty($_POST['phase'])) {
			$data = ['phase' => $_POST['phase']];
			$this->Folders->update($data, $id);
			$this->Events->phase($_POST['phase'], $id);			
		}

		if (isset($_POST['status']) && !empty($_POST['status'])) {
			$data = ['status' => $_POST['status']];
			$this->Folders->update($data, $id);
			$this->Events->status($_POST['status'], $id);
		}

		if (isset($_POST['invoice']) && !empty($_POST['invoice'])) {
			$data = ['invoice' => $_POST['invoice']];
			$this->Folders->update($data, $id);
			$this->Events->invoice($_POST['invoice'], $id);
		}

	}

	public function remove_attachment($id, $attachmentID) {
		$this->Folders->remove_attachment($id, $attachmentID);
		$this->Events->internal('Datei gelöscht ID: ' . $attachmentID, $id);		
	}

	public function update_attachment($id) {
		if (empty($_POST['name'])) {return;}
		$data = ['name' => $_POST['name']];
		$this->Attachment->update($data, $id);
	}

	public function upload($id, $origin = 'home') {
		$uploaded = $this->Attachment->upload($_FILES, $id, $origin);
	
		if (!empty($uploaded['stored'])) {
			$storedIDs = array_column($uploaded['stored'],'id');
			$storedIDs = implode(',', $storedIDs);

			$message = 'Dateien hochgeladen IDs: ' . $storedIDs;
			if ($origin == 'attorney') {$message = 'Vollmacht hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'funding') {$message = 'Förderungsantrag hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'objects') {$message = 'Objektdaten hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'customer') {$message = 'Kundenformulare hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'invoice') {$message = 'Rechnungen hochgeladen IDs: ' . $storedIDs;}
			if ($origin == 'analytics') {$message = 'Ergebnis hochgeladen IDs: ' . $storedIDs;}
			$this->Events->internal($message, $id);		
		}

		$this->view->json($uploaded);
	}

	public function copy($id) {
		$newID = $this->Folders->copy($id);
		$folder = $this->Folders->get($newID);
		$this->Events->internal('Projekt kopiert aus ID: ' . $id, $newID);
		$url = '/auftrag/' . $folder['id'] . '/' . slugify($folder['title']);
		$this->view->redirect($url);
	}

	public function delete($id) {
		$this->Folders->delete($id);
		$this->Events->internal('Projekt gelöscht', $id);	
		$this->view->redirect('/');
	}

	public function create() {
		$this->view->title = 'Neue Auftragsmappe';		
		$this->view->types = $this->Folders->distinct_types();
		$this->view->render('folder/index');
	}

	public function save() {
		if (empty($_POST['title'])) {$_POST['title'] = 'Unbenannt';}
		$_POST = empty_to_null($_POST);
		//$_POST['customertoken'] = $this->Folders->customer_token();
		$newID = $this->Folders->create(array_filter($_POST));
		$this->Events->internal('Projekt erstellt', $newID);
		$url = '/auftrag/' . $newID . '/' . slugify($_POST['title']);
		$this->view->redirect($url);
	}

	public function update($id, $slug = null) {
		$_POST = empty_to_null($_POST);
		$dataChanged = $this->Folders->update($_POST, $id);
		if ($dataChanged) {$this->Events->internal('Projektdaten editiert', $id);}
		$this->view->redirect($_SERVER['HTTP_REFERER']);
	}

}
