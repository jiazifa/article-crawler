'use client';

import { AUTHENTICATION_APP, MAIN_APP } from "@/utils/routes";
import { getServerSession } from "next-auth";
import { useRouter } from 'next/navigation';

export default function Home() {
  const router = useRouter();
  const token_storage = localStorage.getItem("token");
  const no_token = !token_storage;
  if (no_token) {
    router.push(AUTHENTICATION_APP.SignIn)
  } else {
    router.push(MAIN_APP.RssNewest)
  }
  return null;
}
