import { ArticleCard, category } from "@/types/types";
import ArticleCardComponent from "./ArticleCardComponent";

export default function ArticleList({
  category,
  cards,
}: {
  category: category;
  cards: ArticleCard[];
}) {
  switch (category) {
    case "article": {
      return (
        <div className="flex flex-col gap-y-4 flex-1 pt-20 pl-5 pr-5">
          {cards.map((a) => {
            return (
              <ArticleCardComponent key={a.id} article={a} basePath="article" />
            );
          })}
        </div>
      );
    }
    case "note": {
      return (
        <div className="flex flex-col">
          {cards.map((n) => {
            return <div key={n.id}></div>;
          })}
        </div>
      );
    }
    case "pictures": {
      return (
        <div className="flex flex-col">
          {cards.map((p) => {
            return <div key={p.id}></div>;
          })}
        </div>
      );
    }
    case "talk": {
      return (
        <div className="flex flex-col">
          {cards.map((t) => {
            return <div key={t.id}></div>;
          })}
        </div>
      );
    }
    case "think": {
      return (
        <div className="flex flex-col">
          {cards.map((t) => {
            return <div key={t.id}></div>;
          })}
        </div>
      );
    }
  }
}
