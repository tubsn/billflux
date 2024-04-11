<?php
$routes->get('/', 'Folder@index');
$routes->get('/search', 'Search@search');

// Static Pages
$routes->get('/vorlagen', 'Staticpages@vorlagen');
$routes->get('/dashboards', 'Staticpages@dashboards');
$routes->get('/dashboards/invoices', 'Staticpages@invoices');
$routes->get('/kontakte', 'Staticpages@kontakte');
$routes->get('/kunden', 'Customers@index');
$routes->get('/test', 'Tests@index');

// External
$routes->get('/meinprojekt/{token}', 'Clientview@index');
$routes->post('/meinprojekt/{token}', 'Clientview@update');


// Tradeflux
$routes->get('/finance', 'Trade@finance');
$routes->get('/trade', 'Trade@index');

// Folder Management
$routes->post('/auftrag/{id:\d+}/upload[/{origin}]', 'Folder@upload');
$routes->get('/auftrag/{id:\d+}/copy', 'Folder@copy');
$routes->get('/auftrag/{id:\d+}/delete', 'Folder@delete');
$routes->get('/auftrag/{id:\d+}[/{slug}]', 'Folder@edit');
$routes->post('/auftrag/{id:\d+}[/{slug}]', 'Folder@update');

$routes->get('/buero/{name}', 'Folder@category');
$routes->get('/bearbeiter/{name}', 'Folder@category');

$routes->get('/new', 'Folder@create');
$routes->post('/new', 'Folder@save');

// Attachments
$routes->post('/attachment/client/{token:.{17,23}}/{attachmentID:\d+}', 'Clientview@update_attachment');
$routes->post('/attachment/{id:\d+}', 'Folder@update_attachment');
$routes->get('/attachment/{id:\d+}/remove-attachment/{attachmentID:\d+}', 'Folder@remove_attachment');
$routes->get('/attachment/{token:.{17,23}}/remove-attachment/{attachmentID:\d+}', 'Clientview@remove_attachment');

// KI Stuff
$routes->post('/ai/texts', 'AiGenerator@email');
$routes->post('/ai/extractaddress', 'AiGenerator@extract_address');

// API
$routes->get('/api/folder/{id:\d+}', 'Api@folder');
$routes->get('/api/folder/{id:\d+}/attachments', 'Api@folder_attachments');
$routes->get('/api/folder/{id:\d+}/edited', 'Api@folder_edited');
$routes->get('/api/folder/{id:\d+}/events', 'Api@folder_events');
$routes->post('/api/folder/{id:\d+}/status', 'Folder@update_status');
$routes->get('/api/orders', 'Api@list_orders');
$routes->get('/api/orders/add', 'Api@add_order');
$routes->get('/api/orders/{orderID:\d+}/delete', 'Api@delete_order');
$routes->post('/api/orders/{orderID:\d+}', 'Api@change_order');
$routes->get('/api/users/backoffice', 'Api@user_list_orders');



// Authentication Routes
$routes->get('/login', 'Authentication@login');
$routes->post('/login', 'Authentication@login');
$routes->get('/logout', 'Authentication@logout');
$routes->get('/profile', 'Authentication@profile');
$routes->get('/password-reset', 'Authentication@password_reset_form');
$routes->post('/password-reset', 'Authentication@password_reset_send_mail');
$routes->get('/password-change[/{resetToken}]', 'Authentication@password_change_form');
$routes->post('/password-change[/{resetToken}]', 'Authentication@password_change_process');
$routes->get('/profile/edit', 'Authentication@edit_profile');
$routes->post('/profile/edit', 'Authentication@edit_profile');

// Usermanagement / Admin Routes
$routes->get('/admin', 'Usermanagement@index');
$routes->get('/admin/new', 'Usermanagement@new');
$routes->post('/admin', 'Usermanagement@create');
$routes->get('/admin/{id:\d+}', 'Usermanagement@show');
$routes->get('/admin/{id:\d+}/delete/{token}', 'Usermanagement@delete');
$routes->post('/admin/{id:\d+}', 'Usermanagement@update');
