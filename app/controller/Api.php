<?php

namespace app\controller;
use flundr\mvc\Controller;
use flundr\auth\Auth;

class Api extends Controller {

	public function __construct() {
		if (!Auth::logged_in() && !Auth::valid_ip()) {Auth::loginpage();}		
		$this->view('DefaultLayout');
		$this->models('Folders,Attachment,Events,Users,Orders');
	}

	public function folder($id) {
		$data = $this->Folders->get_everything($id);
		$this->view->json($data);
	}

	public function folder_attachments($id) {
		$data = $this->Folders->get_everything($id);
		if (empty($data)) {throw new \Exception("Not found", 404);}

		$this->view->json($data['attachments']);
	}

	public function folder_events($id) {
		$this->view->json($this->Events->stream($id));
	} 

	public function folder_edited($id) {
		$data = $this->Folders->get($id,['edited']);
		if (empty($data)) {throw new \Exception("Not found", 404);}
		
		if (empty($data['edited'])) {
			$this->view->json(['edited' => null]);
			return;
		}
		$this->view->json(['edited' => $data['edited']]);
	}

	public function list_orders() {
		$this->view->json($this->Orders->all());
	}

	public function user_list_orders() {
		$berater = $this->Users->with_group('Berater');
		$backoffice = $this->Users->with_group('Backoffice');
		$users = array_merge($berater, $backoffice);

		// Hack to remove duplicates... :)
		$users = array_map("unserialize", array_unique(array_map("serialize", $users)));
		$this->view->json($users);
	}

	public function add_order() {
		$newID = $this->Orders->create(['indate'=> date('Y-m-d H:i:s')]);
		$this->view->json(['id' => $newID]);
	}

	public function delete_order($orderID) {
		$this->Orders->delete($orderID);
	}

	public function change_order($orderID) {
		$status = $this->Orders->update($_POST, $orderID);
		$this->view->json(['status' => $status]);
	}




}
