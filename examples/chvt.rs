
fn main() {
    match chvt::chvt(5) {
        Ok(()) => println!("Switched TTYs successfully"),
        Err(_) => eprintln!("Couldn't switch TTYs, do you have the correct permissions?")
    };
}