use crate::assets::*;
use lazy_static::lazy_static;
use sn0int_registry::db;
use sn0int_registry::errors::*;
use sn0int_registry::models::*;
use std::cmp::Ordering;
use syntect::html::ClassedHTMLGenerator;
use syntect::parsing::{SyntaxSet, SyntaxReference};
use rocket_contrib::templates::Template;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref SYNTAX: &'static SyntaxReference = SYNTAX_SET.find_syntax_by_extension("lua").unwrap();
}

#[get("/")]
pub fn index(connection: db::Connection) -> ApiResult<Template> {
    let asset_rev = ASSET_REV.as_str();

    let mut modules = Module::start_page(&connection)?
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect::<Vec<_>>();

    modules.sort_by(|a, b| {
        match (a.0.as_str(), b.0.as_str()) {
            ("none", "none") => Ordering::Equal,
            ("none", _) => Ordering::Greater,
            (_, "none") => Ordering::Less,
            (a, b) => a.cmp(b),
        }
    });

    Ok(Template::render("index", json!({
        "ASSET_REV": asset_rev,
        "modules": modules,
    })))
}

#[get("/r/<author>/<name>")]
pub fn details(author: String, name: String,connection: db::Connection) -> ApiResult<Template> {
    let module = Module::find(&author, &name, &connection)
        .not_found()
        .public_context("Module does not exist")?;

    let latest = module.latest.as_ref()
        .ok_or(format_err!("No release exists"))
        .not_found()
        .public_context("No release exists")?;
    let release = Release::find(module.id, &latest, &connection)
        .not_found()
        .public_context("Release does not exist")?;

    let mut html_generator = ClassedHTMLGenerator::new(&SYNTAX, &SYNTAX_SET);
    for line in release.code.lines() {
        html_generator.parse_html_for_line(&format!("{}\n", line));
    }
    let code_html = html_generator.finalize();

    let asset_rev = ASSET_REV.as_str();
    Ok(Template::render("details", json!({
        "ASSET_REV": asset_rev,
        "module": module,
        "release": release,
        "code_html": code_html,
    })))
}
