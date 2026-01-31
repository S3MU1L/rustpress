-- Built-in templates were originally inserted using literal `\n` sequences inside
-- standard string literals (Postgres does not treat `\n` as a newline by default).
-- This migration normalizes the built-ins so previews/pages don't show `\n` text.

-- Rewrite known built-ins explicitly (best-effort idempotent).
UPDATE site_templates
SET
    html = $$<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width,initial-scale=1"/>
    <title>{{title}} - RustPress</title>
    <link rel="stylesheet" href="/static/app.css"/>
  </head>
  <body>
    <header class="topbar">
      <div class="container">
        <a class="brand" href="/">RustPress</a>
        <nav class="nav"><a href="/admin">Admin</a></nav>
      </div>
    </header>
    <main class="container">
      <article class="card">
        <h1>{{title}}</h1>
        <div class="prose">{{content}}</div>
      </article>
    </main>
  </body>
</html>$$,
    edited_at = now()
WHERE name = 'default' AND is_builtin = true;

UPDATE site_templates
SET
    html = $$<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width,initial-scale=1"/>
    <title>{{title}}</title>
  </head>
  <body>
    <h1>{{title}}</h1>
    {{content}}
  </body>
</html>$$,
    edited_at = now()
WHERE name = 'minimal' AND is_builtin = true;

-- Safety net: for any other built-ins that still contain literal "\\n" or "\\t",
-- convert those sequences into actual newline/tab characters.
UPDATE site_templates
SET
    html = replace(replace(html, E'\\\\n', E'\n'), E'\\\\t', E'\t'),
    edited_at = now()
WHERE is_builtin = true
  AND (html LIKE '%\\n%' OR html LIKE '%\\t%');
