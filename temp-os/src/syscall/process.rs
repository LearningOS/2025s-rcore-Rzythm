//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, current_user_token};
use crate::timer::get_time_us;
use crate::mm::translated_byte_buffer;
use crate::mm::{PageTable, VirtAddr, VirtPageNum};
use crate::mm::PTEFlags;
use crate::mm::StepByOne;
use crate::mm::VirtAddr as VA;
use crate::mm::VirtPageNum as VPN;
use crate::mm::PageTableEntry;

// 假设有全局的统计表（可用静态变量或挂在 task 结构体上）
static mut SYSCALL_COUNTS: [usize; 512] = [0; 512];

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    
    // 获取当前时间（微秒）
    let us = get_time_us();
    
    // 使用 translated_byte_buffer 安全地访问用户空间内存
    let buffers = translated_byte_buffer(current_user_token(), ts as *const u8, core::mem::size_of::<TimeVal>());
    
    // 将时间写入用户空间
    let time_val = TimeVal {
        sec: us / 1_000_000,  // 秒
        usec: us % 1_000_000, // 微秒
    };
    
    // 由于 TimeVal 可能跨页，我们需要分别写入每个缓冲区
    let mut offset = 0;
    for buffer in buffers {
        let len = buffer.len();
        unsafe {
            core::ptr::copy_nonoverlapping(
                (&time_val as *const TimeVal as *const u8).add(offset),
                buffer.as_mut_ptr(),
                len
            );
        }
        offset += len;
    }
    
    0  // 返回成功
}

/// 处理 trace 系统调用
pub fn sys_trace(request: usize, id: usize, data: usize) -> isize {
    match request {
        0 => { // 读
            let token = current_user_token();
            let va = id;
            let vpn = VirtAddr::from(va).floor();
            let page_table = PageTable::from_token(token);
            if let Some(pte) = page_table.translate(vpn) {
                let flags = pte.flags();
                // 必须用户可见且可读
                if flags.contains(PTEFlags::U) && flags.contains(PTEFlags::R) {
                    // 只读一个字节
                    let buffers = translated_byte_buffer(token, va as *const u8, 1);
                    if !buffers.is_empty() {
                        return buffers[0][0] as isize;
                    }
                }
            }
            -1
        }
        1 => { // 写
            let token = current_user_token();
            let va = id;
            let vpn = VirtAddr::from(va).floor();
            let page_table = PageTable::from_token(token);
            if let Some(pte) = page_table.translate(vpn) {
                let flags = pte.flags();
                // 必须用户可见且可写
                if flags.contains(PTEFlags::U) && flags.contains(PTEFlags::W) {
                    let mut buffers = translated_byte_buffer(token, va as *const u8, 1);
                    if !buffers.is_empty() {
                        buffers[0][0] = data as u8;
                        return 0;
                    }
                }
            }
            -1
        }
        2 => { // 查询系统调用次数
            let syscall_id = id;
            let count = unsafe {
                SYSCALL_COUNTS[syscall_id] += 1;
                SYSCALL_COUNTS[syscall_id]
            };
            count as isize
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
