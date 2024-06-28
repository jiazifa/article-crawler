import { ResponseAccountAndToken } from "@/types/account";
import { APIResponse, parserServerResponse, serverAPI } from "@/utils";
import { AUTHENTICATION_APP } from "@/utils/routes";
import NextAuth, { AuthOptions } from "next-auth";
import CredentialProvider from "next-auth/providers/credentials";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

const authConfig: AuthOptions = {
  providers: [
    CredentialProvider({
      name: "Credentials",
      credentials: {
        email: { label: "Email", type: "email" },
        password: { label: "Password", type: "password" },
      },
      async authorize(credentials, req) {
        console.log(
          `[AUTH] on Authorize: credentials: ${JSON.stringify(credentials)}`
        );
        if (credentials == undefined) {
          throw new Error("Credentials is undefined");
        }
        const value = {
          email: credentials.email,
          password: credentials.password,
        };
        try {
          const respOrigin = await fetch(`${BASE_URL}/account/login`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(value),
          });
          const resp: APIResponse<ResponseAccountAndToken> =
            await parserServerResponse(respOrigin);
          if (resp.data) {
            console.log(`[AUTH] on authorize: resp: ${JSON.stringify(resp)}`);
            const user = {
              id: resp.data.account.id.toString(),
              name: resp.data.account.nick_name,
              email: resp.data.account.email,
              token: resp.data.token,
            };
            console.log(`[AUTH] on authorize: user: ${JSON.stringify(user)}`);
            return user;
          } else if (resp.context.code !== 200) {
            throw new Error(resp.context.message);
          } else {
            throw new Error("Failed to login");
          }
        } catch (error) {
          console.error(`[AUTH] on authorize error: ${error}`);
          throw new Error("Failed to login");
        }
      },
    }),
  ],
  callbacks: {
    async jwt({ token, user }) {
      if (user) {
        token.id = user.id;
        token.name = user.name;
        token.email = user.email;
        token.token = user.token;
      }
      console.log(`[AUTH] on jwt callback: token: ${JSON.stringify(token)}`);
      return token;
    },
    async session({ session, token }) {
      session.token = token.token;
      console.log(
        `[AUTH] on session callback: session: ${JSON.stringify(session.token)}`
      );
      return session;
    },
  },
  pages: {
    signIn: AUTHENTICATION_APP.SignIn,
  },
};

export const handler = NextAuth(authConfig);
