<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\mvc\Model;

class Customers extends Model
{

	public function __construct() {

		$this->db = new SQLdb(DB_SETTINGS);
		$this->db->table = 'folders';

	}


	public function list() {

		$jsonCustomers = $this->all(['id', 'customer']);

		$customers = array_map(function($set) {
			$customer = json_decode($set['customer'],1);
			$customer['id'] = $set['id'];
			return $customer;
		}, $jsonCustomers);


		$customers = array_filter($customers, function($customer) {
			if (isset($customer['email']) 
				|| isset($customer['fistname']) 
				|| isset($customer['lastname'])) {
				return $customer;
			}
		});

		return $customers;

	}

	public function customer($id) {

	}

	public function load($id) {

	}

}