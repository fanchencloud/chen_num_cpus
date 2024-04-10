extern crate chen_num_cpus;

fn main() {
    println!("逻辑处理器数量：{}", chen_num_cpus::get());
    println!("物理处理器数量：{}", chen_num_cpus::get_physical());
}