// The NONZERO half of the native-CLI working-root control: a type error the compiler must reject, so
// cargo returns nonzero and the emitted program returns the failure through its declared output
// fields rather than panicking.
fn main() {
    let measured: u32 = "not a count";
    println!("{measured}");
}
