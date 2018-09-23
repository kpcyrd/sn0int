use errors::*;
use syscallz::{Context, Syscall};


pub fn init() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::munmap)?;
    ctx.allow_syscall(Syscall::openat)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    ctx.allow_syscall(Syscall::uname)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::ioctl)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::mmap)?;
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

    ctx.load()?;

    Ok(())
}
