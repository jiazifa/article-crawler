import ky from "ky";

// get BASE_URL From Ent
const BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3000";

const PREFIX = `${BASE_URL}`;
const buildServerURL = (path: string): string => {
  return `${PREFIX}/${path}`;
};

const serverAPI = ky.create({
  prefixUrl: PREFIX,
  headers: {
    "Content-Type": "application/json",
  },
  hooks: {
    beforeRequest: [
      (request) => {
        if (request.headers.get("Authorization")) {
          return;
        }
        const token = localStorage.getItem("token");
        if (token) {
          request.headers.set("Authorization", `Bearer ${token}`);
        }
        console.log(`[Ky]: ${request.method} ${request.url}`);
      },
    ],
  },
});

const normalAPI = ky.create({});

const parserServerResponse = async <T>(
  response: Response
): Promise<APIResponse<T>> => {
  const data = await response.json();
  return data;
};

export type APIResponse<T> = {
  data?: T;
  context: {
    code: number;
    message: string;
    server_at: string;
  };
};

export { serverAPI, normalAPI, parserServerResponse, buildServerURL };
