#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod crawl;
mod scan;
mod up;

fn main() {
    rocket::ignite()
        .mount("/up", routes![up::up])
        .mount("/scan", routes![scan::scan])
        .mount("/crawl", routes![crawl::crawl])
        .launch();
}
