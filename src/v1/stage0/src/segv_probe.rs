//! BRANCH-LOCAL DIAGNOSTIC — delete before merge.
//!
//! Answers one question the harness cannot: WHICH stack overflowed. std
//! registers guard pages only for the main thread and threads it spawns, so a
//! fault on a `stacker`-allocated heap segment produces a bare SIGSEGV with no
//! "has overflowed its stack" message — which is exactly what this tree does
//! (six arms, zero messages). Distinguishing that from an ordinary thread-stack
//! overflow, and from a wild pointer, requires the faulting address.
//!
//! The handler uses only async-signal-safe calls: write, open, read, close,
//! _exit. No allocation, no formatting machinery, no locks.

use std::os::raw::{c_int, c_void};

unsafe fn emit(bytes: &[u8]) {
    libc::write(2, bytes.as_ptr() as *const c_void, bytes.len());
}

unsafe fn emit_hex(mut n: usize) {
    let digits = b"0123456789abcdef";
    let mut buf = [0u8; 16];
    let mut i = 16;
    if n == 0 {
        emit(b"0");
        return;
    }
    while n > 0 && i > 0 {
        i -= 1;
        buf[i] = digits[n & 0xf];
        n >>= 4;
    }
    emit(&buf[i..]);
}

/// Walks the saved frame-pointer chain from the FAULTING context.
///
/// Requires `-C force-frame-pointers=yes`; release builds omit rbp otherwise.
/// Pointer-chasing in already-mapped memory only — no allocation, no locks, so
/// it is async-signal-safe. It runs LAST so that a corrupted chain costs only
/// the chain: the fault address and the maps are already emitted.
///
/// The chain gives two things without any symbolization: its LENGTH is the
/// recursion depth, and a repeating return address identifies the recursion
/// itself (one repeated value = direct, a short cycle = mutual).
unsafe fn walk_frames(ctx: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    let uc = ctx as *const libc::ucontext_t;
    let rip = (*uc).uc_mcontext.gregs[libc::REG_RIP as usize] as usize;
    let rbp = (*uc).uc_mcontext.gregs[libc::REG_RBP as usize] as usize;
    emit(b"FAULT_RIP=0x");
    emit_hex(rip);
    emit(b"\nFRAME_CHAIN_BEGIN\n");
    let mut fp = rbp;
    let mut prev = 0usize;
    let mut n = 0usize;
    // Stacks grow down, so rbp must strictly INCREASE walking outward. A chain
    // that stalls, reverses, or misaligns is corrupt and must terminate rather
    // than fault inside the fault handler.
    while n < 400_000 && fp != 0 && fp > prev && (fp & 7) == 0 {
        let ret = *((fp + 8) as *const usize);
        let next = *(fp as *const usize);
        if n < 48 || n % 10_000 == 0 {
            emit(b"  f=0x");
            emit_hex(ret);
            emit(b"\n");
        }
        prev = fp;
        fp = next;
        n += 1;
    }
    emit(b"FRAME_CHAIN_DEPTH_HEX=0x");
    emit_hex(n);
    emit(b"\n");
}

unsafe extern "C" fn on_segv(_sig: c_int, info: *mut libc::siginfo_t, ctx: *mut c_void) {
    let addr = if info.is_null() {
        0usize
    } else {
        (*info).si_addr() as usize
    };
    emit(b"\nSEGV_FAULT_ADDR=0x");
    emit_hex(addr);
    // A local's address approximates the stack in use at fault time, which is
    // what makes the fault address interpretable without symbol machinery.
    let probe_local: u8 = 0;
    emit(b"\nSEGV_SP_NEAR=0x");
    emit_hex(&probe_local as *const u8 as usize);
    emit(b"\n=== MAPS BEGIN ===\n");
    let path = b"/proc/self/maps\0";
    let fd = libc::open(path.as_ptr() as *const _, libc::O_RDONLY);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        loop {
            let r = libc::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len());
            if r <= 0 {
                break;
            }
            libc::write(2, buf.as_ptr() as *const c_void, r as usize);
        }
        libc::close(fd);
    }
    emit(b"=== MAPS END ===\n");
    walk_frames(ctx);
    libc::_exit(139);
}

/// Replaces std's SIGSEGV handler. Safe to call more than once.
pub fn install() {
    unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        let handler: unsafe extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void) = on_segv;
        sa.sa_sigaction = handler as *const () as usize;
        sa.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(libc::SIGSEGV, &sa, std::ptr::null_mut());
    }
}
