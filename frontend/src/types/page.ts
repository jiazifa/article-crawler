export interface PageRequest {
  page: number;
  page_size: number;
}

export interface PageResponse<T> {
  total_page: number;
  cur_page: number;
  page_size: number;
  data: T[];
}
