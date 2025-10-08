"use client";
import React, { useCallback, useEffect, useRef, useState } from "react";
import Search from "../assets/search.svg";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useDebouncedCallback } from "use-debounce";
import { AnimatePresence, motion } from "motion/react";
import Link from "next/link";
import Caption from "../assets/caption.svg";
import Aritcle from "../assets/article.svg";
import Paragraph from "../assets/paragraph.svg";

interface SearchResult {
  id: string;
  title: string;
  content: Array<string>;
}

type contentType = "caption" | "paragraph";

export default function SearchBar({ placeholder }: { placeholder: string }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [query, setQuery] = useState(false);
  const [results, setResults] = useState<SearchResult[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const searchParams = useSearchParams();
  const pathname = usePathname();
  const { replace } = useRouter();
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsExpanded(false);
        setQuery(false);
        // setResults([]);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  const searchTerm = searchParams.get("query");
  useEffect(() => {
    setResults([]);
    setPage(1);
    setHasMore(true);
  }, [searchTerm]);

  useEffect(() => {
    const term = searchParams.get("query") || "";
    if (!term) {
      setResults([]);
      setQuery(false);
      return;
    }

    // if (isExpanded) {
    // setQuery(true);
    const fetchData = async () => {
      try {
        const response = await fetch(
          `/api/search?q=${encodeURIComponent(term)}&page=${page}`
        );

        const data = await response.json();
        setResults((prevResults) =>
          page === 1 ? data.results : [...prevResults, ...data.results]
        );

        if (data.results.length === 0 || page >= data.pages) {
          setHasMore(false);
        }
      } catch (error) {
        console.error("Search request failed:", error);
        setResults([]);
        setHasMore(false);
      } finally {
      }
    };

    fetchData();
    // };
  }, [searchParams, page]);

  useEffect(() => {
    const term = searchParams.get("query") || "";
    if (!term) {
      setResults([]);
      setQuery(false);
      return;
    }

    if (isExpanded) {
      setQuery(true);
    }
  }, [isExpanded, searchParams]);

  const handleSearch = useDebouncedCallback((term) => {
    const params = new URLSearchParams(searchParams);

    if (term.trim()) {
      params.set("query", term);
    } else {
      params.delete("query");
    }
    replace(`${pathname}?${params.toString()}`);
  }, 300);

  const observer = useRef<IntersectionObserver>(null);
  const loadMoreRef = useCallback(
    (node: HTMLDivElement) => {
      if (observer.current) observer.current.disconnect();

      observer.current = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting && hasMore) {
          setPage((prevPage) => prevPage + 1);
        }
      });

      if (node) observer.current.observe(node);
    },
    [hasMore]
  );

  return (
    <div ref={containerRef} className="flex items-center justify-end">
      <div
        className={`relative flex items-center transition-all duration-300 ease-in-out ${
          isExpanded ? "w-56 opacity-100" : "w-0 opacity-0"
        }`}
      >
        <input
          ref={inputRef}
          type="text"
          onChange={(e) => {
            if (isExpanded) {
              handleSearch(e.target.value);
            }
          }}
          defaultValue={searchParams.get("query")?.toString()}
          className={`flex-1 border-2 border-[#334155] rounded-2xl px-4 py-2 pr-9 outline-none transition-all duration-300 ease-in-out ${
            isExpanded ? "w-full" : "w-0 cursor-default"
          }`}
          placeholder={placeholder}
        />
      </div>
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-8 h-8 flex absolute items-center justify-center transition-all duration-300 cursor-pointer z-10 pr-1"
        aria-label="search"
      >
        <Search />
      </button>

      <AnimatePresence>
        {isExpanded && query && (
          <motion.div
            initial={{ height: "0" }}
            animate={{ height: "320px" }}
            exit={{ height: "0" }}
            transition={{ duration: 0.4, ease: "easeInOut" }}
            className="absolute w-96 h-80 top-full mt-2 bg-white border-gray-200 dark:bg-[#22262d] dark:border-[#757575] opacity-90 backdrop-blur-md border rounded-lg shadow-2xl z-10 flex justify-center overflow-scroll
            "
          >
            {results.length > 0 ? (
              <div className="w-full flex flex-col absolute justify-center items-center">
                {results.map((result) => {
                  return (
                    <div key={result.id} className="h-13 w-full pl-2 pr-2 pt-1">
                      <Link
                        href={"#"}
                        className="w-full h-full items-center flex"
                      >
                        <div className="w-6 h-6 absolute">
                          <Aritcle />
                        </div>
                        <div className="w-full h-full items-center flex overflow-hidden hover:bg-gray-100 dark:hover:bg-zinc-700 rounded-lg p-1 pl-7">
                          {result.title + result.id}
                        </div>
                      </Link>
                    </div>
                  );
                })}
                <div ref={loadMoreRef} style={{ height: "1px" }} />
              </div>
            ) : (
              <div>no results</div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
