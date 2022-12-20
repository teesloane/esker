use std::{fs, path::PathBuf};

use crate::frontmatter::Frontmatter;
use crate::link::Link;
use crate::site::Site;
use crate::templates;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use slugify::slugify;
use tera::Context;
use std::io;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct MdFile {
    raw: String,
    html: String,
    path: PathBuf,
    web_path: PathBuf,
    out_path: PathBuf,
    frontmatter: Frontmatter,
    full_url: String,
}

impl MdFile {
    pub fn new(site: &Site, raw_str: String, path: PathBuf, fm: Frontmatter) -> MdFile {
        // Life's too short to write good code:

        // Below we set up the file's "out" path for writing the eventual html file to disk
        // AS WELL as the "web_path" which is the part that follow your domain name: <mydomain.com>/this-is-the-web-path.

        // turns; /Users/tees/development/tees/esker/test-site/foo/bar.md" -> test-site/foo/
        let web_parent_paths = path
            .strip_prefix(site.dir.clone())
            .unwrap()
            .parent()
            .unwrap();

        let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
        let mut out_file_path_slugified = slugify!(&filename);
        // takes slugified file name and adds html extension
        let web_path = PathBuf::from(out_file_path_slugified.clone()).with_extension("html");
        let web_path_str = web_path.clone().into_os_string().into_string().unwrap();
        let out_path =
            PathBuf::from(&site.dir_esker_build).join(web_parent_paths.join(PathBuf::from(&web_path)));

        // now let's make the full url.
        let url_path = PathBuf::from(web_parent_paths)
            .join(web_path)
            .into_os_string()
            .into_string()
            .unwrap();
        let full_url = site.build_with_baseurl(url_path);
        // end bad code byeeee

        let mut md_file = MdFile {
            raw: raw_str,
            html: String::from(""),
            path,
            out_path,
            web_path: PathBuf::new(),
            full_url,
            frontmatter: fm,
        };

        md_file
            .set_raw_contents()
            .expect("Failed to set raw contents for file");
        return md_file;
    }


    /// writes a file to it's specified output path.
    pub fn write_html(&mut self, site: &mut Site) {
        // parse the markdown for writing it. ---
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_FOOTNOTES);
        let mut parser = Parser::new_ext(&self.raw, options);
        let mut html_output = String::new();

        // -- parser stuff

        let parser = parser.map(|event| -> Event {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Link(link_type, url, title) => {
                        Event::Start(Link::update_link(link_type, url, title, site))
                    }
                    Tag::Image(link_type, url, title) => {
                        Event::Start(Link::update_img_link(link_type, url, title, site))
                    }
                    _ => Event::Start(tag),
                },
                _ => event,
            }
        });

        //
        // -- tera stuff
        //
        html::push_html(&mut html_output, parser);
        let mut ctx = Context::new();
        ctx.insert("title", &self.frontmatter.title);
        ctx.insert("baseurl", &site.baseurl.clone());
        ctx.insert("content", &html_output);
        let template_name = templates::get_name(&site.tera, &self.frontmatter.template);
        let rendered_template = site.tera.render(&template_name, &ctx).unwrap();

        // -- write to file ----
        let prefix = &self.out_path.parent().unwrap();
        fs::create_dir_all(prefix).unwrap();
        let mut file = fs::File::create(&self.out_path).expect("couldn't create file");
        fs::write(&self.out_path, rendered_template).expect("Unable to write file");
    }

    pub fn parse_md_to_html(&mut self) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&self.raw, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        self.html = html_output;
    }

    /// sets the "raw" contents field for the md_file to be the file without the frontmatter.
    fn set_raw_contents(&mut self) -> io::Result<()> {
        let input_file = fs::File::open(self.path.clone())?;
        let mut output: Vec<_> = Vec::new();
        let mut reader = BufReader::new(input_file);
        let mut in_frontmatter = false;

        for (i, line) in reader.lines().enumerate() {
            // Write the line to the output file
            let line = line?;
            if line == "---" && in_frontmatter == false {
                in_frontmatter = true
            } else if line == "---" && in_frontmatter == true {
                in_frontmatter = false;
                continue;
            }

            if in_frontmatter == false {
                output.push(line)
            }
        }

        self.raw = output.join("\n");
        Ok(())
    }
}
