use sysinfo::Disks;

pub fn get_disk_usage() -> Option<(f64, f64)> {
    let mut total_space = 0;
    let mut available_space = 0;
    
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        total_space += disk.total_space();
        available_space += disk.available_space();
    }
    

    if total_space == 0 {
        return None
    }
    let used_bytes = total_space - available_space;
    let divisor = 1024_f64.powi(4);
    let used_tib = (used_bytes as f64) / divisor;
    let total_tib = (total_space as f64) / divisor;

    Some((used_tib, total_tib))
}
