use crate::errors::*;
use syscallz::{Context, Syscall};


pub fn init() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::readv)?;
    ctx.allow_syscall(Syscall::writev)?;
    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::munmap)?;
    //ctx.allow_syscall(Syscall::openat)?;
    //#[cfg(not(target_arch = "aarch64"))]
    //ctx.allow_syscall(Syscall::open)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::fcntl64)?;
    ctx.allow_syscall(Syscall::uname)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::connect)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::epoll_pwait)?;
    ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::ioctl)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::send)?;
    ctx.allow_syscall(Syscall::sendto)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::recv)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    #[cfg(not(target_arch = "arm"))]
    ctx.allow_syscall(Syscall::mmap)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::mmap2)?;
    ctx.allow_syscall(Syscall::mprotect)?;
    ctx.allow_syscall(Syscall::clone)?;
    ctx.allow_syscall(Syscall::set_robust_list)?;
    ctx.allow_syscall(Syscall::prctl)?;
    ctx.allow_syscall(Syscall::sched_yield)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::madvise)?;
    ctx.allow_syscall(Syscall::nanosleep)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::brk)?;
    ctx.allow_syscall(Syscall::rt_sigprocmask)?;
    ctx.allow_syscall(Syscall::getpeername)?;

    ctx.load()?;

    Ok(())
}
