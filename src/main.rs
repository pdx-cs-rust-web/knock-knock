mod joke;

use joke::*;

fn main() {
    let joke = Joke::new(
        1,
        "Boo",
        "You don't have to cry about it!",
        &["kids", "oldie"],
    );
    println!("{:?}", joke);
    println!("{}", String::from(&joke));
}
