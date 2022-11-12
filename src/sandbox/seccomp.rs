use crate::errors::*;
use syscallz::{Context, Syscall, Action};


pub fn init() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::readv)?;
    ctx.allow_syscall(Syscall::writev)?;
    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::munmap)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    ctx.allow_syscall(Syscall::fcntl64)?;
    ctx.allow_syscall(Syscall::uname)?;
    ctx.allow_syscall(Syscall::close)?;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::connect)?;
    #[cfg(target_arch = "x86")]
    ctx.allow_syscall(Syscall::socketcall)?;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
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
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    ctx.allow_syscall(Syscall::mmap2)?;
    ctx.allow_syscall(Syscall::mremap)?;
    ctx.allow_syscall(Syscall::mprotect)?;
    ctx.allow_syscall(Syscall::clone)?;
    ctx.allow_syscall(Syscall::set_robust_list)?;
    ctx.allow_syscall(Syscall::prctl)?;
    ctx.allow_syscall(Syscall::sched_yield)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::madvise)?;
    ctx.allow_syscall(Syscall::rt_sigaction)?;
    ctx.allow_syscall(Syscall::rseq)?;
    #[cfg(target_arch = "x86")]
    ctx.allow_syscall(Syscall::time)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::clock_gettime64)?;
    ctx.allow_syscall(Syscall::nanosleep)?;
    ctx.allow_syscall(Syscall::clock_nanosleep)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::clock_nanosleep_time64)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::brk)?;
    ctx.allow_syscall(Syscall::rt_sigprocmask)?;
    ctx.allow_syscall(Syscall::getpeername)?;
    ctx.allow_syscall(Syscall::gettimeofday)?;
    ctx.allow_syscall(Syscall::membarrier)?;
    ctx.allow_syscall(Syscall::statx)?;
    ctx.allow_syscall(Syscall::fstat)?;
    ctx.allow_syscall(Syscall::lseek)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::_llseek)?;
    ctx.allow_syscall(Syscall::clone3)?;

    ctx.set_action_for_syscall(Action::Errno(1), Syscall::openat)?;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    ctx.set_action_for_syscall(Action::Errno(1), Syscall::open)?;

    ctx.load()?;

    Ok(())
}
