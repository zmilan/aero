/*
 * Copyright (C) 2021-2022 The Aero Project Developers.
 *
 * This file is part of The Aero Project.
 *
 * Aero is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Aero is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Aero. If not, see <https://www.gnu.org/licenses/>.
 */

//! System Calls are used to call a kernel service from userland.
//!
//! | %rax   | Name                    |
//! |--------|-------------------------|
//! | 0      | read                    |
//! | 1      | write                   |
//! | 2      | open                    |
//! | 3      | close                   |
//! | 4      | shutdown                |
//! | 5      | exit                    |
//! | 6      | fork                    |
//! | 7      | reboot                  |
//! | 8      | mmap                    |
//! | 9      | munmap                  |
//! | 10     | arch_prctl              |
//! | 11     | get_dents               |
//! | 12     | get_cwd                 |
//! | 13     | chdir                   |
//! | 14     | mkdir                   |
//! | 15     | mkdirat                 |
//! | 16     | rmdir                   |
//! | 17     | exec                    |
//! | 18     | log                     |
//! | 19     | uname                   |
//! | 20     | waitpid                 |
//! | 21     | ioctl                   |
//! | 22     | getpid                  |
//! | 23     | socket                  |
//! | 24     | connect                 |
//! | 25     | bind                    |
//! | 26     | listen                  |
//! | 27     | accept                  |
//! | 28     | seek                    |
//! | 29     | gettid                  |
//! | 30     | gettime                 |
//! | 31     | sleep                   |
//! | 32     | access                  |
//! | 33     | pipe                    |
//! | 34     | unlink                  |

use core::mem::MaybeUninit;

use aero_syscall::prelude::*;

pub mod fs;
mod net;
pub mod process;
pub mod time;

use alloc::boxed::Box;
use alloc::vec::Vec;

pub use fs::*;
pub use process::*;
use raw_cpuid::CpuId;
pub use time::*;

use crate::arch::{gdt::GdtEntryType, interrupts};
use crate::utils::{io, StackHelper};

pub struct ExecArgs {
    inner: Vec<Box<[u8]>>,
}

impl ExecArgs {
    pub fn push_into_stack(&self, stack: &mut StackHelper) -> Vec<u64> {
        let mut tops = Vec::with_capacity(self.inner.len());

        for slice in self.inner.iter() {
            unsafe {
                stack.write(0u8);
                stack.write_bytes(slice);
            }

            tops.push(stack.top());
        }

        tops
    }
}

pub fn exec_args_from_slice(args: usize, size: usize) -> ExecArgs {
    // NOTE: Arguments must be moved into kernel space before we utilize them.
    //
    // struct SliceReference {
    //    ptr: *const usize,
    //    len: usize,
    // }
    let data = args as *const [usize; 2];
    let slice = unsafe { core::slice::from_raw_parts(data, size) };

    let mut result = Vec::new();

    for inner in slice {
        let mut boxed = Box::new_uninit_slice(inner[1]);
        let ptr = inner[0] as *const MaybeUninit<u8>;

        unsafe {
            boxed.as_mut_ptr().copy_from(ptr, inner[1]);

            let inner_slice = boxed.assume_init();
            result.push(inner_slice);
        }
    }

    ExecArgs { inner: result }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct SyscallFrame {
    pub rflags: u64,
    pub rip: u64,
    pub rsp: u64,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RegistersFrame {
    pub cr2: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[no_mangle]
extern "C" fn __inner_syscall(sys: &mut SyscallFrame, stack: &mut RegistersFrame) {
    let a = stack.rax as usize;
    let b = stack.rdi as usize;
    let c = stack.rsi as usize;
    let d = stack.rdx as usize;
    let e = stack.r10 as usize;
    let f = stack.r8 as usize;
    let g = stack.r9 as usize;

    match a {
        SYS_EXIT => {}
        _ => unsafe { interrupts::enable_interrupts() },
    }

    let result = match a {
        SYS_EXIT => process::exit(b),
        SYS_SHUTDOWN => process::shutdown(),
        SYS_FORK => process::fork(),
        SYS_MMAP => process::mmap(b, c, d, e, f, g),
        SYS_ARCH_PRCTL => process::arch_prctl(b, c),
        SYS_MUNMAP => process::munmap(b, c),
        SYS_EXEC => process::exec(b, c, d, e, f, g),
        SYS_LOG => process::log(b, c),
        SYS_UNAME => process::uname(b),
        SYS_WAITPID => process::waitpid(b, c, d),
        SYS_GETPID => process::getpid(),
        SYS_GETTID => process::gettid(),

        0x13A => {
            let syscall_name = unsafe {
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(b as *const u8, c))
            };

            panic!("Unimplemented syscall: {}", syscall_name);
        }

        SYS_READ => fs::read(b, c, d),
        SYS_OPEN => fs::open(b, c, d, e),
        SYS_CLOSE => fs::close(b),
        SYS_WRITE => fs::write(b, c, d),
        SYS_GETDENTS => fs::getdents(b, c, d),
        SYS_GETCWD => fs::getcwd(b, c),
        SYS_CHDIR => fs::chdir(b, c),
        SYS_MKDIR => fs::mkdir(b, c),
        SYS_MKDIR_AT => fs::mkdirat(b, c, d),
        SYS_RMDIR => fs::rmdir(b, c),
        SYS_IOCTL => fs::ioctl(b, c, d),
        SYS_SEEK => fs::seek(b, c, d),
        SYS_ACCESS => fs::access(b, c, d, e, f),
        SYS_PIPE => fs::pipe(b, c),
        SYS_UNLINK => fs::unlink(b, c, d, e),

        SYS_SOCKET => net::socket(b, c, d),

        SYS_GETTIME => time::gettime(b, c),
        SYS_SLEEP => time::sleep(b),

        _ => {
            log::error!("invalid syscall: {:#x}", a);
            Err(AeroSyscallError::ENOSYS)
        }
    };

    stack.rax = aero_syscall::syscall_result_as_usize(result) as _;
}

extern "C" {
    fn syscall_handler();
}

pub fn init() {
    // Check if syscall is supported as it is a required CPU feature for aero to run.
    let has_syscall = CpuId::new()
        .get_extended_processor_and_feature_identifiers()
        .map_or(false, |i| i.has_syscall_sysret());

    assert!(has_syscall);

    unsafe {
        /*
         * Enable support for `syscall` and `sysret` instructions if the current
         * CPU supports them and the target pointer width is 64.
         */
        let syscall_base = GdtEntryType::KERNEL_CODE << 3;
        let sysret_base = (GdtEntryType::USER_CODE32_UNUSED << 3) | 3;

        let star_hi = syscall_base as u32 | ((sysret_base as u32) << 16);

        io::wrmsr(io::IA32_STAR, (star_hi as u64) << 32);
        io::wrmsr(io::IA32_LSTAR, syscall_handler as u64);

        // Clear the trap flag and enable interrupts.
        io::wrmsr(io::IA32_FMASK, 0x300);

        // Set the EFER.SCE bit to enable the syscall feature
        let efer = io::rdmsr(io::IA32_EFER);
        io::wrmsr(io::IA32_EFER, efer | 1);
    }
}
