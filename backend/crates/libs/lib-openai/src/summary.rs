use std::marker::PhantomData;

use crate::{prompt, schema::LinkSummary, LinkMindMap};
pub use async_openai::config::{Config, OpenAIConfig};
use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionFunctionCall, ChatCompletionFunctionsArgs,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
        ChatCompletionResponseFormat, ChatCompletionToolArgs, ChatCompletionToolType,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse, FunctionObjectArgs, Role,
    },
    Client,
};

pub struct AISummaryController<C: Config> {
    _config: Option<C>,
}

impl<C: Config> AISummaryController<C> {
    pub fn new(config: C) -> Self {
        Self {
            _config: Some(config),
        }
    }

    pub fn no_config() -> Self {
        Self { _config: None }
    }

    pub fn update_config(&mut self, config: C) {
        self._config = Some(config);
    }
}

impl<C: Config> AISummaryController<C> {
    fn prefer_model_for_context_size(&self, messages: &[ChatCompletionRequestMessage]) -> String {
        let model_context_mapping: &[(&str, usize)] = &[
            ("gpt-3.5-turbo-1106", 3000),  // 4k
            ("gpt-3.5-turbo-1106", 14000), // 16k
            ("gpt-4-32k-0613", 28000),     // 32k
        ];
        let mut prefer_model = "gpt-3.5-turbo-1106";
        for (model, context_size) in model_context_mapping {
            prefer_model = model;
            let num_tokens = self.num_tokens_with_messages(model, messages).unwrap_or(0);
            if num_tokens < *context_size {
                break;
            }
        }
        prefer_model.to_string()
    }

    pub fn num_tokens_with_content(
        &self,
        model: Option<String>,
        content: String,
    ) -> anyhow::Result<usize> {
        let model = model.unwrap_or("gpt-3.5-turbo".to_string());
        // build a user message
        let messages = [ChatCompletionRequestUserMessageArgs::default()
            .content(
                async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    content.to_string(),
                ),
            )
            .build()?
            .into()];
        self.num_tokens_with_messages(&model, &messages)
    }

    pub fn num_tokens_with_messages(
        &self,
        model: &str,
        messages: &[ChatCompletionRequestMessage],
    ) -> anyhow::Result<usize> {
        let tokenizer = tiktoken_rs::tokenizer::get_tokenizer(model)
            .ok_or(anyhow::anyhow!("Model {} not supported", model))?;
        if tokenizer != tiktoken_rs::tokenizer::Tokenizer::Cl100kBase {
            anyhow::bail!("Chat completion is only supported chat models")
        }
        let bpe = tiktoken_rs::get_bpe_from_tokenizer(tokenizer)?;

        let (tokens_per_message, tokens_per_name) = if model.starts_with("gpt-3.5") {
            (
                4,  // every message follows <im_start>{role/name}\n{content}<im_end>\n
                -1, // if there's a name, the role is omitted
            )
        } else {
            (3, 1)
        };

        let mut num_tokens: i32 = 0;
        for message in messages {
            match message {
                ChatCompletionRequestMessage::User(msg) => {
                    let mut content = match msg.content.clone() {
                        async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                            text,
                        ) => text,
                        async_openai::types::ChatCompletionRequestUserMessageContent::Array(
                            vec,
                        ) => {
                            // 如果是字符串，则拼接后返回，如果包含图片，则过滤
                            let mut content = String::new();
                            for item in vec {
                                match item {
                                async_openai::types::ChatCompletionRequestMessageContentPart::Text(txt) => {
                                    content.push_str(txt.text.as_str());
                                },
                                async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(_) => {
                                    // ignore
                                },
                            }
                            }
                            content
                        }
                    };

                    num_tokens += tokens_per_message;
                    num_tokens += bpe.encode_with_special_tokens(&content).len() as i32;
                    if let Some(name) = &msg.name {
                        num_tokens += bpe.encode_with_special_tokens(name).len() as i32;
                        num_tokens += tokens_per_name;
                    }
                }
                ChatCompletionRequestMessage::System(msg) => {
                    num_tokens += tokens_per_message;
                    num_tokens += bpe.encode_with_special_tokens(&msg.content.clone()).len() as i32;
                }
                ChatCompletionRequestMessage::Assistant(msg) => {
                    num_tokens += tokens_per_message;
                    num_tokens += bpe
                        .encode_with_special_tokens(&msg.content.clone().unwrap_or_default())
                        .len() as i32;
                    if let Some(name) = &msg.name {
                        num_tokens += bpe.encode_with_special_tokens(name).len() as i32;
                    }
                }
                ChatCompletionRequestMessage::Function(msg) => {
                    num_tokens += tokens_per_message;
                    num_tokens += bpe
                        .encode_with_special_tokens(&msg.content.clone().unwrap_or_default())
                        .len() as i32;
                }
                ChatCompletionRequestMessage::Tool(msg) => {
                    num_tokens += tokens_per_message;
                    num_tokens += bpe.encode_with_special_tokens(&msg.content.clone()).len() as i32;
                    num_tokens += bpe
                        .encode_with_special_tokens(&msg.tool_call_id.to_string())
                        .len() as i32;
                }
            }
        }
        num_tokens += 3; // every reply is primed with <|start|>assistant<|message|>
        Ok(num_tokens as usize)
    }
}

impl<C: Config> AISummaryController<C> {
    pub async fn generate_summary(&self, content: String) -> Result<LinkSummary, anyhow::Error> {
        let config = match &self._config {
            Some(config) => config,
            None => return Err(anyhow::anyhow!("config not found")),
        };
        // calculate the token cost of content
        let messages = [ChatCompletionRequestSystemMessageArgs::default()
            .content(prompt::generate_article_summary(content.to_string()))
            .build()?
            .into()];

        let prefer_model = self.prefer_model_for_context_size(&messages);
        let client = Client::with_config(config.clone());
        let request = CreateChatCompletionRequestArgs::default()
            .response_format(ChatCompletionResponseFormat {
                r#type: async_openai::types::ChatCompletionResponseFormatType::JsonObject,
            })
            .model(prefer_model.clone())
            .messages(messages)
            .build()?;
        let resp = client.chat().create(request).await?;

        let raw_json_str = resp
            .choices
            .first()
            .ok_or(anyhow::anyhow!("openai generate article summary failed"))?
            .message
            .content
            .clone()
            .unwrap_or("".to_string());
        let mut resp = serde_json::from_str::<LinkSummary>(&raw_json_str)?;
        resp.provider = Some(prefer_model);
        Ok(resp)
    }

    /// 根据文章内容提取出导读内容， 将文章内容结构化整理
    pub async fn article_mindmap(&self, content: String) -> Result<LinkMindMap, anyhow::Error> {
        let config = match &self._config {
            Some(config) => config,
            None => return Err(anyhow::anyhow!("config not found")),
        };
        let messages = [ChatCompletionRequestUserMessageArgs::default()
            .content(prompt::extract_article_mindmap(content.to_string()))
            .build()?
            .into()];
        let prefer_model = self.prefer_model_for_context_size(&messages);
        let client = Client::with_config(config.clone());

        let request = CreateChatCompletionRequestArgs::default()
            .response_format(ChatCompletionResponseFormat {
                r#type: async_openai::types::ChatCompletionResponseFormatType::JsonObject,
            })
            .model(prefer_model)
            .messages(messages)
            .build()?;
        let resp = client.chat().create(request).await?;

        let raw_json_str = resp
            .choices
            .first()
            .ok_or(anyhow::anyhow!("openai generate article summary failed"))?
            .message
            .content
            .clone()
            .unwrap_or("".to_string());
        let resp = serde_json::from_str::<LinkMindMap>(&raw_json_str)?;
        Ok(resp)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_config() -> Option<OpenAIConfig> {
        if dotenv::dotenv().is_err() {
            return None;
        }
        if let (Ok(api_key), Ok(api_base)) = (
            std::env::var("OPENAI_API_KEY"),
            std::env::var("OPENAI_API_BASE"),
        ) {
            Some(
                OpenAIConfig::default()
                    .with_api_base(api_base)
                    .with_api_key(api_key),
            )
        } else {
            None
        }
    }

    #[test]
    fn test_token() {
        let model = "gpt-3.5-turbo";
        let tokenizer = tiktoken_rs::tokenizer::get_tokenizer(model).unwrap();
        let bpe = tiktoken_rs::get_bpe_from_tokenizer(tokenizer).unwrap();
        let content = "hello world";
        let tokens = bpe.encode_with_special_tokens(content);
        assert!(!tokens.is_empty());
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_token_2() {
        let model = "gpt-3.5-turbo";
        let messages = [ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "hello word".to_string(),
                ),
                name: None,
            },
        )];

        let controller: AISummaryController<_> = AISummaryController::<OpenAIConfig>::no_config();
        let token = controller
            .num_tokens_with_messages(model, &messages)
            .unwrap();
        assert_eq!(token, 9);
    }

    #[test]
    fn test_prefer_model_for_context_size() {
        let messages = [ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "hello word".to_string(),
                ),
                name: None,
            },
        )];
        let controller: AISummaryController<_> = AISummaryController::<OpenAIConfig>::no_config();
        let model = controller.prefer_model_for_context_size(&messages);
        assert_eq!(model, "gpt-3.5-turbo");

        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();
        for _ in 0..1000 {
            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "hello word".to_string(),
                    ),
                    name: None,
                },
            ));
        }
        let model = controller.prefer_model_for_context_size(&messages);
        assert_eq!(model, "gpt-3.5-turbo-1106");

        for _ in 0..3000 {
            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        "hello word".to_string(),
                    ),
                    name: None,
                },
            ));
        }
        let model = controller.prefer_model_for_context_size(&messages);
        assert_eq!(model, "gpt-4-32k-0613");
    }

    #[tokio::test]
    async fn test_summary_article() {
        let config = match get_config() {
            Some(config) => config,
            None => return,
        };
        let controller: AISummaryController<_> = AISummaryController::<OpenAIConfig>::new(config);

        let summary = controller.generate_summary(r#"
        "轻舟计划"是什么？
"轻舟计划"源于"轻舟已过万重山"，是平台最新推出的二次分销计划，也是专为小规模开发者量身定制的快速起步的计划。

在互联网时代，所有的商业尝试都应该够快、发挥极致的效率。当你有一个AI领域的想法时，你不应该在还没开始挣钱的时候，就把时间花在开发最基础的用户体系、账单体系、计费体系等上，而是应该专注于你的核心产品和竞争力，快速上线、快速迭代、快速验证，用最小的开发成本，迅速验证自己的商业想法。当业务成熟稳定下来，值得大力投入后，再回过头来完善用户体系等一系列基础能力。

"轻舟计划"重在一个"轻"，即辅助开发者快速完成产品验证到大力投入之间的空白区域，当别人还在埋头开发用户管理等基础体系时，您的产品已快速上线，抢占市场，积累起一批核心用户。当时机成熟时，您就可以回过头来，慢慢完善用户体系等基础能力，此时"轻舟计划"使命完成，便可以退出舞台。

如何参与"轻舟计划"？
"轻舟计划"的主体就是平台新推出的"子账户"能力，可以快速生成一批定额的子账户API Key去分销给你的客户使用，目前参与轻舟计划要求通过本站专业开发者认证，未认证用户只能创建少量子账户进行功能体验。

假设你开发了一个AI领域的软件或网站，给某类特定人群使用：

你的第一步就是将你的软件或网站的OpenAI API接口指向https://api.openai-proxy.org。
在平台生成一批有余额的API Key，设置好每个Key的定价，利用淘宝店铺或者其他分销体系，进行销售，无需任何开发工作，让您把精力重点放在运营和推广上。
用户拿到API Key后，开始在你的平台使用和消费，后续充值也是继续找您的销售体系充值。
如此，便快速完成了初始的商业逻辑验证，完全不需要开发用户体系、计费体系、账单体系等基础能力，零开发成本快速开始您的商业计划。
        "#.to_string()).await.unwrap();

        assert!(!summary.summary.is_empty());
        assert!(!summary.key_points.is_empty());
        assert!(!summary.action_items.is_empty());
        assert!(!summary.keywords.is_empty());
    }

    #[tokio::test]
    async fn test_mind_map_article() {
        let config = match get_config() {
            Some(config) => config,
            None => return,
        };
        let controller: AISummaryController<_> = AISummaryController::<OpenAIConfig>::new(config);
        let mind_map = controller.article_mindmap(r#"
        "轻舟计划"是什么？
"轻舟计划"源于"轻舟已过万重山"，是平台最新推出的二次分销计划，也是专为小规模开发者量身定制的快速起步的计划。

在互联网时代，所有的商业尝试都应该够快、发挥极致的效率。当你有一个AI领域的想法时，你不应该在还没开始挣钱的时候，就把时间花在开发最基础的用户体系、账单体系、计费体系等上，而是应该专注于你的核心产品和竞争力，快速上线、快速迭代、快速验证，用最小的开发成本，迅速验证自己的商业想法。当业务成熟稳定下来，值得大力投入后，再回过头来完善用户体系等一系列基础能力。

"轻舟计划"重在一个"轻"，即辅助开发者快速完成产品验证到大力投入之间的空白区域，当别人还在埋头开发用户管理等基础体系时，您的产品已快速上线，抢占市场，积累起一批核心用户。当时机成熟时，您就可以回过头来，慢慢完善用户体系等基础能力，此时"轻舟计划"使命完成，便可以退出舞台。

如何参与"轻舟计划"？
"轻舟计划"的主体就是平台新推出的"子账户"能力，可以快速生成一批定额的子账户API Key去分销给你的客户使用，目前参与轻舟计划要求通过本站专业开发者认证，未认证用户只能创建少量子账户进行功能体验。

假设你开发了一个AI领域的软件或网站，给某类特定人群使用：

你的第一步就是将你的软件或网站的OpenAI API接口指向https://api.openai-proxy.org。
在平台生成一批有余额的API Key，设置好每个Key的定价，利用淘宝店铺或者其他分销体系，进行销售，无需任何开发工作，让您把精力重点放在运营和推广上。
用户拿到API Key后，开始在你的平台使用和消费，后续充值也是继续找您的销售体系充值。
如此，便快速完成了初始的商业逻辑验证，完全不需要开发用户体系、计费体系、账单体系等基础能力，零开发成本快速开始您的商业计划。
        "#.to_string()).await.unwrap();
        println!("{:?}", mind_map);
        assert!(mind_map.mind_map.len() > 10);
        assert_eq!(mind_map.language, Some("zh".to_string()));
    }
}
