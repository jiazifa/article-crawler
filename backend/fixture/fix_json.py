import os
import json

feed_dir = "fixture/rss/feeds"

# load .json fiels in dir
files = os.listdir(feed_dir)
# filter extension by .json
files = list(filter(lambda x: x.endswith('.json'), files))
# {
#     "accentColor": "",
#     "articlesCountForThisWeek": 545,
#     "categoryId": 1001,
#     "completed": true,
#     "customTitle": "BBC News",
#     "description": "Visit BBC News for up-to-the-minute news, breaking news, video, audio and feature stories. BBC News provides trusted World and UK news as well as local and regional perspectives. Also entertainment, business, science, technology and health news.",
#     "feedId": "feed/http://feeds.bbci.co.uk/news/rss.xml",
#     "iconUrl": "https://storage.googleapis.com/site-assets/KqlRbklUF62RRb5xGKl1_MA8VbYW2S9f567fx9QP22A_visual-180f79b8d54",
#     "id": 100100001,
#     "language": "en",
#     "lastUpdated": 1700062204417,
#     "logo": "",
#     "score": 1,
#     "sortOrder": 1,
#     "subscribedTime": -1,
#     "subscribers": 21630,
#     "title": "BBC News",
#     "updated": 1700062204417,
#     "velocity": 540.0999755859375,
#     "visualUrl": "",
#     "website": "https://www.bbc.co.uk/news/"
#   },

# extract keys from above
remin_keys: list[str] = ["accentColor", "categoryId", "customTitle", "description", 
                         "iconUrl", "language",
                         "sortOrder", 
                         "title", "website"]

def fix_json(value: dict[str, str]) -> dict:
    new_value = dict()
    for k in value:
        if k in remin_keys:
            new_value[k] = value[k]
        if k == "feedId":
            new_value["feedUrl"] = value[k].removeprefix("feed/")
    return new_value

def map_json(file_path: str) -> dict:
    with open(file_path, 'r') as f:
        file_content = f.read()
    datas: list[dict] = json.loads(file_content)
    new_datas = list(map(fix_json, datas))
    
    with open(f"{file_path}", 'w') as f:
        json.dump(new_datas, f, indent=4)
        
if __name__ == "__main__":
    for file in files:
        map_json(f"{feed_dir}/{file}")