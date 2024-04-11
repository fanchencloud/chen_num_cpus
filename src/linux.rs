use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

macro_rules! debug {
    ($($args: expr), * ) => {
        if cfg!(debug_assertions) {
            println!($($args), *);
        }
    };
}

/// 自定义宏
/// 这段代码是 Rust 中的一个宏定义，使用了 `macro_rules!` 宏来定义一个名为 `some` 的宏。
///
/// 这个宏接受一个参数 `$e`，该参数是一个表达式（`$e:expr`）。
/// 宏展开后，会将传入的表达式 `$e` 进行匹配，并根据匹配结果执行相应的操作。<br/>
///
/// 具体来说，这个宏会将传入的表达式 `$e` 匹配到一个 `match` 表达式中，如果表达式 `$e` 的结果是 `Some(v)`，
/// 则返回 `v`；如果表达式 `$e` 的结果是 `None`，则会执行一个调试输出（使用了 `debug!` 宏），
/// 输出字符串 `NONE: $e`，其中 `$e` 是传入表达式的字符串化形式（使用了 `stringify!` 宏），然后返回 `None`。
///
/// 这个宏的作用类似于 `unwrap`，但是当结果为 `None` 时会输出调试信息并返回 `None`，
/// 而不是直接 panic。这样可以在调试模式下及时发现代码中的问题，并提供更多的调试信息。
macro_rules! some {
    ($e: expr) => {
        match $e {
            Some(v) => v,
            None => {
                debug!("NONE: {:?}", stringify!($e));
                return None;
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CgroupVersion {
    V1,
    V2,
}

#[derive(Debug)]
struct Subsys {
    version: CgroupVersion,
    base: String,
}

impl Subsys {
    fn load_cpu<P>(proc_path: P) -> Option<Subsys> where P: AsRef<Path> {
        let file = File::open(&proc_path).unwrap_or_else(|_| panic!("Failed to open /proc/self/cgroup"));
        let buf_reader = BufReader::new(file);

        // 逐行读取文件内容
        for line in buf_reader.lines() {
            println!("- {}", line.unwrap())
        }

        let file = File::open(proc_path).unwrap_or_else(|_| panic!("Failed to open /proc/self/cgroup"));
        let buf_reader = BufReader::new(file);

        buf_reader.lines()
            .filter_map(|result| result.ok())
            .filter_map(Subsys::parse_line)
            .fold(None, |previous, line| {
                // already-found v1 trumps v2 since it explicitly specifies its controllers
                if previous.is_some() && line.version == CgroupVersion::V2 {
                    return previous;
                }

                Some(line)
            })
    }

    fn parse_line(line: String) -> Option<Subsys> {
        // Example format:
        // 11:cpu,cpuacct:/
        let mut fields = line.split(':');

        let sub_systems = some!(fields.nth(1));

        let version = if sub_systems.is_empty() {
            CgroupVersion::V2
        } else {
            CgroupVersion::V1
        };

        if version == CgroupVersion::V1 && !sub_systems.split(',').any(|sub| sub == "cpu") {
            return None;
        }

        fields.next().map(|path| Subsys {
            version: version,
            base: path.to_owned(),
        })
    }
}

pub fn get_num_cpus() -> usize {
    cgroups_num_cpus().unwrap_or_else(|| logical_cpus())
}

fn logical_cpus() -> usize {
    0
}

fn cgroups_num_cpus() -> Option<usize> {
    // 确保只执行一次初始化操作
    static ONCE: Once = Once::new();
    ONCE.call_once(init_cgroups);


    None
}

/// 获取 cgroups 中的 CPU 数
/// 如果获取 0 ， 检查逻辑处理器的数量
static CGROUPS_CPUS: AtomicUsize = AtomicUsize::new(0);

fn init_cgroups() {
    /// 仅在 debug 模式下执行，指定使用 `Ordering::SeqCst` 加载操作的内存顺序，
    /// 这确保了在加载 `CGROUPS_CPUS` 变量的值时使用顺序一致性顺序。
    debug_assert!(CGROUPS_CPUS.load(Ordering::SeqCst) == 0);

    // 检查当前是否是使用 miri 工具进行编译和执行的，如果是，返回true。
    if cfg!(miri) {
        return;
    }

    /// 加载 cgroups
    /// 1. /proc/self/cgroup：
    ///
    ///      `/proc/self/cgroup` 是一个用于查看当前进程所属的 cgroups（控制组）信息的虚拟文件。/proc/self 是一个符号链接，指向当前进程的虚拟文件系统路径。
    ///     这个文件通常用于查看当前进程所属的 cgroups 层次结构，以及各个 cgroup 的名称和配置信息。
    ///     每一行代表一个 cgroup，并列出了当前进程在该 cgroup 中的控制信息，如 cgroup 的名称、层次结构路径等。
    /// 2. /proc/self/mountinfo：
    ///
    ///     `/proc/self/mountinfo` 是一个用于查看当前进程的挂载信息的虚拟文件。
    ///     这个文件提供了有关当前进程所在的挂载点、文件系统类型、挂载选项等详细信息。
    ///     每一行表示一个挂载点的信息，包括挂载的源路径、目标路径、文件系统类型、挂载选项等。
    if let Some(quota) = load_cgroups("/proc/self/cgroup", "/proc/self/mountinfo") {}
}

fn load_cgroups<P1, P2>(cgroup_proc: P1, mountinfo_proc: P2) -> Option<usize>
    where P1: AsRef<std::path::Path>, P2: AsRef<std::path::Path> {
    let subsys = some!(Subsys::load_cpu(cgroup_proc));
    println!("subsys: {:?}", subsys);

    return None;
}

pub fn get_num_physical_cpus() -> usize {
    0
}