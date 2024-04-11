<?php

namespace app\models;
use \flundr\database\SQLdb;
use \flundr\mvc\Model;
use \flundr\file\Storage;
use app\models\Folders;

class Attachment extends Model
{

	private $storage;

	public function __construct() {

		$this->db = new SQLdb(DB_SETTINGS);
		$this->db->table = 'attachments';
		$this->storage = new Storage();
		$this->storage->thumbnails = true;
		$this->storage->maxSize = 50*1024*1024;

		$this->storage->formats = [
			"gif","eps","ai","jpg","jpeg","svg","png",
			"pdf","pptx","indd","idml","eps","tif","txt",
			"doc","docx","rtf","xls","csv","xlsx",
			"psd", "zip", "rar", "webp", "mp3"
		];

	}

	public function map_filetypes($attachment) {

		$attachment['target'] = '_self';

		// Use Icons for specific Files
		switch (strtolower($attachment['type'])) {

			case "image/jpeg": case "image/gif": case "image/png": case "image/webp":
				$attachment['thumbnail'] = $attachment['thumbnail'] ?? '/styles/flundr/img/no-thumb.svg';
				$attachment['target'] = '_blank';
				break;

			case "application/pdf":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-pdf.svg';
				$attachment['target'] = '_blank';
				break;

			case "application/postscript":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-eps.svg';
				break;

			case "image/tiff":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-tif.svg';
				break;

			case "application/octet-stream":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-psd.svg';
				break;

			case "text/plain":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-txt.svg';
				$attachment['target'] = '_blank';
				break;

			case "application/vnd.openxmlformats-officedocument.wordprocessingml.document":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-doc.svg';
				break;

			case "application/vnd.openxmlformats-officedocument.presentationml.presentation":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-ppt.svg';
				break;

			case "application/x-zip-compressed":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-zip.svg';
				break;

			case "application/zip":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-zip.svg';
				break;

			case "audio/mpeg":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-mp3.svg';
				$attachment['target'] = '_blank';
				break;

			case "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet":
				$attachment['thumbnail'] = '/styles/flundr/img/icon-xls.svg';
				break;

			default:
				$attachment['thumbnail'] = '/styles/flundr/img/icon-file.svg';
		}


		if ($attachment['size'] > 1024*1024) {$attachment['size'] = round($attachment['size'] / 1024 / 1024, 1) . 'mb';}
		else {$attachment['size'] = round($attachment['size'] / 1024) . 'kb';}

		return $attachment;

	}

	public function upload($data, $id, $origin = 'home') {

		$this->storage->folder = 'uploads/' . $id . '/' .$origin;
		$this->storage->store($data);

		$failed = $this->storage->failed();
		$stored = $this->storage->stored();

		$newAttachmentIDs = [];
		foreach ($stored as $index => $file) {
			$file['origin'] = $origin;

			if (($file['type'] == 'image/png' || 'image/jpeg')) {
				$sizes = getimagesize(PUBLICFOLDER . $file['url']);
				$file['width'] = $sizes[0] ?? null;
				$file['height'] = $sizes[1] ?? null;
			}

			$attachmentID = $this->create($file);
			$stored[$index]['id'] = $attachmentID;
			array_push($newAttachmentIDs, $attachmentID);
		}

		$Folders = new Folders();
		$oldAttachments = $Folders->get($id)['attachments'];

		if (!empty($oldAttachments)) {
			$oldAttachmenIDs = explode(',', $oldAttachments);
			$newAttachmentIDs = array_merge($oldAttachmenIDs, $newAttachmentIDs);
		}

		$newAttachments = implode(',' ,$newAttachmentIDs);
		$Folders->update(['attachments' => $newAttachments], $id);

		return ['failed' => $failed, 'stored' => $stored];

	}

	public function remove($id) {

		$files = $this->get($id,['url','thumbnail']) ?? [];
		if (!$files) {return false;}

		foreach ($files as $file) {

			if (!$file) {continue;}

			$file = ltrim($file, '/');
			if (!file_exists($file)) {continue;}
			
			unlink($file);

		}

		return $this->delete($id);

	}


}