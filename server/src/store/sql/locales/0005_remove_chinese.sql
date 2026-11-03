delete from revisions_fts where page_id in (select id from pages where lang = 'zh');
delete from proposals where page_id in (select id from pages where lang = 'zh');
delete from revisions where page_id in (select id from pages where lang = 'zh');
delete from pages where lang = 'zh';
