import ky from "ky";

const PREFIX = "http://localhost:10123/8bd86ee69";
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
    msg: string;
    server_at: string;
  };
};

export { serverAPI, normalAPI, parserServerResponse, buildServerURL };
