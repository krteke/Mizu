import { codeToHtml, bundledLanguages, bundledThemes } from "shiki";
import type { ShikiTransformer } from "shiki";

export interface HighlightCodeOptions {
  code: string;
  lang?: string;
  lightTheme?: string;
  darkTheme?: string;
  lineNumbers?: boolean;
}

// export interface DualThemeHighlightResult {
//   light: string;
//   dark: string;
// }
export async function highlightCodeDualTheme({
  code,
  lang = "typescript",
  lightTheme = "github-light",
  darkTheme = "github-dark",
  lineNumbers = true,
}: HighlightCodeOptions): Promise<string> {
  try {
    // 检查语言是否支持
    const language = lang in bundledLanguages ? lang : "text";

    const lineNumberTransformer: ShikiTransformer = {
      name: "line-numbers",
      line(node, line) {
        node.properties["data-line"] = line;
        // 添加 class
        const existingClass = node.properties.class || "";
        node.properties.class = existingClass
          ? `${existingClass} line`
          : "line";
      },
    };

    const transformers = lineNumbers ? [lineNumberTransformer] : [];

    // 同时生成明暗两个主题
    const [lightHtml, darkHtml] = await Promise.all([
      codeToHtml(code, {
        lang: language,
        theme: lightTheme,
        transformers,
      }),
      codeToHtml(code, {
        lang: language,
        theme: darkTheme,
        transformers,
      }),
    ]);

    const html = await codeToHtml(code, {
      lang: language,
      themes: {
        light: lightTheme,
        dark: darkTheme,
      },
      transformers: transformers,
    });

    return html;
  } catch (error) {
    console.error("Failed to highlight code:", error);
    // 失败时返回纯文本
    const fallback = `<pre><code>${escapeHtml(code)}</code></pre>`;

    return fallback;
  }
}

export async function highlightCode({
  code,
  lang = "typescript",
  lightTheme = "github-light",
  darkTheme = "github-dark",
  lineNumbers = true,
}: HighlightCodeOptions): Promise<string> {
  const result = await highlightCodeDualTheme({
    code,
    lang,
    lightTheme,
    darkTheme,
    lineNumbers,
  });

  return result;
}

/**
 * 转义 HTML 特殊字符
 */
function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#039;",
  };
  return text.replace(/[&<>"']/g, (m) => map[m]);
}

/**
 * 获取所有支持的语言列表
 */
export function getSupportedLanguages(): string[] {
  return Object.keys(bundledLanguages);
}

/**
 * 获取所有支持的主题列表
 */
export function getSupportedThemes(): string[] {
  return Object.keys(bundledThemes);
}
