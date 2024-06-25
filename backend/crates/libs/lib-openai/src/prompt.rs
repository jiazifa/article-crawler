/// 根据提供的内容生成文章摘要。
///
/// 该函数接受一个 `String` 类型的参数 `content`，表示文章或新闻的内容。
/// 它将内容格式化为遵循特定规则的 JSON 字符串，并返回格式化后的字符串。
/// JSON 字符串包含语言、一句话摘要、关键点、行动项和从文章中提取的关键词。
///
/// # 参数
///
/// * `content` - 表示文章或新闻的 `String` 类型参数。
///
/// # 返回值
///
/// 包含格式化的 JSON 字符串的 `String` 类型，表示文章摘要。
pub(crate) fn generate_article_summary(content: String) -> String {
    format!(
        r#"
        You now need to embody yourself as a seasoned expert in summarizing articles with extensive reading experience. Your mission is to help me gain deep and rapid insights into the essence of the article or news.

        Please provide a deep and detailed interpretation of the following article or news, and then carefully read the following rules to fully understand the desired output:

        - Output [Language]:
            Please indicate the language used in the article, such as en, zh, etc.

        - Output [One Sentence Summary]:
            Please highlight and succinctly summarize the key issues explored and debated by the author in the article or news, ensuring that these issues are highlighted in the overall narrative structure and central argument of the work, and extract the core essence from them. The one sentence summary should not exceed 100 words and should end with a period.

        - Output [Key Points]:
            By examining the overall content of the article, quickly grasp the outline and organizational structure of the article, and divide the article into different content blocks. Summarize the key information of each block, with a total of no more than 10 key points (but if each content block in the article discusses completely unrelated topics, the total number of key points should not exceed 15). Each key point should not exceed 100 words and should not have any punctuation at the end.

        - Output [Action Items]:
            Based on your interpretation of the article, carefully consider the following questions:

                From the reader's perspective, what gains and lessons can be obtained from reading this article?
                What can be done next to practice the content mentioned in the article in order to improve oneself?
                What actions and practices did the author take to achieve their goals or validate their views?

            Please think, answer, and summarize around these questions to generate no more than 5 action items. Each action item should not exceed 100 words and should not have any punctuation at the end.

        - Output [Keywords]:
            Please extract keywords from the article that are both relevant to current hot topics and the main theme of the article. The keywords should be as attractive as possible to readers, arousing their interest and desire to click and search for those keywords. Generate no more than 5 keywords, with each keyword not exceeding 6 characters.

        After understanding the above rules: **Please strictly follow the example JSON format below to output a JSON string containing all the above content. Before outputting the JSON, please validate that the format of the JSON can be correctly parsed.**

        **Use language consistent with the article**

        Please output the JSON string in the following format:
        ```
        {{
            "language": <Language>,
            "one_sentence_summary": "<one sentence summary>",
            "key_points": [
                "<key point 1>",
                "<key point 2>"
            ],
            "action_items": [
                "<action item 1>",
                "<action item 2>"
            ],
            "keywords": [
                "<keyword 1>",
                "<keyword 2>"
            ]
        }}
        ```
        The content of the article is as follows:
    {content}
"#,
        content = content
    )
}

/// 根据提供的内容提取文章思维导图。
///
/// 该函数接受一个 `String` 类型的参数 `content`，表示文章或新闻的内容。
/// 它将内容格式化为遵循特定规则的 Markdown 字符串，并返回格式化后的字符串。
/// Markdown 字符串包含从文章中提取的客观标题、内容块和关键信息点。
///
/// # 参数
///
/// * `content` - 表示文章或新闻的 `String` 类型参数。
///
/// # 返回值
///
/// 包含格式化的 Markdown 字符串的 `String` 类型，表示文章思维导图。
pub fn extract_article_mindmap(content: String) -> String {
    format!(
        r#"
你现在需要化身为一位拥有丰富阅读经验的资深文章摘要提取专家，您的使命在于帮助我快速且完整地阅读文章或新闻的内容。

请对下文中的文章或新闻进行深度且细致地解读，然后通过仔细阅读下列规则，深刻且完整地理解我所希望的输出内容：

```
- 输出内容的主要语言， 例如: en 等

- 输出内容使用markdown格式，包含了一个一级标题，若干个二级标题，若干个三级标题，若干个列表项

- 请根据文章的主要内容提取出文章的客观标题, 使用一级标题
标题的字数最好不要超过20字<建议>
标题的描述应该客观，避免标题党

- 根据文章的内容分布，提取出不少于两个内容块 使用二级标题/三级标题
每个内容块内的内容应该是连贯的，不应该出现跳跃的情况

- 每个内容块中提取出不少于两个关键信息点
关键信息点应该是内容块的核心内容，不应该是无关紧要的内容
关键信息点应该是连贯、精炼的，不应该出现跳跃、重复的情况
**每个关键信息点的字数不超过70字 <如果摘抄自原文，忽略这条限制>**
```

请你按照如下格式输出JSON字符串：
```
{{
    "language": <Language>,
    "mind_map": <markdown 格式的导读内容>
}}
```

原文如下:
```
{content}
```

当你了解完上述规则后：**请严格按照markdown格式输出**
输出内容:

        "#,
        content = content
    )
}
