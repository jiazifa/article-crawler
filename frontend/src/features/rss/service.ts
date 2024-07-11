import useSWR from "swr";

import { APIResponse, parserServerResponse, serverAPI } from "@/utils/api";
import {
  Category,
  PageRequest,
  PageResponse,
  Subscription,
  Link,
} from "@/types";

export interface QuerySubscriptionRequest {
  ids?: number[];
  title?: string;
  description?: string;
  language?: string[];
  page: PageRequest;
}

export interface QueryCategoryRequest {
  ids?: number[];
  title?: string;
  description?: string;
  parent_ids?: number[];
  need_feed_logo_count?: number;
  page: PageRequest;
}

export interface QueryFeedLinkRequest {
  ids?: number[];
  title?: string;
  subscription_ids?: number[];
  published_at_lower?: number;
  published_at_upper?: number;
  page: PageRequest;
}

const category_fetcher = async (
  options: QueryCategoryRequest
): Promise<PageResponse<Category>> => {
  const resp = await serverAPI.post("feed/category/query", {
    json: options,
  });
  const respData: APIResponse<PageResponse<Category>> =
    await parserServerResponse(resp);
  if (respData.data === undefined) {
    throw new Error("Data is undefined");
  } else if (respData.context.code !== 200) {
    throw new Error(respData.context.message);
  }
  return respData.data;
};

const subscription_fetcher = async (
  options: QuerySubscriptionRequest
): Promise<PageResponse<Subscription>> => {
  const resp = await serverAPI.post("feed/subscrition/query", {
    json: options,
  });
  const respData: APIResponse<PageResponse<Subscription>> =
    await parserServerResponse(resp);
  if (respData.data === undefined) {
    throw new Error("Data is undefined");
  } else if (respData.context.code !== 200) {
    throw new Error(respData.context.message);
  }
  return respData.data;
};

const feedlink_fetcher = async (
  options: QueryFeedLinkRequest
): Promise<PageResponse<Link>> => {
  const resp = await serverAPI.post("feed/link/query", {
    json: options,
  });
  const respData: APIResponse<PageResponse<Link>> = await parserServerResponse(
    resp
  );

  console.log(`respData: ${JSON.stringify(respData)}`);
  if (respData.data === undefined) {
    throw new Error("Data is undefined");
  } else if (respData.context.code !== 200) {
    throw new Error(respData.context.message);
  }
  return respData.data;
};

export const useSubscriptionList = (options: QuerySubscriptionRequest) => {
  const params = new URLSearchParams();
  params.append("options", JSON.stringify(options));
  return useSWR<PageResponse<Subscription>>(
    `/subscription?${params.toString()}`,
    () => subscription_fetcher(options)
  );
};

export const useCategoryList = (options: QueryCategoryRequest) => {
  const params = new URLSearchParams();
  params.append("options", JSON.stringify(options));
  return useSWR<PageResponse<Category>>(`/category?${params.toString()}`, () =>
    category_fetcher(options)
  );
};

export const useFeedLinkList = (options: QueryFeedLinkRequest) => {
  const params = new URLSearchParams();
  params.append("options", JSON.stringify(options));
  return useSWR<PageResponse<Link>>(`/feedlink?${params.toString()}`, () =>
    feedlink_fetcher(options)
  );
};
