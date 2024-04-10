#![cfg_attr(test, deny(warnings))]
// #![deny(missing_docs)]
#![allow(non_snake_case)]

#[cfg(not(windows))]
extern crate libc;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::{get_num_cpus, get_num_physical_cpus};

/// 返回当前系统的可用 CPU 数。此函数将获取逻辑内核数。
/// 有时这与物理内核的数量不同（请参阅维基百科上的同步多线程）。这将始终返回至少 1。
/// # 用法
/// ```rust
/// let cpus = chen_num_cpus::get();
/// if cpus > 1 {
///     println!("We are on a multicore system with {} CPUs", cpus);
/// } else {
///      println!("We are on a single core system");
/// }
/// ```
pub fn get() -> usize {
    get_num_cpus()
}

/// 适用于windows 的获取 CPU 数量的实现
#[cfg(windows)]
fn get_num_cpus() -> usize {
    #[repr(C)]
    #[allow(non_snake_case)]
    struct SYSTEM_INFO {
        /// 无符号16位整数，用于描述处理器的体系结构（例如，x86、x64等）
        ///
        /// | 值 | 枚举值 | 描述|
        /// | -- | -- | --|
        /// | PROCESSOR_ARCHITECTURE_AMD64   | 9      | x64 (AMD 或 Intel) |
        /// | PROCESSOR_ARCHITECTURE_ARM     | 5      | ARM |
        /// | PROCESSOR_ARCHITECTURE_ARM64   | 12     | ARM64 |
        /// | PROCESSOR_ARCHITECTURE_IA64    | 6      | 基于 Intel Itanium |
        /// | PROCESSOR_ARCHITECTURE_INTEL   | 0      | x86|
        /// | PROCESSOR_ARCHITECTURE_UNKNOWN | 0xffff | 未知的体系结构。|
        wProcessorArchitecture: u16,
        /// 无符号16位整数，保留字段
        wReserved: u16,
        /// 无符号32位整数，页面保护和承诺的页面大小和粒度。 这是 VirtualAlloc 函数使用的页大小
        dwPageSize: u32,
        /// 指向应用程序和动态链接库可访问的最低内存地址的指针， (DLL) 。
        lpMinimumApplicationAddress: *mut u8,
        /// 指向应用程序和 DLL 可访问的最高内存地址的指针。
        lpMaximumApplicationAddress: *mut u8,
        /// 一个掩码，表示在系统中配置的处理器集。 位 0 是处理器 0;位 31 是处理器 31
        dwActiveProcessorMask: *mut u8,
        /// 无符号32位整数，表示系统中的处理器数量。
        dwNumberOfProcessors: u32,
        /// 无符号32位整数，表示处理器类型。
        /// 为保持兼容性而保留的已过时成员。
        /// 使用 wProcessorArchitecture、 wProcessorLevel 和 wProcessorRevision 成员来确定处理器的类型。
        dwProcessorType: u32,
        /// 可以分配虚拟内存的起始地址的粒度。
        dwAllocationGranularity: u32,
        /// 依赖于体系结构的处理器级别
        wProcessorLevel: u16,
        /// 依赖于体系结构的处理器修订版
        wProcessorRevision: u16,
    }

    /// 使用 `extern` 关键字来声明外部函数,并通过 extern "system" 指定使用的调用约定。
    /// 在Windows平台上，system 调用约定与C语言的标准调用约定（stdcall）相同。
    /// 这个声明告诉编译器，在程序中有一个名为 GetSystemInfo 的外部函数，
    /// 它使用Windows的标准调用约定，并接受一个指向 SYSTEM_INFO 结构体的指针作为参数。
    /// 在Rust中，*mut SYSTEM_INFO 表示一个可变指针，指向 SYSTEM_INFO 结构体。
    extern "system" {
        fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
    }

    unsafe {
        let mut system_info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut system_info);
        return system_info.dwNumberOfProcessors as usize;
    }
}

/// 返回当前系统的物理核心数量 <br />
/// 至少会返回 1
/// ## 注意
/// 获取物理核心数量仅仅在 windows 、 macos 、 linux 三个平台下有效，其他平台返回逻辑核心数量
pub fn get_physical() -> usize {
    get_num_physical_cpus()
}

#[cfg(target_os = "windows")]
fn get_num_physical_cpus() -> usize {
    get_num_physical_cpus_windows().unwrap_or_else(|| get_num_cpus())
}

#[cfg(target_os = "windows")]
fn get_num_physical_cpus_windows() -> Option<usize> {
    use std::ptr;
    use std::mem;

    #[allow(non_upper_case_globals)]
    const RelationProcessorCore: u32 = 0;

    /// `#[repr(C)]` 注解用于指定结构体在 C 语言中的布局方式。
    /// `#[allow(non_camel_case_types)]` 属性的作用是允许使用非驼峰式命名（non_camel_case）作为类型名。
    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
        mask: usize,
        relationship: u32,
        _unused: [u64; 2],
    }

    extern "system" {
        fn GetLogicalProcessorInformation(info: *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION, length: &mut u32) -> u32;
    }

    // 首先，我们需要确定要预留多少空间。

    // 所需的缓冲区大小（以字节为单位）。
    let mut needed_size: u32 = 0;

    unsafe {
        GetLogicalProcessorInformation(ptr::null_mut(), &mut needed_size);
    }

    let struct_size = mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION>() as u32;

    if needed_size == 0 || needed_size < struct_size || needed_size % struct_size != 0 {
        return None;
    }

    let count = needed_size / struct_size;

    // 分配一些内存，我们将在其中存储处理器信息。
    let mut buf: Vec<SYSTEM_LOGICAL_PROCESSOR_INFORMATION> = Vec::with_capacity(count as usize);

    let result: u32;

    unsafe {
        result = GetLogicalProcessorInformation(buf.as_mut_ptr(), &mut needed_size);
    }

    if result == 0 {
        return None;
    }

    let count = needed_size / struct_size;

    unsafe {
        buf.set_len(count as usize);
    }

    let phys_proc_count: usize = buf.iter()
        .filter(|proc_info| proc_info.relationship == RelationProcessorCore)
        .count();

    if phys_proc_count == 0 {
        None
    } else {
        Some(phys_proc_count)
    }
}