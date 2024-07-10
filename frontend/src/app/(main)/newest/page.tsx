'use client';

import { Heading } from "@/components/ui/heading";
import { QueryFeedLinkRequest, QuerySubscriptionRequest, useFeedLinkList, useSubscriptionList } from "@/features/rss/service";
import { PageRequest } from "@/types";
import Image from "next/image";
import { useState } from "react";


export default function Home() {
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState(10);

    // 查找今日的内容
    const today = new Date();
    const todayStart = new Date(today.getFullYear(), today.getMonth(), today.getDate());

    const page: PageRequest = {
        page_size: pageSize,
        page: currentPage,
    };
    const option: QuerySubscriptionRequest = {
        page: page
    };

    const linkOption: QueryFeedLinkRequest = {
        published_at_lower: todayStart.getTime() / 1000,
        page: {
            page_size: 10,
            page: 1
        }
    }

    const { data: subscriptionResp, error: subsError, mutate: subsMutate } = useSubscriptionList(option);
    const { data: linkResp, error: linkError, mutate: linkMutate } = useFeedLinkList(linkOption);

    if (subsError) {
        return <div>failed to load{JSON.stringify(subsError)}</div>;
    }

    if (subscriptionResp === undefined) {
        return <div>loading...</div>;
    }
    const subscriptionPage = subscriptionResp.data;
    if (!subscriptionPage) {
        return <div>loading...</div>;
    }

    return (
        <>
            <div className="flex-1 space-y-4 p-4 pt-6 md:p-8">
                <div className="flex items-start justify-between">
                    <Heading title={`Kanban`} description="Manage tasks by dnd" />
                    {/* <NewTaskDialog /> */}
                </div>

                {linkResp?.data.length}
            </div>

        </>
    );
}
