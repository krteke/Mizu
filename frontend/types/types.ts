export type category = "article" | "pictures" | "think" | "note" | "talk";

export interface ArticleDigital {
  title: string;
  summary: string;
  tags: string[];
  content: string;
  created_at: string;
  updated_at: string;
}

export interface ArticleCard {
  id: string;
  title: string;
  tags: string[];
  content: string;
}
