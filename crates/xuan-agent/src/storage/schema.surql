-- XuanAgent SurrealDB Schema
--
-- 用于文献管理和向量检索的数据库结构
--
-- 使用方式：
-- surreal sql --endpoint ws://localhost:8000 --ns xuan --db papers -c schema.surql

-- ============================================================
-- 表定义
-- ============================================================

-- 文献表：存储文献元数据
DEFINE TABLE paper SCHEMALESS;

-- 文献分块表：存储文献内容分块及向量嵌入
DEFINE TABLE chunk SCHEMALESS;

-- ============================================================
-- 索引定义
-- ============================================================

-- paper 表索引
DEFINE INDEX paper_title_idx ON paper FIELDS title;
DEFINE INDEX paper_authors_idx ON paper FIELDS authors;
DEFINE INDEX paper_year_idx ON paper FIELDS year;
DEFINE INDEX paper_journal_idx ON paper FIELDS journal;
DEFINE INDEX paper_doi_idx ON paper FIELDS doi;
DEFINE INDEX paper_tags_idx ON paper FIELDS tags;
DEFINE INDEX paper_created_at_idx ON paper FIELDS created_at;

-- chunk 表索引
DEFINE INDEX chunk_paper_id_idx ON chunk FIELDS paper_id;
DEFINE INDEX chunk_paper_id_chunk_idx ON chunk FIELDS paper_id, chunk_index;
DEFINE INDEX chunk_embedding_idx ON chunk FIELDS embedding;

-- ============================================================
-- 权限定义（可选，生产环境使用）
-- ============================================================

-- 限制 paper 表的记录只能通过特定方式访问
-- DEFINE ACCESS paper_access ON DATABASE
--     FOR SELECT, UPDATE, DELETE
--     WHERE $auth.id;

-- ============================================================
-- 示例数据（可选，用于测试）
-- ============================================================

-- 创建示例文献
-- INSERT INTO paper {
--     id: "example-paper-001",
--     title: "Example Research Paper",
--     abstract_text: "This is an example abstract for testing purposes.",
--     authors: ["Author One", "Author Two"],
--     year: 2024,
--     journal: "Journal of Examples",
--     doi: "10.1234/example.001",
--     tags: ["example", "test"],
--     created_at: time::now()
-- };

-- ============================================================
-- 有用的查询示例
-- ============================================================

-- 查询所有文献（按创建时间倒序）
-- SELECT * FROM paper ORDER BY created_at DESC;

-- 按关键词搜索标题
-- SELECT * FROM paper WHERE title @ 'machine learning';

-- 按标签搜索
-- SELECT * FROM paper WHERE 'ai' IN tags;

-- 获取某篇文献的所有分块（按索引排序）
-- SELECT * FROM chunk WHERE paper_id = 'paper-id' ORDER BY chunk_index;

-- 统计文献数量
-- SELECT count() FROM paper;

-- 统计分块数量
-- SELECT count() FROM chunk;

-- ============================================================
-- 视图定义（可选）
-- ============================================================

-- 创建带分块数量的文献视图
-- DEFINE VIEW paper_with_chunk_count AS
-- SELECT *,
--     (SELECT count() FROM chunk WHERE chunk.paper_id = paper.id) AS chunk_count
-- FROM paper;
