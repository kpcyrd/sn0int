#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::response::content;

#[get("/")]
fn index() -> content::Html<&'static str> {
    content::Html(include_str!("../templates/index.html"))
}

#[get("/assets/style.css")]
fn style() -> content::Css<&'static str> {
    content::Css(include_str!("../assets/style.css"))
}

fn main() {
    rocket::ignite().mount("/", routes![index, style]).launch();
}
