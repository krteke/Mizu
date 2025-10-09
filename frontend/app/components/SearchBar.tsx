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
import { SearchHit, SearchResponse } from "@/types/types";

// 一个搜索栏组件，带有展开动画和搜索结果显示功能
export default function SearchBar({ placeholder }: { placeholder: string }) {
  // 控制搜索栏是否展开
  const [isExpanded, setIsExpanded] = useState(false);
  // 控制是否正在查询
  const [query, setQuery] = useState(false);
  // 存储搜索结果
  const [results, setResults] = useState<SearchHit[]>([]);
  // 引用输入框和容器元素
  const inputRef = useRef<HTMLInputElement>(null);
  // 引用加载更多的元素
  const containerRef = useRef<HTMLDivElement>(null);
  // 使用 Next.js 的路由和搜索参数钩子
  const searchParams = useSearchParams();
  // 获取当前路径
  const pathname = usePathname();
  // 获取路由的 replace 方法
  const { replace } = useRouter();
  // 记录当前页码和是否有更多结果
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);

  // 点击外部区域时关闭搜索栏
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsExpanded(false);
        setQuery(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  // 处理搜索结果
  const searchTerm = searchParams.get("query");
  // 每当搜索词变化时，重置结果和页码
  useEffect(() => {
    setResults([]);
    setPage(1);
    setHasMore(true);
  }, [searchTerm]);

  // 当页码或搜索词变化时，触发搜索请求
  useEffect(() => {
    // 获取当前搜索词
    const term = searchParams.get("query") || "";
    // 如果搜索词为空，清空结果并返回
    if (!term) {
      setResults([]);
      setQuery(false);
      return;
    }

    // 定义异步函数来获取搜索结果
    const fetchData = async () => {
      try {
        // 发送请求到搜索API
        const response = await fetch(
          `/api/search?q=${encodeURIComponent(term)}&page=${page}`
        );

        const data: SearchResponse = await response.json();
        // 更新搜索结果
        setResults((prevResults) =>
          page === 1 ? data.results : [...prevResults, ...data.results]
        );

        // 如果没有更多结果，更新状态
        if (data.results.length === 0 || page >= data.total_pages) {
          setHasMore(false);
        }
      } catch (error) {
        // 处理搜索请求失败的情况
        console.error("Search request failed:", error);
        setResults([]);
        setHasMore(false);
      } finally {
      }
    };

    // 在组件挂载时获取初始搜索结果
    fetchData();
  }, [searchParams, page]);

  // 处理搜索框输入变化的函数
  useEffect(() => {
    // 当搜索框输入变化时，更新 URL 参数
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

  // 处理搜索框输入变化的函数, 300ms防抖
  const handleSearch = useDebouncedCallback((term) => {
    const params = new URLSearchParams(searchParams);

    if (term.trim()) {
      params.set("query", term);
    } else {
      params.delete("query");
    }
    replace(`${pathname}?${params.toString()}`);
  }, 300);

  // 使用 Intersection Observer 来实现无限滚动
  const observer = useRef<IntersectionObserver>(null);
  // 监听加载更多的元素进入视图
  const loadMoreRef = useCallback(
    (node: HTMLDivElement) => {
      // 如果没有更多结果，直接返回
      if (observer.current) observer.current.disconnect();

      // 创建新的 Intersection Observer 实例
      observer.current = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting && hasMore) {
          setPage((prevPage) => prevPage + 1);
        }
      });

      // 观察传入的节点, 当它进入视图时触发加载更多
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
                        href={
                          "/" +
                          result.category +
                          "/" +
                          result.id +
                          "/" +
                          result.title
                        }
                        className="w-full h-full items-center flex"
                      >
                        <div className="w-6 h-6 absolute">
                          <Aritcle />
                        </div>
                        <div className="w-full h-full items-center flex overflow-hidden hover:bg-gray-100 dark:hover:bg-zinc-700 rounded-lg p-1 pl-7">
                          // Todo
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
