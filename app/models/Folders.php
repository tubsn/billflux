<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\mvc\Model;
use \flundr\security\CryptLib;
use app\models\Attachment;
use app\models\Events;

class Folders extends Model
{

	public function __construct() {
		$this->db = new SQLdb(DB_SETTINGS);
		$this->db->table = 'folders';
		$this->db->orderby = 'date';
		$this->db->order = 'DESC';
	}

	public function list() {
		$folders = $this->all();
		$folders = array_map([$this,'json_fields_to_array'], $folders);
		return $folders;
	}

	public function find($query) {
		$folders = $this->search($query,['title','description']);

		if (empty($folders)) {
			$table = $this->db->table;
			$SQLstatement = $this->db->connection->prepare(
				"SELECT * FROM $table 
				WHERE JSON_SEARCH(LOWER(customer), 'one', LOWER(:query)) IS NOT NULL;"
			);
			$SQLstatement->execute([':query' => '%'.$query.'%']);
			$folders = $SQLstatement->fetchall();
		}

		if (empty($folders)) {return [];}
		return array_map([$this,'json_fields_to_array'], $folders);
	}

	public function copy($id) {

		$copy = $this->db->read($id);
		unset(
			$copy['id'],
			$copy['edited'],
			$copy['created'],
			$copy['phase'],
			$copy['invoice'],
			$copy['invoice_id'],
			$copy['revenue'],
			$copy['income'],
			$copy['commission'],
			$copy['bank'],
			$copy['attachments'],
		);

		$copy['status'] = 'geplant';
		$copy['title'] .= ' (Kopie)';

		return $this->db->create($copy);

	}


	public function type($name) {
		$folders = $this->search($name, 'type');
		if (empty($folders)) {$folders = $this->search(urldecode($name), 'issuer');}
		if (empty($folders)) {$folders = $this->search($name, 'office');}
		if (empty($folders)) {return false;}
		$folders = array_map([$this,'json_fields_to_array'], $folders);
		return $folders;
	}


	public function get_everything($id) {

		$folder = $this->get($id);
		if (!$folder) {return null;}

		$Events = new Events();
		$folder['events'] = $Events->stream($id);

		$folder['attachments'] = $this->get_attachments($folder['attachments']);
		$folder['customer'] = json_decode($folder['customer'] ?? '', true);
		$folder['object'] = json_decode($folder['object'] ?? '', true);
		$folder['usages'] = json_decode($folder['usages'] ?? '', true);
		$folder['hull'] = json_decode($folder['hull'] ?? '', true);
		$folder['renovation'] = json_decode($folder['renovation'] ?? '', true);
		$folder['analytics'] = json_decode($folder['analytics'] ?? '', true);
		$folder['appointments'] = json_decode($folder['appointments'] ?? '', true);
		$folder['bank'] = json_decode($folder['bank'] ?? '', true);
		return $folder;

	}


	public function distinct_types() {

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT DISTINCT type FROM $table WHERE type != '' LIMIT 0, 50"
		);

		$SQLstatement->execute();
		$output = $SQLstatement->fetchall(\PDO::FETCH_COLUMN);
		if (empty($output)) {return [];}
		return $output;

	}

	public function get_attachments($ids) {

		if (empty($ids)) {return null;}

		$attachmentDB = new Attachment();
		$attachments = $attachmentDB->get(explode(',', $ids));

		// Attachments always have to be an array
		if (isset($attachments['id'])) {$attachments = [$attachments];}
		$attachments = array_map([$attachmentDB,'map_filetypes'], $attachments);

		return $attachments;

	}

	public function remove_attachment($id, $attachmentID) {

		$attachmentDB = new Attachment();
		$attachmentDB->remove($attachmentID);

		$attachmentIDs = $this->get($id, ['attachments'])['attachments'];
		$attachmentIDs = remove_from_list($attachmentID, $attachmentIDs);

		$this->update(['attachments' => $attachmentIDs], $id);

	}


	public function tax_year($start = 'first day of january last year', $end = 'last day of december last year') {

		$start = date("Y-m-d", strtotime($start));
		$end = date("Y-m-d", strtotime($end));

		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT id, invoice_id, status, title, date_invoice, date_payment,  revenue, income, type, 
			 IF(revenue <> income, REPLACE(income, ',', '.') - REPLACE(revenue, ',', '.') , 0) AS mwst
			 FROM $table 
			WHERE `date_invoice` BETWEEN :lastYearStart AND :lastYearEnd
			ORDER BY `date_invoice` ASC"
		);

		$SQLstatement->execute([':lastYearStart' => $start, ':lastYearEnd' => $end]);
		$folder = $SQLstatement->fetchAll() ?? null;

		$folder = $this->to_german_number($folder, 'mwst');
		return $folder;

	}

	public function to_german_number($data, $field) {

		return array_map(function($set) {
			$set['mwst'] = gnum($set['mwst'],2);
			return $set; 
		}, $data);

	}

	public function amount_by($field = 'type') {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT $field, COUNT(*) as folders FROM $table WHERE `$field` is not null GROUP BY $field"
		);

		$SQLstatement->execute();
		$folder = $SQLstatement->fetchAll(\PDO::FETCH_UNIQUE|\PDO::FETCH_COLUMN) ?? null;
		
		return $folder;
	}

	public function folders_by($format = '%Y-%m') {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT DATE_FORMAT(`date`, '$format') as period, count(*) as count
			FROM $table
			GROUP BY period"
		);

		$SQLstatement->execute();
		$folder = $SQLstatement->fetchAll(\PDO::FETCH_UNIQUE|\PDO::FETCH_COLUMN) ?? null;
		
		return $folder;
	}

	public function revenue_by($format = '%Y-%m') {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT DATE_FORMAT(`date_invoice`, '$format') as period, sum(revenue) as revenue
			FROM $table WHERE `revenue` is not null AND `invoice` = 'bezahlt'
			GROUP BY period"
		);

		$SQLstatement->execute();
		$folder = $SQLstatement->fetchAll(\PDO::FETCH_UNIQUE|\PDO::FETCH_COLUMN) ?? null;
		
		return $folder;
	}

	public function potential_by($format = '%Y-%m') {
		$table = $this->db->table;
		$SQLstatement = $this->db->connection->prepare(
			"SELECT DATE_FORMAT(`date`, '$format') as period, sum(revenue) as revenue
			FROM $table WHERE `revenue` is not null AND `status` != 'abgelehnt'
			GROUP BY period"
		);

		$SQLstatement->execute();
		$folder = $SQLstatement->fetchAll(\PDO::FETCH_UNIQUE|\PDO::FETCH_COLUMN) ?? null;
		
		return $folder;
	}

	private function json_fields_to_array(array $data) {
		if (isset($data['customer'])) {$data['customer'] = json_decode($data['customer'],true);}
		return $data;
	}

}