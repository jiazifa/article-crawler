import { NavSecton } from "@/components/Nav";
import { CircleGauge } from "lucide-react";


// 获得分类列表
export const Routes: NavSecton[] = [
    {
        "items": [
            {
                "title": "最新",
                "href": "/newest",
                icon: <CircleGauge className="h-4 w-4" />
            },
        ]
    }
]

