<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\mvc\Model;

class Users extends Model
{

	public function __construct() {
		$this->db = new SQLdb(USER_DB_SETTINGS);
		$this->db->table = 'users';

		$this->db->columns = ['id','edited','created','email','firstname','lastname','`groups`','rights'];
		$this->db->protected = ['level','groups','rights']; // By Default these can't be changed		
	}

	public function by_id($id) {
		return $this->get($id, $this->db->columns);
	}

	public function list_of_names() {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT id, CONCAT_WS(' ', firstname,lastname) as name FROM $table"
		);
		$SQLstatement->execute();
		$output = $SQLstatement->fetchall(\PDO::FETCH_UNIQUE|\PDO::FETCH_COLUMN);
		if (empty($output)) {return [];}
		return $output;
	}

	public function with_group($group) {
		return $this->find_in($group, 'groups');
	}

	public function with_right($right) {
		return $this->find_in($right, 'rights');
	}

	private function find_in($value, $field = 'groups') {

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT id, email, CONCAT_WS(' ', firstname,lastname) as name, firstname, lastname FROM $table 
			WHERE FIND_IN_SET(:value, REPLACE(REPLACE(`$field`, ', ', ','), ' ,', ',')) <> 0"
		);

		$SQLstatement->execute([':value' => $value]);
		$output = $SQLstatement->fetchall();

		if (empty($output)) {return [];}
		return $output;

	}

}