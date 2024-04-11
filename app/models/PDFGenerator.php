<?php

namespace app\models;

class PDFGenerator
{

	public function test() {

		$docraptor = new \DocRaptor\DocApi();
		# this key works in test mode!
		$docraptor->getConfig()->setUsername("YOUR_API_KEY_HERE");    // D6-ROpEYixuGTKP1AvLZ

		try {
			$doc = new \DocRaptor\Doc();
			$doc->setTest(true); # test documents are free but watermarked
			$doc->setDocumentType("pdf");
			$doc->setDocumentContent('


<main>

<style>
	.invoice {
		box-sizing: border-box;

		width:210mm;
		height:297mm;
		margin:0 auto;
		font-size:16px;
		position:relative;
	}

	.customer-adress {white-space: pre;}

	.invoice-footer {
		position:absolute;
		bottom:0;
		left:0;
		width:100%;
		box-sizing: border-box;
		padding:20mm;
		font-size:0.8em; line-height:120%;
		display:flex;
		justify-content: space-between;
	}

	.invoice-table {margin-top:3em; font-size:0.9em;}

</style>

<button class="button fright" type="button">als PDF Speichern (geht noch nicht)</button>

<section class="invoice" contenteditable="true"> 

<p style="margin-bottom:5em">
MEF MyEnergy.Farm GmbH<br/>
Prießnitzstr. 52, 01099 Dresden<br/>
Web: www.myenergy.farm<br/>
</p>

<h1>Rechnung Solarpark Buxtehude ID: 00078</h1>

<p>Lorem Ipsum blabla</p>

<div class="customer-adress">
Dresdener Stadtwerke 
John Cena 
03050 Buxtehude</div>

<table class="invoice-table fancy wide">
<tr>
	<th>Beschreibung</th>
	<th>Anzahl</th>
	<th>Einheit</th>
	<th>Einzelpreis</th>
	<th>Preis</th>
</tr>
<tr>
	<td>Solarpark BJ: 2021</td>
	<td>1</td>
	<td>#</td>
	<td>5000</td>
	<td>5000</td>
</tr>

</table>

<div class="fright">
	<b>Gesamtpreis: 5000€</b>
</div>


<div class="invoice-footer">
<p>
Geschäftsführer:<br>
Ole Renner<br>
Sitz der Gesellschaft:<br>
Dresden
</p>

<p>
Handelsregister:<br>
Amtsgericht Dresden<br>
HRB 41591<br>
USt.ID: DE348559640
</p>

<p>
Bankverbindung:<br>
GLS Bank<br>
BIC GENODEM1GLS<br>
IBAN DE58430609671237094900
</p>

</div>


</section>
</main>






				');
			# $doc->setDocumentUrl("https://docraptor.com/examples/invoice.html");
			# $doc->setJavascript(true);
			# $prince_options = new DocRaptor\PrinceOptions();
			# $doc->setPrinceOptions($prince_options);
			# $prince_options->setMedia("print"); # @media 'screen' or 'print' CSS
			# $prince_options->setBaseurl("https://yoursite.com"); # the base URL for any relative URLs

			$response = $docraptor->createDoc($doc);

			# createDoc() returns a binary string
			file_put_contents(APP . 'example-invoice1.pdf', $response);
			echo 'Successfully created example-invoice.pdf!';
		} catch (\DocRaptor\ApiException $error) {
			echo $error . "\n";
			echo $error->getMessage() . "\n";
			echo $error->getCode() . "\n";
			echo $error->getResponseBody() . "\n";
		}


	}

}
