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

use aero_syscall::{AeroSyscallError, MMapFlags, MMapProt};

use crate::fs;
use crate::fs::Path;

use crate::mem::paging::VirtAddr;
use crate::userland::scheduler;
use crate::utils::validate_str;

pub fn exit(status: usize) -> ! {
    log::trace!(
        "exiting the process (pid={pid}) with status: {status}",
        pid = scheduler::get_scheduler().current_task().pid().as_usize(),
        status = status
    );

    scheduler::get_scheduler().inner.exit(status as isize);
}

const ARCH_SET_GS: usize = 0x1001;
const ARCH_SET_FS: usize = 0x1002;
const ARCH_GET_FS: usize = 0x1003;
const ARCH_GET_GS: usize = 0x1004;

pub fn arch_prctl(command: usize, address: usize) -> Result<usize, AeroSyscallError> {
    match command {
        ARCH_SET_FS => {
            scheduler::get_scheduler()
                .current_task()
                .arch_task_mut()
                .set_fs_base(VirtAddr::new(address as u64));

            Ok(0x00)
        }

        ARCH_GET_FS => Ok(scheduler::get_scheduler()
            .current_task()
            .arch_task_mut()
            .get_fs_base()
            .as_u64() as usize),

        ARCH_SET_GS => unimplemented!(),
        ARCH_GET_GS => unimplemented!(),

        _ => Err(AeroSyscallError::EINVAL),
    }
}

pub fn uname(buffer: usize) -> Result<usize, AeroSyscallError> {
    fn init_array(fixed: &mut [u8; 65], init: &'static str) {
        let init_bytes = init.as_bytes();
        let len = init.len();

        fixed[..len].copy_from_slice(init_bytes)
    }

    // TODO: Safety checks!
    let struc = unsafe { &mut *(buffer as *mut aero_syscall::Utsname) };

    init_array(&mut struc.name, "Aero");
    init_array(&mut struc.nodename, "unknown");
    init_array(&mut struc.version, env!("CARGO_PKG_VERSION"));
    init_array(
        &mut struc.release,
        concat!(env!("CARGO_PKG_VERSION"), "-aero"),
    );

    #[cfg(target_arch = "x86_64")]
    init_array(&mut struc.machine, "x86_64");

    #[cfg(not(target_arch = "x86_64"))]
    init_array(&mut struc.machine, "unknown");

    Ok(0x00)
}

pub fn fork() -> Result<usize, AeroSyscallError> {
    let scheduler = scheduler::get_scheduler();
    let forked = scheduler.current_task().fork();

    scheduler.register_task(forked.clone());
    Ok(forked.pid().as_usize())
}

pub fn exec(
    path: usize,
    path_size: usize,
    args: usize,
    argc: usize,
    envs: usize,
    envc: usize,
) -> Result<usize, AeroSyscallError> {
    let path = validate_str(path as *const u8, path_size).ok_or(AeroSyscallError::EINVAL)?;
    let path = Path::new(path);

    let executable = fs::lookup_path(path)?;

    if executable.inode().metadata()?.is_directory() {
        return Err(AeroSyscallError::EISDIR);
    }

    // NOTE: Neither args nor envs should be used after this point, the kernel
    // now has owned copies in args and environment variables.
    let argv = if argc > 0 {
        Some(super::exec_args_from_slice(args, argc))
    } else {
        None
    };
    let envv = if envc > 0 {
        Some(super::exec_args_from_slice(envs, envc))
    } else {
        None
    };

    scheduler::get_scheduler()
        .current_task()
        .exec(executable, argv, envv)
        .expect("task: failed to exec task");

    unreachable!()
}

pub fn log(msg_start: usize, msg_size: usize) -> Result<usize, AeroSyscallError> {
    let message = validate_str(msg_start as *const u8, msg_size).ok_or(AeroSyscallError::EINVAL)?;
    log::debug!("{}", message);

    Ok(0x00)
}

pub fn waitpid(pid: usize, status: usize, _flags: usize) -> Result<usize, AeroSyscallError> {
    let current_task = scheduler::get_scheduler().current_task();
    let status = unsafe { &mut *(status as *mut u32) };

    Ok(current_task.waitpid(pid, status)?)
}

pub fn mmap(
    address: usize,
    size: usize,
    protocol: usize,
    flags: usize,
    _fd: usize,
    _offset: usize,
) -> Result<usize, AeroSyscallError> {
    let address = VirtAddr::new(address as u64);

    let protocol = MMapProt::from_bits(protocol).ok_or(AeroSyscallError::EINVAL)?;
    let flags = MMapFlags::from_bits(flags).ok_or(AeroSyscallError::EINVAL)?;

    if !flags.contains(MMapFlags::MAP_ANONYOMUS) {
        unimplemented!()
    }

    // HACK: This is currently a hack since mlibc tries to do somethin
    // fancy. Oh well andy plz fix this in the future.
    let size = size + 4096;

    if let Some(alloc) = scheduler::get_scheduler()
        .current_task()
        .vm()
        .mmap(address, size, protocol, flags)
    {
        Ok(alloc.as_u64() as usize)
    } else {
        Err(AeroSyscallError::EFAULT)
    }
}

pub fn munmap(address: usize, size: usize) -> Result<usize, AeroSyscallError> {
    let address = VirtAddr::new(address as u64);

    if scheduler::get_scheduler()
        .current_task()
        .vm
        .munmap(address, size)
    {
        Ok(0x00)
    } else {
        Err(AeroSyscallError::EFAULT)
    }
}

pub fn getpid() -> Result<usize, AeroSyscallError> {
    Ok(scheduler::get_scheduler().current_task().pid().as_usize())
}

pub fn gettid() -> Result<usize, AeroSyscallError> {
    Ok(scheduler::get_scheduler().current_task().tid().as_usize())
}

pub fn shutdown() -> ! {
    fs::cache::dcache().log();

    fs::cache::clear_inode_cache();
    fs::cache::clear_dir_cache();

    // TODO
    loop {}
}
