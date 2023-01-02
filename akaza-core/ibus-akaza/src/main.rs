extern "C" {
    fn main2();
}

fn main() {
    unsafe {
        main2();
    }
}
