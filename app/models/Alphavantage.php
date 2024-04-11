<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\mvc\Model;
use \flundr\cache\RequestCache;

class Alphavantage extends Model
{

	private $apikey = ALPHA_APIKEY;

	public function __construct() {

		$this->db = new SQLdb(DB_SETTINGS);
		$this->db->table = 'stocks';

	}

	public function days($wkin = null) {

		$filter = '';
		if ($wkin) {
			$filter = "WHERE wkin = '" . $wkin . "'";
		}

		$SQLstatement = $this->db->connection->prepare(
			"SELECT DAYNAME(`date`) AS weekday, round(avg(close),2) as avgclose FROM `stocks` 
			$filter
			#AND date > '2024-01-01'
			GROUP BY weekday
			ORDER BY avgclose DESC"
		);
		$SQLstatement->execute();
		$output = $SQLstatement->fetchall(\PDO::FETCH_COLUMN|\PDO::FETCH_UNIQUE);
		return $output;

	}

	public function distinct() {

		$SQLstatement = $this->db->connection->prepare(
			"SELECT distinct(wkin) FROM `stocks`"
		);
		$SQLstatement->execute();
		$output = $SQLstatement->fetchall(\PDO::FETCH_COLUMN);
		return $output;

	}




	public function wkin($name = 'NVDA') {
		$url = 'https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol='.
		$name. '&apikey='. $this->apikey;

		$cache = new RequestCache('stock-' . $name, 60 * 60);
		$cachedData = $cache->get();
		if ($cachedData) {return $cachedData;}

		$data = $this->curl($url);
		$cache->save($data);

		return $data;
	}

	public function import_to_db($data) {

		if (!isset($data['Time Series (Daily)']) || empty($data['Time Series (Daily)'])) {
			return;
		}

		$name = $data['Meta Data']['2. Symbol'];
		$data = $data['Time Series (Daily)'];

		foreach ($data as $day => $set) {
			$set['wkin'] = $name;
			$set['date'] = $day;
			$set['open'] = $set['1. open'];	unset($set['1. open']);
			$set['high'] = $set['2. high'];	unset($set['2. high']);
			$set['low'] = $set['3. low'];	unset($set['3. low']);
			$set['close'] = $set['4. close'];	unset($set['4. close']);
			$set['volume'] = $set['5. volume'];	unset($set['5. volume']);
			$this->create($set);
		}

	}


	private function curl($url) {

		$ch = curl_init();
		curl_setopt ($ch, CURLOPT_URL, $url);
		curl_setopt ($ch, CURLOPT_SSL_VERIFYPEER, 1);
		curl_setopt ($ch, CURLOPT_SSL_VERIFYHOST, 2);
		curl_setopt ($ch, CURLOPT_HEADER, 0);
		curl_setopt ($ch, CURLOPT_RETURNTRANSFER, 1);

		$recievedData = curl_exec($ch);
		if ($recievedData === false) {
			dd(curl_error($ch));
		}

		curl_close ($ch);

		return json_decode($recievedData, true);

	}


}
