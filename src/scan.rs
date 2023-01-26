use rocket::get;
use rocket::response::content;

#[get("/scan")]
pub fn catch_scan() -> content::RawText<&'static str> {
    content::RawText("begin scanning")
}
