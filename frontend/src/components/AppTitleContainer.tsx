import { FC } from "react";

export type AppTitleContainerProps = Readonly<{
    title: string;
}>;

const AppTitleContainer: FC<AppTitleContainerProps> = ({ title }) => {
    return (
        <div className="flex items-center">
            <h1 className="text-lg font-semibold md:text-2xl">{title}</h1>
        </div>
    )
};

export { AppTitleContainer };