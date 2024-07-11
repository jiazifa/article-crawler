// provider.tsx
'use client';
import React from 'react';
import { ThemeProvider } from './ThemeProvider';
import { SWRConfig, useSWRConfig } from 'swr';
import { SessionProvider, SessionProviderProps } from 'next-auth/react';

export function Providers({
  session,
  children
}: {
  session?: SessionProviderProps['session'];
  children: React.ReactNode;
}) {
  return (
    <>
      <SWRConfig
        value={{
          dedupingInterval: 100,
          refreshInterval: 0,
          revalidateOnFocus: true,
          shouldRetryOnError: false,
          errorRetryInterval: 10000,
          errorRetryCount: 3,
          refreshWhenHidden: false,
          refreshWhenOffline: false,
        }}
      >
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
          <SessionProvider session={session}>{children}</SessionProvider>
        </ThemeProvider>
      </SWRConfig>
    </>
  );
}
