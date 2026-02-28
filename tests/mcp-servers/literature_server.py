#!/usr/bin/env python3
"""
XuanAgent 文献管理 MCP 服务器

这是一个演示用的文献管理 MCP 服务器，模拟 Zotero 等文献管理工具的功能。
生产环境中应该连接到实际的文献管理数据库（如 Zotero、Mendeley 等）。
"""

import sys
import json
from typing import Dict, List, Any


# 模拟文献数据库
MOCK_LITERATURE_DB = [
    {
        "id": "lit001",
        "title": "Attention Is All You Need",
        "authors": ["Vaswani, A.", "Shazeer, N.", "Parmar, N."],
        "year": 2017,
        "venue": "NeurIPS",
        "abstract": "The dominant sequence transduction models...",
        "tags": ["transformer", "attention", "nlp"],
        "citation_count": 50000
    },
    {
        "id": "lit002",
        "title": "BERT: Pre-training of Deep Bidirectional Transformers",
        "authors": ["Devlin, J.", "Chang, M.W.", "Lee, K."],
        "year": 2018,
        "venue": "NAACL",
        "abstract": "We introduce a new language representation model...",
        "tags": ["bert", "pretraining", "nlp"],
        "citation_count": 80000
    },
    {
        "id": "lit003",
        "title": "Language Models are Few-Shot Learners",
        "authors": ["Brown, T.", "Mann, B.", "Ryder, N."],
        "year": 2020,
        "venue": "NeurIPS",
        "abstract": "Recent work has demonstrated substantial gains...",
        "tags": ["gpt", "few-shot", "language-model"],
        "citation_count": 30000
    }
]


def create_response(result: Any = None, error: Any = None, id: Any = None) -> str:
    """创建 JSON-RPC 响应"""
    response = {
        "jsonrpc": "2.0",
        "id": id
    }

    if error is not None:
        response["error"] = error
    else:
        response["result"] = result

    return json.dumps(response, ensure_ascii=False) + "\n"


def handle_initialize(params: Dict, id: Any) -> str:
    """处理 initialize 请求"""
    return create_response({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "xuan-literature-mcp",
            "version": "0.1.0"
        }
    }, id=id)


def handle_tools_list(params: Dict, id: Any) -> str:
    """处理 tools/list 请求"""
    return create_response({
        "tools": [
            {
                "name": "search_literature",
                "description": "搜索文献库中的文献",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "搜索关键词（标题、作者、标签等）"
                        },
                        "year": {
                            "type": "integer",
                            "description": "按年份筛选"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "返回结果数量限制",
                            "default": 10
                        }
                    }
                }
            },
            {
                "name": "get_literature",
                "description": "获取指定文献的详细信息",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "文献 ID"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "add_literature",
                "description": "添加新文献到库中",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "文献标题"
                        },
                        "authors": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "作者列表"
                        },
                        "year": {
                            "type": "integer",
                            "description": "发表年份"
                        },
                        "venue": {
                            "type": "string",
                            "description": "发表会议/期刊"
                        },
                        "abstract": {
                            "type": "string",
                            "description": "摘要"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "标签"
                        }
                    },
                    "required": ["title", "authors"]
                }
            },
            {
                "name": "list_collections",
                "description": "列出所有文献集合/分类",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "export_citation",
                "description": "导出指定文献的引用信息",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "文献 ID"
                        },
                        "format": {
                            "type": "string",
                            "description": "引用格式 (bibtex, apa, mla)",
                            "default": "bibtex"
                        }
                    },
                    "required": ["id"]
                }
            }
        ]
    }, id=id)


def handle_tools_call(params: Dict, id: Any) -> str:
    """处理 tools/call 请求"""
    name = params.get("name")
    arguments = params.get("arguments", {})

    try:
        if name == "search_literature":
            query = arguments.get("query", "").lower()
            year = arguments.get("year")
            limit = arguments.get("limit", 10)

            results = []
            for lit in MOCK_LITERATURE_DB:
                # 应用筛选条件
                if year and lit.get("year") != year:
                    continue

                # 搜索匹配
                text_match = (
                    query in lit.get("title", "").lower() or
                    query in lit.get("venue", "").lower() or
                    any(query in tag.lower() for tag in lit.get("tags", [])) or
                    any(query in author.lower() for author in lit.get("authors", []))
                )

                if text_match or not query:
                    results.append(lit)
                    if len(results) >= limit:
                        break

            return create_response({
                "content": [{
                    "type": "text",
                    "text": f"找到 {len(results)} 篇文献:\n\n" + \
                           "\n".join(f"- [{lit['id']}] {lit['title']} ({lit['year']}) - {', '.join(lit['authors'][:2])}"
                                   for lit in results)
                }]
            }, id=id)

        elif name == "get_literature":
            lit_id = arguments.get("id")
            for lit in MOCK_LITERATURE_DB:
                if lit["id"] == lit_id:
                    return create_response({
                        "content": [{
                            "type": "text",
                            "text": f"Title: {lit['title']}\n" \
                                   f"Authors: {', '.join(lit['authors'])}\n" \
                                   f"Year: {lit['year']}\n" \
                                   f"Venue: {lit['venue']}\n" \
                                   f"Citations: {lit.get('citation_count', 'N/A')}\n" \
                                   f"Tags: {', '.join(lit.get('tags', []))}\n" \
                                   f"Abstract: {lit.get('abstract', 'N/A')}"
                        }]
                    }, id=id)

            return create_response({
                "error": f"文献 '{lit_id}' 未找到"
            }, id=id)

        elif name == "add_literature":
            # 模拟添加文献
            new_id = f"lit{len(MOCK_LITERATURE_DB) + 1:03d}"
            new_lit = {
                "id": new_id,
                "title": arguments.get("title", ""),
                "authors": arguments.get("authors", []),
                "year": arguments.get("year"),
                "venue": arguments.get("venue"),
                "abstract": arguments.get("abstract"),
                "tags": arguments.get("tags", []),
                "citation_count": 0
            }

            return create_response({
                "content": [{
                    "type": "text",
                    "text": f"✓ 文献已添加 (ID: {new_id})\n  Title: {new_lit['title']}"
                }]
            }, id=id)

        elif name == "list_collections":
            collections = [
                {"id": "col001", "name": "深度学习", "count": 15},
                {"id": "col002", "name": "自然语言处理", "count": 23},
                {"id": "col003", "name": "计算机视觉", "count": 18}
            ]

            return create_response({
                "content": [{
                    "type": "text",
                    "text": "文献集合:\n" + "\n".join(
                        f"- [{c['id']}] {c['name']} ({c['count']} 篇)"
                        for c in collections
                    )
                }]
            }, id=id)

        elif name == "export_citation":
            lit_id = arguments.get("id")
            format_type = arguments.get("format", "bibtex")

            for lit in MOCK_LITERATURE_DB:
                if lit["id"] == lit_id:
                    if format_type == "bibtex":
                        citation = "@inproceedings{" + lit['id'].replace('lit', '') + lit['year'] + ",\n" \
                                  f"  title={{{lit['title']}}},\n" \
                                  f"  author={{{' and '.join(lit['authors'])}}},\n" \
                                  f"  booktitle={{{lit['venue']}}},\n" \
                                  f"  year={{{lit['year']}}}\n" \
                                  "}"
                    elif format_type == "apa":
                        citation = f"{', '.join(lit['authors'][:-1])} & {lit['authors'][-1]} " \
                                  f"({lit['year']}). {lit['title']}. {lit['venue']}."
                    else:
                        citation = f"{', '.join(lit['authors'])}. \"{lit['title']}.\" {lit['venue']}, {lit['year']}."

                    return create_response({
                        "content": [{
                            "type": "text",
                            "text": f"引用格式 ({format_type}):\n\n{citation}"
                        }]
                    }, id=id)

            return create_response({
                "error": f"文献 '{lit_id}' 未找到"
            }, id=id)

        else:
            return create_response({
                "error": f"未知工具: {name}"
            }, id=id)

    except Exception as e:
        return create_response({
            "error": str(e)
        }, id=id)


def main():
    """MCP 服务器主循环"""
    for line in sys.stdin:
        try:
            request = json.loads(line.strip())
            method = request.get("method")
            params = request.get("params", {})
            req_id = request.get("id")

            if method == "initialize":
                response = handle_initialize(params, req_id)
            elif method == "tools/list":
                response = handle_tools_list(params, req_id)
            elif method == "tools/call":
                response = handle_tools_call(params, req_id)
            else:
                response = create_response({
                    "error": f"未知方法: {method}"
                }, id=req_id)

            sys.stdout.write(response)
            sys.stdout.flush()

        except json.JSONDecodeError:
            # 忽略非 JSON 行（如空行）
            continue
        except Exception as e:
            sys.stderr.write(f"Error: {e}\n")
            sys.stderr.flush()


if __name__ == "__main__":
    main()
