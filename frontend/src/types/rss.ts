// 定义Link Model

export interface Author {
  name: string;
  email?: string;
  uri?: string;
}

export interface Image {
  url: string;
  title?: string;
  link?: string;
  width?: number;
  height?: number;
  description?: string;
}

export interface Link {
  id: number;
  identifier: string;
  title: string;
  subscription_id: number;
  link: string;
  content?: string;
  description?: string;
  published_at?: Date;
  authors?: Author[];
  images?: Image[];
}

export interface Category {
  id: number;
  title: string;
  description?: string;
  parent_id?: number;
  sort_order?: number;
}

export interface Subscription {
  id: number;
  identifier: string;
  title: string;
  link: string;
  category_id: number;
  description?: string;
  site_link?: string;
  icon?: string;
  logo?: string;
  visual_url?: string;
  language?: string;
  rating?: number;
  last_build_date?: Date;
  accent_color?: string;
  article_count_for_this_week?: number;
  subscribers?: number;
  subscribed_at?: Date;
  custom_title?: string;
  sort_order?: number;
  links?: Link[];
}
