import { NavSecton } from "@/components/Nav";
import { CircleGauge } from "lucide-react";


function path(root: string, sublink: string) {
    return `${root}${sublink}`;
}

const NEWEST_PAGE = "/newest";

const AUTHENTICATION_ROOT = "/auth";

export const AUTHENTICATION_APP = {
    SignIn: path(AUTHENTICATION_ROOT, "/signin"),
    SignUp: path(AUTHENTICATION_ROOT, "/signup"),
}

export const MAIN_APP = {
    RssNewest: NEWEST_PAGE,
}

// 获得分类列表
export const Routes: NavSecton[] = [
    {
        "items": [
            {
                "title": "最新",
                "href": NEWEST_PAGE,
                icon: <CircleGauge className="h-4 w-4" />
            },
        ]
    }
]

