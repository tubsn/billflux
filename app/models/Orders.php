<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\auth\Auth;
use \flundr\mvc\Model;
use \app\models\Users;

class Orders extends Model
{

	public function __construct() {
		$this->db = new SQLdb(DB_SETTINGS);
		$this->db->table = 'orders';
		//$this->db->primaryIndex = 'folder_id';
	}

	public function by_folder($id) {

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT * FROM $table
			 WHERE folder_id = :folderID"
		);
		$SQLstatement->execute([':folderID' => $id]);
		$output = $SQLstatement->fetchAll();
		return $output;

	}

	/*
	public function category($category) {

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT category, folder_id, content, user_id, DATE_FORMAT(`date`, '%Y-%m-%d') as `day`, DATE_FORMAT(`date`, '%H:%i') as `time` FROM $table
			 WHERE category = :category"
		);
		$SQLstatement->execute([':category' => $category]);
		$output = $SQLstatement->fetchAll(\PDO::FETCH_GROUP);
		return $output;

	}
	*/

}