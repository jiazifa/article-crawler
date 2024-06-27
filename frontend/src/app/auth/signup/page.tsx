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
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"

import { toast } from "sonner"
import { z } from 'zod';
import { useForm } from "react-hook-form";
import { AUTHENTICATION_APP } from "@/utils/routes"
import { parserServerResponse, serverAPI } from "@/utils";
import { useRouter } from "next/navigation";

const formSchema = z.object({
    email: z.string().email("Invalid email address").min(1, "Email cannot be blank"),
    password: z.string().min(3, {
        message: "Password must be at least 3 characters long",
    }),
    passwordConfirm: z.string()

}).refine(({ email, password, passwordConfirm }) => {
    return password === passwordConfirm;
}, { message: "Passwords do not match", path: ["passwordConfirm"] });

export default function RegisterForm() {
    const router = useRouter();
    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            email: "test@email.com",
            password: "123456",
            passwordConfirm: "123456"
        },
    })

    const onSubmitAction = async (value: z.infer<typeof formSchema>) => {
        try {
            const respOrigin = await serverAPI.post("account/register", { json: value });
            const resp = await parserServerResponse(respOrigin);

            console.log(`resp: ${JSON.stringify(resp)}`)
            toast.success("Account created successfully")
            // router.push(AUTHENTICATION_APP.SignIn)

        } catch (error) {
            toast.error(`Failed to create account: ${error}`)
        }
    };

    return (
        <Card className="mx-auto max-w-sm" >
            <CardHeader>
                <CardTitle className="text-xl">Sign Up</CardTitle>
                <CardDescription>
                    Enter your information to create an account
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
                                        <Input
                                            {...field}
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

                        <FormField
                            control={form.control}
                            name="passwordConfirm"
                            render={({ field }) => (
                                <FormItem className="mt-2">
                                    <FormLabel htmlFor="passwordConfirm">Confirm Password</FormLabel>
                                    <FormControl>
                                        <Input id="passwordConfirm" type="password" {...field} />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <Button type="submit" className="w-full mt-3">
                            Create an account
                        </Button>
                        <Link className="mt-2" href={AUTHENTICATION_APP.SignIn}>
                            Already have an account? Sign in
                        </Link>
                    </form>
                </Form>

            </CardContent>
        </Card >
    )
}
