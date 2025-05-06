SELECT name, mtime, length(data)
FROM fsdir(:source_dir)
WHERE data IS NOT NULL;
