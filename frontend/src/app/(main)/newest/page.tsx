'use client';

import { AppTitleContainer } from "@/components/AppTitleContainer";
import { QuerySubscriptionRequest, useSubscriptionList } from "@/service/rss_service";
import { PageRequest } from "@/types";
import Image from "next/image";
import { useState } from "react";


export default function Home() {
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState(10);

    const page: PageRequest = {
        page_size: pageSize,
        page: currentPage,
    };
    const option: QuerySubscriptionRequest = {
        page: page
    };

    const { data: subscriptionResp, error: subsError, mutate: subsMutate } = useSubscriptionList(option);

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
            <AppTitleContainer title="最新" />
            {subscriptionPage && `${subscriptionPage.length} 条数据`}
        </>
    );
}
