
mod environment;

fn main() {

    environment::load();
    println!("{}", environment::get_value("MEANING_OF_LIFE"));

}
