'use client';

import Link from "next/link"
import { zodResolver } from "@hookform/resolvers/zod"
import { Button } from "@/components/ui/button"
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card"
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { z } from 'zod';
import { toast } from "sonner"
import { useForm } from "react-hook-form";
import { AUTHENTICATION_APP, MAIN_APP } from "@/utils/routes"
import { APIResponse, parserServerResponse, serverAPI } from "@/utils";
import { useRouter } from "next/navigation";

const formSchema = z.object({
    email: z.string().email(),
    password: z.string().min(3, {
        message: "Password must be at least 3 characters long",
    }),
});

export default function LoginForm() {
    const router = useRouter();
    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            email: "test@email.com",
            password: "123456",
        },
    })

    const onSubmitAction = async (value: z.infer<typeof formSchema>) => {
        try {
            const respOrigin = await serverAPI.post("account/login", { json: value });

            const resp = await parserServerResponse(respOrigin);
            console.log(`resp: ${JSON.stringify(resp)}`)
            const jwt: string = resp.data?.token;
            localStorage.setItem("token", jwt);
            toast.success("Account created successfully")
            router.push(MAIN_APP.RssNewest);
        } catch (error) {
            toast.error(`Failed to create account: ${error}`)
        }
    };

    return (
        <Card className="mx-auto max-w-sm" >
            <CardHeader>
                <CardTitle className="text-xl">登录</CardTitle>
                <CardDescription>
                    输入账号密码登录
                </CardDescription>
            </CardHeader>
            <CardContent>
                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmitAction)}>

                        <FormField
                            control={form.control}
                            name="email"
                            render={({ field }) => (
                                <FormItem className="mt-2">
                                    <FormLabel htmlFor="email">Email</FormLabel>
                                    <FormControl>
                                        <Input {...field}
                                        />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <FormField
                            control={form.control}
                            name="password"
                            render={({ field }) => (
                                <FormItem className="mt-2">
                                    <FormLabel htmlFor="password">Password</FormLabel>
                                    <FormControl>
                                        <Input id="password" type="password" {...field} />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />


                        <Button type="submit" className="w-full mt-3">
                            登录
                        </Button>

                        <Link className="mt-2" href={AUTHENTICATION_APP.SignUp}>
                            还没有账号? 注册
                        </Link>
                    </form>
                </Form>

            </CardContent>
        </Card >
    )
}
