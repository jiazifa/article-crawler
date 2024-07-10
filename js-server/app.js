
import Parser from "@postlight/parser";
import Fastify from "fastify";
import lruCache from "fastify-lru-cache";
// import { htmlToText } from 'html-to-text';
import TurndownService from 'turndown';

const fastify = Fastify({ logger: true });
var turndownService = TurndownService({headingStyle: 'atx', codeBlockStyle: 'fenced'});

fastify.register(lruCache, {
    // 配置缓存的大小
    max: 10000,
    ttl: 60 * 60 * 12,
});

fastify.get("/", async (_request, _reply) => {
    return { hello: "world" };
    }
);

// add a health check
fastify.get("/health/check", async (_request, _reply) => {
    return { status: "ok" };
});

// 定义 Schema
const schema = {
    body: {
        type: "object",
        properties: {
            url: { type: "string" },
            ignore_cache: { type: "boolean", default: false },
        },
        required: ["url"],
    },
};

// 定义一个解析的接口
// 解析后的内容是纯文本的
fastify.post("/parse", { schema }, async (request, _reply) => {
    const { url, ignore_cache } = request.body;
    console.log(`got url: ${url}`)
    // 从缓存中获取
    const cached = fastify.cache.get(url);
    if (cached) {
        return cached;
    }
    // 配置超时时间, 以及其他推荐配置
    const result = await Parser.parse(url, { timeout: 3000, maxRedirects: 3 });
    // 解构出需要的字段
    const {
        title, author, date_published, 
        dek, lead_image_url, content, 
        next_page_url,  url: self_url, 
        domain, excerpt, word_count, 
        direction, total_pages, 
        rendered_pages
    } = result;
    // 构造返回值
    // const txt_content = htmlToText(content, {wordwrap: 130});
    const resp = {
        title, author, date_published, 
        dek, lead_image_url, content, 
        next_page_url, url: self_url, 
        domain, excerpt, word_count, 
        direction, total_pages, 
        rendered_pages
    };
    // 设置缓存
    if (!ignore_cache) {
        fastify.cache.set(url, resp);
    }
    return  resp;
});

// 定义一个 markdown 解析的接口
fastify.post("/parse/md", {schema}, async (request, _reply) => {
    const { url } = request.body;
    // 从缓存中获取
    if (!fastify.cache.has(url)) {
        // 转发到 /parse 获得结果
        await fastify.inject({
            method: 'POST',
            url: '/parse',
            payload: {url}
        });
    }
    const cached = fastify.cache.get(url);
    // 解析出需要的字段
    const content = cached.content;
    // html to markdown
    const txt_content = turndownService.turndown(content);
    // content 字段替换为 markdown, 其余部分不变
    const resp = {
        ...cached,
        content: txt_content
    };
    return resp;
});


fastify.listen({port: 5012, host: '0.0.0.0'}, (err, address) => {
    if (err) {
        fastify.log.error(`got error: ${err}`);
        process.exit(1);
    }
    // Server is now listening on ${address}
    console.log(`Server is now listening on ${address}`);
});
