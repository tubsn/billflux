<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\auth\Auth;
use \flundr\mvc\Model;

class Events extends Model
{

	public function __construct() {
		$this->db = new SQLdb(DB_SETTINGS);
		$this->db->table = 'events';
		$this->db->primaryIndex = 'folder_id';
	}

	public function category($category) {

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT category, folder_id, content, user_id, DATE_FORMAT(`date`, '%Y-%m-%d') as `day`, DATE_FORMAT(`date`, '%H:%i') as `time` FROM $table
			 WHERE category = :category ORDER BY date DESC"
		);
		$SQLstatement->execute([':category' => $category]);
		$output = $SQLstatement->fetchAll(\PDO::FETCH_GROUP);
		return $output;

	}

	public function find($search, $category = '') {

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT category, folder_id, content, user_id, DATE_FORMAT(`date`, '%Y-%m-%d') as `day`, DATE_FORMAT(`date`, '%H:%i') as `time` FROM $table
			 WHERE content like :search AND category like :category
			 ORDER BY date DESC LIMIT 30"
		);
		$SQLstatement->execute([':search' => '%%'.$search.'%%', ':category' => '%%'.$category.'%%']);
		$output = $SQLstatement->fetchAll(\PDO::FETCH_GROUP);
		return $output;

	}


	public function grouped($id) {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT category, folder_id, content, user_id, DATE_FORMAT(`date`, '%Y-%m-%d') as `day`, DATE_FORMAT(`date`, '%H:%i') as `time` FROM $table
			 WHERE folder_id = :id ORDER BY date DESC"
		);
		$SQLstatement->execute([':id' => $id]);
		$output = $SQLstatement->fetchAll(\PDO::FETCH_GROUP);
		return $output;
	}

	public function stream($id) {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT category, folder_id, content, user_id, DATE_FORMAT(`date`, '%Y-%m-%d') as `day`, DATE_FORMAT(`date`, '%H:%i') as `time` FROM $table
			 WHERE folder_id = :id ORDER BY date DESC"
		);
		$SQLstatement->execute([':id' => $id]);
		$output = $SQLstatement->fetchAll();
		return $output;
	}

	public function reset($id) {
		$this->delete($id);
	}

	public function log($content = null, $folderID = null, $category = null) {
		$data['folder_id'] = $folderID;
		$data['user_id'] = Auth::get('id');
		$data['content'] = $content;
		$data['category'] = $category;
		$this->create($data);
	}

	public function customer($content, $folderID) {
		$this->log($content, $folderID, 'customer');
	}	

	public function download($content, $folderID) {
		$this->log('Download - '. $content, $folderID, 'customer');
	}

	public function internal($content, $folderID) {
		$this->log($content, $folderID, 'internal');
	}	

	public function phase($content, $folderID) {
		$this->log($content, $folderID, 'phase');
	}

	public function status($content, $folderID) {
		$this->log($content, $folderID, 'status');
	}

	public function invoice($content, $folderID) {
		$this->log($content, $folderID, 'invoice');
	}

}