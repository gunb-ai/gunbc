extern "C" {
    fn revision_job_link_smoke(x: i32) -> i32;
}

fn main() {
    println!("link-smoke {}", unsafe { revision_job_link_smoke(7) });
}
