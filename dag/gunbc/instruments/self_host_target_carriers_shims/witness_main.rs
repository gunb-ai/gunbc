use im::{vector as vec, Vector as Vec};
use std::rc::Rc;
use v1_compiled::extdeps_communication_medium::{
    DecodeFidelity as EDecodeFidelity, Medium as EMedium,
};
use v1_compiled::std_algebra::FreeMonoid;
use v1_compiled::v2_compiler_target_carriers as emitted;
use v1_compiled::v2_std_nat::Nat;
use v1_compiled::v2_std_text::Char;
use v1_compiler::v2_compiler_target_carriers as seed;

fn nat_from_u32(n: u32) -> Rc<Nat> {
    (0..n).fold(Rc::new(Nat::Zero), |acc, _| {
        Rc::new(Nat::Succ { prev: acc })
    })
}

fn free_monoid_from_str(s: &str) -> Rc<FreeMonoid<Char>> {
    s.chars()
        .rev()
        .fold(Rc::new(FreeMonoid::Empty), |tail, ch| {
            Rc::new(FreeMonoid::Cons {
                head: nat_from_u32(ch as u32),
                tail: Rc::new(vec![tail]),
            })
        })
}

fn decode_fidelity_eq(e: EDecodeFidelity, s: seed::DecodeFidelity) -> bool {
    matches!(
        (e, s),
        (EDecodeFidelity::Lossless, seed::DecodeFidelity::Lossless)
            | (EDecodeFidelity::Lossy, seed::DecodeFidelity::Lossy)
    )
}

fn medium_text_eq_emitted_seed(
    e: &EMedium<Rc<FreeMonoid<Char>>>,
    s: &seed::Medium<String>,
) -> bool {
    decode_fidelity_eq(e.fidelity, s.fidelity) && free_monoid_to_string(&e.carried) == s.carried
}

fn free_monoid_to_string(fm: &Rc<FreeMonoid<Char>>) -> String {
    match &**fm {
        FreeMonoid::Empty => String::new(),
        FreeMonoid::Cons { head, tail } => {
            let mut out = String::new();
            let code = nat_to_u32(head);
            if code != 0 {
                out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
            }
            if let Some(rest) = tail.iter().next() {
                out.push_str(&free_monoid_to_string(rest));
            }
            out
        }
    }
}

fn nat_to_u32(n: &Rc<Nat>) -> u32 {
    match &**n {
        Nat::Zero => 0,
        Nat::Succ { prev } => nat_to_u32(prev).saturating_add(1),
    }
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let probe = "fn add(x:Int, y:Int) -> Int { x + y }";
    let e_text = free_monoid_from_str(probe);
    let e_lossless = if inject_fault {
        emitted::source_medium(e_text.clone(), EDecodeFidelity::Lossy)
    } else {
        emitted::lossless_source(e_text.clone())
    };
    let s_lossless = seed::lossless_source(probe.to_string());
    let lossless_ok = medium_text_eq_emitted_seed(&e_lossless, &s_lossless);
    println!(
        "lossless_source probe={probe:?} eq={lossless_ok} fidelity emitted={:?} seed={:?}",
        e_lossless.fidelity, s_lossless.fidelity
    );
    all_pass &= lossless_ok;

    let e_lossy = emitted::source_medium(e_text.clone(), EDecodeFidelity::Lossy);
    let s_lossy = seed::source_medium(probe.to_string(), seed::DecodeFidelity::Lossy);
    let lossy_ok = medium_text_eq_emitted_seed(&e_lossy, &s_lossy);
    println!("source_medium Lossy eq={lossy_ok}");
    all_pass &= lossy_ok;

    let e_lossless2 = emitted::source_medium(e_text, EDecodeFidelity::Lossless);
    let s_lossless2 = seed::source_medium(probe.to_string(), seed::DecodeFidelity::Lossless);
    let lossless2_ok = medium_text_eq_emitted_seed(&e_lossless2, &s_lossless2);
    println!("source_medium Lossless eq={lossless2_ok}");
    all_pass &= lossless2_ok;

    if all_pass {
        println!("SELF_HOST_TARGET_CARRIERS_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_TARGET_CARRIERS_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
