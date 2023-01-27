use colored::*;
use std::collections::HashMap;
use std::fs;
use std::{env, path::PathBuf};

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


# Add the name of a theme if you are using
# to use a theme, you must have it in your themes directory, for ex:
# _esker/themes/<theme_name>

# theme: "<my_theme_name>"
"#;

pub const BASE_HTML: &str = r##"<html>
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

  <body class="flex">
    <nav>
      <div class="site-name">///// {{config.title}} </div>
      <ul class="flex">
        <li><a class="nav-link" href="#replace_me">Post</a></li>
        <li><a class="nav-link" href="#replace_me">About</a></li>
        <li><a class="nav-link" href="#replace_me">Feed</a></li>
      </ul>
    </nav>

    <div class="flex">
    <main class="flex">
      <div class="flex-col flex-1">
        <article>
          {% if page.title %}
            <h1 class="page-title"> {{page.title}}</h1>
          {% endif %}
          {% block content %} {{page.content}} {% endblock content %}
        </article>
      </div>


    </main>

      <aside class="sidebar">
        {% if page %}
          {% if page.toc | length > 0 %}
          <h4> Contents </h4>
          <ul style="list-style-type: none">
            {% for link in page.toc %}
            {% set hlvl = link.link_type.Toc.heading_level -%}
            {% set indent_width = hlvl * 4 -%}
            {% if hlvl == 1 or hlvl == 2 %}
              <li><a  href="{{link.url}}">{{link.title}}</a></li>
            {% else %}
              <li style="margin-left: {{indent_width}}px"><a href="{{link.url}}">{{link.title}}</a></li>
            {% endif %}
            {% endfor %}
          </ul>
          {% endif %}

        {% if page.backlinks | length > 0 %}
          <h4> Backlinks </h4>
          <ul>
            {% for link in page.backlinks %}
            <li><a href="{{link.url}}">{{link.title}}</a></li>
            {% endfor %}
          </ul>
        {% endif %}

          {% if page.related_files | length > 0 %}
          <h4>Similarly tagged</h2>
          <ul class="">
            {% for related_link in page.related_files %}
            <li><a href="{{related_link.url}}">{{related_link.title}}</a></li>
            {% endfor %}
          </ul>
          {% endif %}
        {% endif %}
      </aside>
    </div>
  </body>
  <script>
    window.x = {{__tera_context}};
  </script>

</html>

"##;

pub const SINGLE_HTML: &str = r#"{% extends "base.html" %}
{% block title %} {{page.title}} {% endblock title %}
"#;

pub const LIST_HTML: &str = r#"{% extends "base.html" %}
{% block title %} {{page.title}} {% endblock title %}

{% block content %}
  {{super()}}
  <ol reversed>
      {% for page in section.pages | sort(attribute="date_created_timestamp") %}
        <li>
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

pub const DEFAULT_CSS: &str = r#"@import url("syntax-theme-dark.css") (prefers-color-scheme: dark);
@import url("syntax-theme-light.css") (prefers-color-scheme: light);

:root {
  --bg: rgb(253, 253, 253);
  --bg-alt: rgb(253, 253, 253);
  --border-col: rgb(70, 70, 70);
  --color: #333;
  --color-alt: #666;
  --link-color: #2980b9;
  --font: "Avenir", "Arial";
}

/* CSS Variables (dark mode) */
@media (prefers-color-scheme: dark) {
  :root {
    --bg: rgb(33, 33, 39);
    --bg-alt: rgb(28, 28, 38);
    --border-col: rgb(25, 25, 25);
    --color: #dfdfdf;
    --color-alt: #666;
    --link-color: #fdcb6e;
    --font: "Avenir", "Arial";
  }
}


body {
  color: var(--color);
  background-color: var(--bg);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  font-family: var(--font);
  font-size: 16px;
  max-width: 1920px;
  margin: 0 auto;
}

.footnote-definition {
  display: flex;
  align-items: baseline;
}

.footnote-definition-label {
  margin-right: 16px;
}

nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 32px;
  border-bottom: 2px solid var(--border-col);
  background-color: var(--bg);
}

nav ul {
  list-style-type: none;
}

.nav-link {
  margin-left:16px;
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

main {
  max-width: 60em;
  width: 100%;
  margin: 0 auto;
  margin-top: 48px;
  padding: 0 32px;
}

article {
  font-size: 17px;
  max-width: 36em;
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

.sidebar {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  margin-right: 80px;
  padding: 32px;
  position: sticky;
  top: 20px;
  height: 100vh;
  overflow: auto;

}

@media(max-width: 1200px) {
  .sidebar {
    display: none;
  }
}

.page-title {
  margin-bottom: 64px;
}

/* -- utility classes -- */

.flex {
  display: flex;
}

.flex-col {
  display: flex;
  flex-direction: column;
}

.flex-1 {
  flex: 1;
}
"#;

pub fn init(dir: Option<PathBuf>) {
    let cwd: PathBuf;
    if let Some(dir) = dir {
        cwd = dir;
    } else {
        cwd = env::current_dir().unwrap();
    }

    let dir_esker = cwd.join("_esker");
    if fs::metadata(&dir_esker).is_ok() {
        println!(
            "{}: An '_esker' site already exists in this directory.",
            " Failed ".yellow().on_black()
        );
    } else {
        let dirs = vec![
            "templates/",
            "templates/partials",
            "sass",
            "public/css",
            "public/js",
            "_site",
        ];

        let mut files = HashMap::new();
        files.insert(String::from("public/js/main.js"), DEFAULT_JS);
        files.insert(String::from("public/css/main.css"), DEFAULT_CSS);
        files.insert(String::from("templates/base.html"), BASE_HTML);
        files.insert(String::from("templates/single.html"), SINGLE_HTML);
        files.insert(String::from("templates/tags.html"), TAGS_HTML);
        files.insert(String::from("templates/list.html"), LIST_HTML);
        files.insert(String::from("templates/feed.rss"), RSS_XML);
        files.insert(String::from("config.yaml"), CONFIG_YAML);

        // Map over the above strings, turn them into paths, and create them.
        for &dir in &dirs {
            let joined_dir = dir_esker.join(dir);
            fs::create_dir_all(joined_dir).expect("Couldn't create a new firn, directory");
        }

        for (filename, file_contents) in files {
            let joined_dir = dir_esker.join(filename);
            fs::write(joined_dir, file_contents).expect("Unable to write new site layout files.");
        }

        println!(
            "{}: created a new esker site at: {:?}",
            " Success ".green().on_black(),
            dir_esker
        );
    }
}
