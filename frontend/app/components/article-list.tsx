import { ArticleCard, category } from "@/types/types";

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
        <div className="flex flex-col">
          {cards.map((a) => {
            return <div key={a.title}></div>;
          })}
        </div>
      );
    }
    case "note": {
      return <div></div>;
    }
    case "pictures": {
      return <div></div>;
    }
    case "talk": {
      return <div></div>;
    }
    case "think": {
      return <div></div>;
    }
  }
}
