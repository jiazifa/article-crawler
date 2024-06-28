import { NavItem } from "@/types/nav";


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
export const routes: NavItem[] = [
    {
        "title": "最新",
        "href": NEWEST_PAGE,
        icon: 'gauge'
    },
]

