use std::path::Path;
use tera::Tera;
use crate::util;

pub fn load_templates(dir_templates: &Path) -> Tera {
    let template_path = format!("{}/**/*.html", util::path_to_string(dir_templates));
    let mut tera = match Tera::new(&template_path) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            util::exit();
        }
    };
    tera.autoescape_on(vec![]);
    if tera.templates.is_empty() {
        println!("\nError: No templates found in {:?}\n", template_path);
        util::exit();
    }
    tera
}


// get_template
// Returns the name of a template (to later render), provided it's found
// in the tera instance.
pub fn get_name(tera: &Tera, template: &str) -> String {
    let template_with_html = format!("{}.html", &template);
    let default_template_name = "default.html".to_string();
    if tera.get_template_names().any(|x| x == template_with_html) {
        return template_with_html;
    } else {
        return default_template_name
    }
}
