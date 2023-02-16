use rocket::get;

#[get("/ready")]
pub fn catch_ready() -> Result<(), ()> {
    Ok(())
}

#[get("/health")]
pub fn catch_health() -> Result<(), ()> {
    Ok(())
}
