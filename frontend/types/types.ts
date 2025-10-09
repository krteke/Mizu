// 文章类别
export type category = "article" | "pictures" | "think" | "note" | "talk";

// 点击ScrollToEdge时的滑动方向
export type scrollDir = "top" | "bottom";

// 文章的详细信息
export interface ArticleDigital {
  title: string;
  summary: string;
  tags: string[];
  content: string;
  created_at: string;
  updated_at: string;
}

// 文章卡片信息
export interface ArticleCard {
  id: string;
  title: string;
  tags: string[];
  content: string;
}

export interface SearchResponse {
  total_hints: number;
  total_pages: number;
  current_page: number;
  results: SearchHit[];
}

export interface SearchHit {
  id: string;
  title: string;
  category: string;
  summary: string;
  content: string;
}
