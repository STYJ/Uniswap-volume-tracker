extern crate dotenv;

use dotenv::dotenv;
use std::env;

pub fn load() {
    dotenv().ok();
    set_teloxide_env_var();
}

pub fn get_value(key: &str) -> String
{
    let val = env::var(key).expect(&format!("key \"{}\" not found in .env", key));
    val
}

fn set_teloxide_env_var() {
    let key = "TELEGRAM_KEY";
    let val = get_value(key);
    env::set_var("TELOXIDE_TOKEN", &val);
    assert_eq!(env::var(key), Ok(val.to_string()));
}