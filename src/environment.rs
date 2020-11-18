extern crate dotenv;

use dotenv::dotenv;
use std::env;

pub fn load() {
    dotenv().ok();
}

pub fn get_value(key: &str) -> String
{
    let val = env::var(key).expect(&format!("The key \"{}\" is not found in your .env", key));
    val
}