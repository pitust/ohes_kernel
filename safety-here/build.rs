
fn main() {
    cc::Build::new()
        .file("src/swap.c")
        .flag("-ffreestanding")
        .flag("-nostdinc")
        .flag("-nostdlib")
        .opt_level(3)
        .compile("safelib");
}