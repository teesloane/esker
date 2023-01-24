use colored::*;

use crate::site::Site;

#[derive(Debug)]
pub struct Errors {
    invalid_date_created: Vec<String>,
    invalid_date_updated: Vec<String>,
}

impl Errors {
    pub fn new() -> Errors {
        Errors {
            invalid_date_created: Vec::new(),
            invalid_date_updated: Vec::new(),
        }
    }

    pub fn add_invalid_date_created(&mut self, filepath_str: String) {
        self.invalid_date_created.push(filepath_str);
    }

    pub fn add_invalid_date_updated(&mut self, filepath_str: String) {
        self.invalid_date_updated.push(filepath_str);
    }

    pub fn report_errors(&self, verbose: bool) {
        println!("\n⚠️  Errors and Warnings",);
        if !verbose {
            println!("Pass a {} flag to print additional information", "-v".yellow().on_black());
        }

        if !self.invalid_date_created.is_empty() {
            println!(
                "\n{} files did not have correct {} frontmatter\ndate_created should look like: {}",
                self.invalid_date_created.len(),
                "date_created".to_string().yellow().on_black(),
                "YYYY-MM-DD HH:MM".to_string().green().on_black()
            );

            if verbose {
                println!("\nThe following files have invalid date_created frontmatter\n{:#?}", self.invalid_date_created);
            }
        }

        if !self.invalid_date_updated.is_empty() {
            println!(
                "\n{} files did not have correct {} frontmatter\ndate_updated should look like: {}",
                self.invalid_date_updated.len(),
                "date_updated".to_string().yellow().on_black(),
                "YYYY-MM-DD HH:MM".to_string().green().on_black()
            );

            if verbose {
                println!("\nThe following files have invalid date_updated frontmatter\n{:#?}", self.invalid_date_updated);
            }
        }
    }

    pub fn has_errors(&self) -> bool {
        !(self.invalid_date_updated.is_empty() && self.invalid_date_updated.is_empty())
    }

    pub fn clear(&mut self) {
        self.invalid_date_created.clear();
        self.invalid_date_updated.clear();
    }
}
