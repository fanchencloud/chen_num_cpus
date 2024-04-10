pub fn get_num_cpus() -> usize {
    0
}

fn cgroups_num_cpus() -> Option<usize> {
    None
}

pub fn get_num_physical_cpus() -> usize {
    0
}
