create virtual table revisions_fts using fts5(slug, body, page_id unindexed, tokenize = 'unicode61');
