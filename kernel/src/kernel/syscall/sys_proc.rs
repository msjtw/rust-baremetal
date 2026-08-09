use alloc::{string::String, vec::Vec};

use crate::{
    process::Process,
    virtmemory::{copy_in, copy_in_str},
};

pub fn sys_fork(proc: &mut Process) {
    let mut kernel = crate::KERNEL.get().unwrap().lock();
    proc.kfork(&mut kernel).unwrap();
}

pub fn sys_exec(proc: &mut Process) {
    let path_addr = proc.trapframe.a0;
    let mut argv_addr = proc.trapframe.a1;

    let path = copy_in_str(&mut proc.pagetable, path_addr).unwrap();

    let mut argv = Vec::<String>::new();
    loop {
        let arg_addr = copy_in::<usize>(&mut proc.pagetable, argv_addr).unwrap();
        if arg_addr == 0 {
            break;
        }

        let arg_str = copy_in_str(&mut proc.pagetable, argv_addr).unwrap();
        argv.push(arg_str);

        argv_addr += size_of::<usize>();
    }

    let argv_str = argv.iter().map(|s| s.as_ref()).collect();
    proc.kexec(path, argv_str);
}

pub fn sys_wait(proc: &mut Process) {
    let status_addr = proc.trapframe.a0;
    let ret = proc.kwait(status_addr);
    proc.trapframe.a0 = ret as usize;
}

pub fn sys_exit(proc: &mut Process) {
    let xstatus_addr = proc.trapframe.a0;
    let xstatus: u32 = copy_in(&mut proc.pagetable, xstatus_addr).unwrap();
    
    proc.kexit(xstatus);
}

pub fn sys_getpid() {}

pub fn sys_sbrk() {}

pub fn sys_pause() {}

pub fn sys_kill() {}

pub fn sys_uptime() {}
