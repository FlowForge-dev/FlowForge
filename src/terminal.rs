use console::style;

pub fn heading(text: &str) {
    println!("{}", style(text).bold().cyan());
}

pub fn success(text: &str) {
    println!("{}", style(text).green());
}

pub fn info(text: &str) {
    println!("{}", style(text).blue());
}
