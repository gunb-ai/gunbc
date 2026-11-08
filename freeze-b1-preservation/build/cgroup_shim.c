/* cgroup_shim.c -- path mapping for the hermetic end-to-end control of packet-evidence-freeze2.sh.
 *
 * WHY THIS EXISTS. The frame refuses to run unless the cgroup it is ACTUALLY in carries the booked limits: it
 * reads /proc/self/cgroup (exactly one 0:: record naming the booked unit) and then reads the limit files under
 * /sys/fs/cgroup<that path>. On the target host those are set by the keeper's systemd unit. In a local control
 * they cannot be created at all -- this container's /sys/fs/cgroup is read-only and unshare() is refused -- so a
 * fixture that edited the frame to skip the gate would be testing a different program than the one that is frozen.
 * This shim therefore fakes the ENVIRONMENT and not the frame: the frozen bytes stay byte-identical (the control
 * hashes the frame before and after and refuses if either hash moves), and LD_PRELOAD only redirects two path
 * families into a directory the fixture owns:
 *
 *   /proc/self/cgroup          -> $FAKE_CG_ROOT/proc-cgroup
 *   /sys/fs/cgroup/<rest>      -> $FAKE_CG_ROOT/root/<rest>
 *
 * Nothing else is intercepted, so git, grep, sed, bash and the stubs see the ordinary file system. The mapping is
 * one-way: no write is redirected, so a frame that tried to write a cgroup attribute would hit the real (read-only)
 * path and fail visibly -- which is what we want, because the real frame never writes limits, it only reads them.
 *
 * Build: gcc -O2 -fPIC -shared -o cgroup_shim.so cgroup_shim.c -ldl
 */
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <fcntl.h>
#include <stdarg.h>
#include <sys/stat.h>
#include <unistd.h>

/* Rewrite a path if it belongs to the two mapped families. Returns the original pointer when unmapped. */
#define CG_PREFIX "/sys/fs/cgroup"

static const char *cgmap(const char *p)
{
    _Static_assert(sizeof(CG_PREFIX) - 1 == 14, "CG_PREFIX length must match the literal");
    static __thread char buf[8192];
    const char *root = getenv("FAKE_CG_ROOT");
    if (!root || !p) return p;
    if (strcmp(p, "/proc/self/cgroup") == 0) {
        snprintf(buf, sizeof buf, "%s/proc-cgroup", root);
        return buf;
    }
    /* strlen("/sys/fs/cgroup") is 14. An earlier draft of this file used 12 for both the prefix test and the
     * offset, which mapped "/sys/fs/cgroup/fake/x/memory.max" to ".../rootoup/fake/x/memory.max": the prefix
     * test still passed (12 bytes of 14) and every mapped read then failed with ENOENT, which is what made the
     * frame report "REFUSED_SETUP: /sys/fs/cgroup/.../memory.max unreadable" instead of a wrong VALUE. A length
     * used twice is a second source of truth; strlen_of_literal is asserted below instead. */
    if (strncmp(p, CG_PREFIX, sizeof(CG_PREFIX) - 1) == 0) {
        const char *rest = p + (sizeof(CG_PREFIX) - 1);
        if (!*rest) snprintf(buf, sizeof buf, "%s/root", root);
        else if (*rest == '/') snprintf(buf, sizeof buf, "%s/root%s", root, rest);
        else return p;                      /* /sys/fs/cgroupfoo is not inside the cgroup fs */
        return buf;
    }
    return p;
}

typedef int (*openat_fn)(int, const char *, int, ...);
typedef int (*open_fn)(const char *, int, ...);
typedef int (*stat_fn)(const char *, struct stat *);
typedef int (*access_fn)(const char *, int);
typedef int (*faccessat_fn)(int, const char *, int, int);
typedef int (*unlink_fn)(const char *);
typedef int (*mkdir_fn)(const char *, mode_t);
typedef FILE *(*fopen_fn)(const char *, const char *);

/* mode is only meaningful when O_CREAT is present; pass it through in that case only. */
int openat(int fd, const char *path, int flags, ...)
{
    static openat_fn real = NULL;
    if (!real) real = (openat_fn)dlsym(RTLD_NEXT, "openat");
    mode_t m = 0;
    if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
    const char *p = cgmap(path);
    if (flags & O_CREAT) return real(fd, p, flags, m);
    return real(fd, p, flags);
}

int open(const char *path, int flags, ...)
{
    static open_fn real = NULL;
    if (!real) real = (open_fn)dlsym(RTLD_NEXT, "open");
    mode_t m = 0;
    if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
    const char *p = cgmap(path);
    if (flags & O_CREAT) return real(p, flags, m);
    return real(p, flags);
}

/* glibc's 64-bit entry points: coreutils' cat/open are often open64/openat64 in practice, and missing those is
 * exactly how a mapped file ends up "unreadable". */
int open64(const char *path, int flags, ...)
{
    static open_fn real = NULL;
    if (!real) real = (open_fn)dlsym(RTLD_NEXT, "open64");
    mode_t m = 0;
    if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
    const char *p = cgmap(path);
    if (flags & O_CREAT) return real(p, flags, m);
    return real(p, flags);
}

int openat64(int fd, const char *path, int flags, ...)
{
    static openat_fn real = NULL;
    if (!real) real = (openat_fn)dlsym(RTLD_NEXT, "openat64");
    mode_t m = 0;
    if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
    const char *p = cgmap(path);
    if (flags & O_CREAT) return real(fd, p, flags, m);
    return real(fd, p, flags);
}

FILE *fopen(const char *path, const char *mode)
{
    static fopen_fn real = NULL;
    if (!real) real = (fopen_fn)dlsym(RTLD_NEXT, "fopen");
    return real(cgmap(path), mode);
}

FILE *fopen64(const char *path, const char *mode)
{
    static fopen_fn real = NULL;
    if (!real) real = (fopen_fn)dlsym(RTLD_NEXT, "fopen64");
    return real(cgmap(path), mode);
}

int stat(const char *path, struct stat *st)
{
    static stat_fn real = NULL;
    if (!real) real = (stat_fn)dlsym(RTLD_NEXT, "stat");
    return real(cgmap(path), st);
}

int lstat(const char *path, struct stat *st)
{
    static stat_fn real = NULL;
    if (!real) real = (stat_fn)dlsym(RTLD_NEXT, "lstat");
    return real(cgmap(path), st);
}

int access(const char *path, int mode)
{
    static access_fn real = NULL;
    if (!real) real = (access_fn)dlsym(RTLD_NEXT, "access");
    return real(cgmap(path), mode);
}

int faccessat(int fd, const char *path, int mode, int flags)
{
    static faccessat_fn real = NULL;
    if (!real) real = (faccessat_fn)dlsym(RTLD_NEXT, "faccessat");
    return real(fd, cgmap(path), mode, flags);
}

/* Writes to the cgroup fs must stay visibly refused, so mapping them keeps the failure honest. */
int unlink(const char *path)
{
    static unlink_fn real = NULL;
    if (!real) real = (unlink_fn)dlsym(RTLD_NEXT, "unlink");
    return real(cgmap(path));
}

int mkdir(const char *path, mode_t mode)
{
    static mkdir_fn real = NULL;
    if (!real) real = (mkdir_fn)dlsym(RTLD_NEXT, "mkdir");
    return real(cgmap(path), mode);
}
