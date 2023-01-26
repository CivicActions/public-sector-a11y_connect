#![feature(proc_macro_hygiene, decl_macro)]

extern crate rocket;

pub mod bigquery;
mod crawl;
mod scan;
mod up;

fn main() {
    rocket::ignite()
        .mount("/up", routes![up::catch_up])
        .mount("/scan", routes![scan::catch_scan])
        .mount("/crawl", routes![crawl::catch_crawl])
        .launch();
}
