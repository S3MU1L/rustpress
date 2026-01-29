CREATE TABLE IF NOT EXISTS site_templates (
    id           uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    name         text        NOT NULL UNIQUE,
    description  text        NOT NULL DEFAULT '',
    html         text        NOT NULL,
    is_builtin   boolean     NOT NULL DEFAULT false,
    created_at   timestamptz NOT NULL DEFAULT now(),
    edited_at    timestamptz NOT NULL DEFAULT now()
);

-- A couple of built-in templates.
INSERT INTO site_templates (name, description, html, is_builtin)
VALUES
(
  'default',
  'Default template (title + content). Placeholders: {{title}}, {{content}}',
  '<!doctype html>\n<html lang="en">\n  <head>\n    <meta charset="utf-8"/>\n    <meta name="viewport" content="width=device-width,initial-scale=1"/>\n    <title>{{title}} - RustPress</title>\n    <link rel="stylesheet" href="/static/app.css"/>\n  </head>\n  <body>\n    <header class="topbar">\n      <div class="container">\n        <a class="brand" href="/">RustPress</a>\n        <nav class="nav"><a href="/admin">Admin</a></nav>\n      </div>\n    </header>\n    <main class="container">\n      <article class="card">\n        <h1>{{title}}</h1>\n        <div class="prose">{{content}}</div>\n      </article>\n    </main>\n  </body>\n</html>',
  true
),
(
  'minimal',
  'Minimal template (no chrome). Placeholders: {{title}}, {{content}}',
  '<!doctype html>\n<html lang="en">\n  <head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/><title>{{title}}</title></head>\n  <body><h1>{{title}}</h1>{{content}}</body>\n</html>',
  true
)
ON CONFLICT (name) DO NOTHING;
