'use client';

import { AUTHENTICATION_APP, MAIN_APP } from "@/utils/routes";
import { useSession } from "next-auth/react";
import { useRouter } from 'next/navigation';
import { useEffect } from "react";

export default function Home() {
  const router = useRouter();
  const { data: session, status } = useSession();

  useEffect(() => {
    async function checkSession() {
      if (session?.user === undefined) {
        router.push(AUTHENTICATION_APP.SignIn);
      } else {
        router.push(MAIN_APP.RssNewest);
      }
    }
    checkSession();
  }, [router, session?.user, status]);

  return null;
}