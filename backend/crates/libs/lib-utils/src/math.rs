pub fn get_page_count(all_count: u64, page_size: u64) -> u64 {
    let mut page_count = all_count / page_size;
    if all_count % page_size != 0 {
        page_count += 1;
    }
    page_count
}

pub fn get_page_offset(page: u64, page_size: u64) -> u64 {
    if page == 0 {
        0
    } else {
        (page - 1) * page_size
    }
}
