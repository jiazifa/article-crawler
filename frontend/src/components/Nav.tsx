'use client'

import { usePathname } from 'next/navigation'
import React, { FC } from "react";
import Link from "next/link"


export type NavItem = {
    title: string;
    href: string;
    icon?: React.ReactNode;
};

type ItemBuilderProps = NavItem & {
    active: boolean;
};

export type NavSecton = {
    title?: string;
    items: NavItem[];
};

export type NavProps = {
    sections: NavSecton[];
};

const ItemBuilder: FC<ItemBuilderProps> = ({ title, href, icon, active }): React.ReactNode => {
    var classes = "flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary";
    if (active) {
        classes = "flex items-center gap-3 rounded-lg bg-muted px-3 py-2 text-primary transition-all hover:text-primary";
    }
    return (
        <Link
            href={href}
            className={classes}
        >
            {icon}
            {title}
        </Link>
    )
}

const NavContainer: FC<NavProps> = ({ sections }) => {
    // 获得当前页面的链接

    const pathname = usePathname()

    return (
        <nav className="grid items-start px-2 text-sm font-medium lg:px-4">
            {sections.map((section) => (
                <div key={section.title}>
                    {section.title && <h2 className="text-muted-foreground text-xs font-semibold uppercase tracking-wider">{section.title}</h2>}
                    {section.items.map((item) => (
                        <ItemBuilder key={item.href} title={item.title} href={item.href} icon={item.icon} active={item.href === pathname} />
                    ))}
                </div>
            ))}
        </nav>
    );
}

export { NavContainer };