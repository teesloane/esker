pub const CONFIG_YAML: &str = r#"# site-wide configuration:
# your sites url
url: "http://localhost:8080"

# site title
title: "My Site"

# your site description.
description: "My Site Description"

# name of the directory where you store attachments
attachment_directory: "attachments"

# directories to ignore
ignored_directories: ["dailies", "jots", "work", "media", "templates"]

# the url you wish to use for grouping "tags" under
tags_url: "tags"
"#;

pub const BASE_HTML: &str = r#"<html>
  <head>
    <meta charset="utf-8">
    <title>My Site - {% block title %} {% endblock title %}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="{{config.title}}" content="{{config.description}}" />
    <link rel="apple-touch-icon" href="/apple-touch-icon.png">
    <script src="{{baseurl}}/public/js/main.js"></script>
    <link rel="stylesheet" href="{{baseurl}}/public/css/syntax-theme-dark.css" type="text/css" media="screen" />
    <link rel="stylesheet" href="{{baseurl}}/public/css/syntax-theme-light.css" type="text/css" media="screen" />
    <link rel="stylesheet" href="{{baseurl}}/public/css/main.css" type="text/css" media="screen" />
    <style>
    </style>
  </head>

  <body style="display: flex;">
    <main style="display: flex;">
      <div style="display: flex; flex-direction: column; flex: 1;">
        <article>
          {% if page.title %}
          <h1> {{page.title}}</h1>
          {% endif %}
          {% block content %} {{page.content}} {% endblock content %}
        </article>

        {% if page.backlinks  %}
        {% if page.backlinks | length > 0 %}
        <h2> Backlinks </h2>
        <ul>
          {% for bl in page.backlinks %}
          <li><a href="{{bl.originating_file_url}}">{{bl.originating_file_title}}</a></li>
          {% endfor %}
        </ul>
        {% endif %}
        {% endif %}
      </div>

      <aside style="display: flex; flex-direction: column; margin-left: 64px;">

        {% if page %}
        <h3> TOC </h3>
        <ul>
          {% for link in page.toc %}
          <li><a href="{{link.url}}">{{link.title}}</a></li>
          {% endfor %}
        </ul>
        {% endif %}

        <h3> Tags </h3>
        <ul>
          {% for tag, tagged_items in tags %}
          <li><a href="{{baseurl}}/tags/{{tag}}.html">{{tag}} ({{tagged_items | length }})</a></li>
          {% endfor %}
        </ul>

        {% if page %}
          {% if page.related_files | length > 0 %}
          <h2>Similarly tagged</h2>
          <ul class="">
            {% for related_link in page.related_files %}
            <li><a href="{{related_link.url}}">{{related_link.title}}</a></li>
            {% endfor %}
          </ul>
          {% endif %}
        {% endif %}

        <h3> Sitemap </h3>
        <ul>
          {% for link in sitemap %}
          <li><a href="{{link.url}}">{{link.title}}</a></li>
          {% endfor %}
        </ul>
      </aside>

    </main>
  </body>
  <script>
    window.x = {{__tera_context}};
  </script>

</html>


"#;

pub const DEFAULT_HTML: &str = r#"{% extends "base.html" %}
{% block title %} {{page.title}} {% endblock title %}
"#;

pub const LIST_HTML: &str = r#"{% extends "base.html" %}
{% block title %} {{page.title}} {% endblock title %}

{% block content %}
  {{super()}}
  <ol reversed>
      {% for page in section.pages | sort(attribute="date_created_timestamp") %}
        <li style="list-style-type: none; margin-bottom: 32px">
          <a href="{{page.url}}"> <h3>{{page.title}}</h3> </a>
          <i>{{page.summary}}</i>
          <div class="text-sm text-alt">{{page.date_created}}</div>
        </li>
      {% endfor %}
  </ol>

{% endblock content %}
"#;

pub const RSS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">
    <channel>
      <title>{{ config.title }}</title>
        <link>{{ config.url }}</link>
        <description>{{ config.description }}</description>
        <generator>Esker</generator>
        {% for page in pages %}
          {% if page.is_section == false %}
          <item>
              <title>{{ page.title }}</title>

              <pubDate>{{ page.date_created_timestamp | date(format="%a, %d %b %Y %H:%M:%S %z", timezone="EST")}}</pubDate>
              <link>{{ page.url | escape_xml | safe }}</link>
              <guid>{{ page.url | escape_xml | safe }}</guid>
              <description>
                {% if page.summary %}
                  {{ page.summary }}
                {% else %}
                  {{ page.content | escape_xml | safe }}
                {% endif %}
              </description>
          </item>
          {% endif %}
        {% endfor %}
    </channel>
</rss>
"#;


pub const TAGS_HTML: &str = r#"{% extends "base.html" %}
{% block title %} Tags {% endblock title %}
{% block content %}
      <h3>{{tag}}</h3>
      <ul>
      {% for link_tag in tags[tag] %}
        <li>
          <a href={{link_tag.url}}>{{link_tag.title}} </a>
        </li>
      {% endfor %}
      </ul>
{% endblock content %}
"#;

pub const DEFAULT_JS: &str = r#"
"#;

pub const DEFAULT_CSS: &str = r#"

@import url("syntax-theme-dark.css") (prefers-color-scheme: dark);
@import url("syntax-theme-light.css") (prefers-color-scheme: light);


:root {
  --bg: #efefef;
  --color: #333;
  --color-alt: #666;
  --link-color: #2980b9;
}

/* CSS Variables (dark mode) */
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #111;
    --color: #dfdfdf;
    --color-alt: #666;
    --link-color: #fdcb6e;
  }
}


body {
  color: var(--color);
  background-color: var(--bg);
  max-width: 48em;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  font-family: "Charter", Arial;
  padding: 64px 0;
}

.footnote-definition {
  display: flex;
  align-items: baseline;
}

.footnote-definition-label {
  margin-right: 16px;
}


pre {
  overflow: scroll;
  border: 1px solid #dfdfdf;
  padding: 16px;
  margin: 24px 0;
}

a {
  color: var(--link-color);
text-decoration: none;
}

a:visited {
  color: var(--color);
}


section {
  padding-bottom: 32px;
}

ul, ol {
  padding-left: 16px;
}

img { max-width: 100%; }

.text-sm {font-size: 12px;}
.text-md {font-size: 16px;}

.text-alt {
  color: var(--color-alt);
}
"#;
