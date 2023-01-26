#![feature(proc_macro_hygiene, decl_macro)]

extern crate rocket;
use rocket::routes;

pub mod bigquery;
mod crawl;
mod scan;
mod up;

fn main() {
    rocket::build()
        .mount("/up", routes![up::catch_up])
        .mount("/scan", routes![scan::catch_scan])
        .mount("/crawl", routes![crawl::catch_crawl])
        .launch();
}
