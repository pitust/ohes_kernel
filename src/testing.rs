use crate::io;
use crate::print;
use crate::println;
pub fn test_ok() {
    io::Printer.set_color(0, 255, 0);
    println!("[ok]");
    io::Printer.set_color(255, 255, 255);
}
pub fn test_fail() {
    io::Printer.set_color(255, 0, 0);
    println!("[fail]");
    io::Printer.set_color(255, 255, 255);
    panic!("Test failed!");
}

pub fn test_header(test: &str) {
    print!("Test: {}... ", test);
}
