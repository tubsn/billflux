<?php

namespace app\views;
use \flundr\mvc\views\htmlView;

class DefaultLayout extends htmlView {

	// Page Header Information is available in the Templates
	// as a $page Array. It can be accessed via $page['title']

	public $title = APP_NAME;
	public $description = 'The Finance solution!';
	public $css = ['/styles/flundr/css/defaults.css', '/styles/css/main.css', '/styles/css/photoswipe.css'];
	public $fonts = 'https://fonts.googleapis.com/css?family=Fira+Sans:400,400i,600|Fira+Sans+Condensed:400,600';
	public $js = ['/styles/js/main.js', '/styles/js/tablesort.js'];
	public $framework = ['/styles/js/vendors/vue34.min.prod.js','/styles/flundr/components/fl-dialog.js', '/styles/js/wyld-upload.js' , '/styles/js/vendors/photoswipe-lightbox.umd.min.js', '/styles/js/vendors/photoswipe.umd.min.js', '/styles/js/components/vue-upload.js', '/styles/js/components/vue-dialog.js', '/styles/js/components/vue-trades.js'];
	public $meta = [
		'author' => 'flundr',
		'robots' => 'noindex, nofollow',
		'favicon' => '/styles/img/cash-flow.svg',
	];




	// You can add some "default" Variables to the Template
	// which can be easily overwritten in the Controller by setting view->varname
	// One usage example could be the path to a template of a Subnavigation for that Page
	// which you can include by using the tpl() helper function "include tpl($navigation)"

	public $templateVars = [
		'layout' => 'internal',
		'navigation' => 'nav/nav-overview',
	];

	// Place the Templateblocks to build your Page here.
	// The "main" Section is usually overwritten in the Controller in the Render function.
	// You can add as many template Blocks as you like or none, if you are just using one "main" template.

	public $templates = [
		'tinyhead' => 'layout/html-doc-header',
		'header' => 'navigation/main-nav',
		'main' => null,
		'footer' => null,
		'tinyfoot' => 'layout/html-doc-footer',
	];

}
