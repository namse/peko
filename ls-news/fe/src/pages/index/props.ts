// Auto-generated from src/pages/index/mod.rs

import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
  z.object({
    t: z.literal("Ok"),
    posts: z.array(
      z.object({
        id: z.string(),
        title: z.string(),
        url: z.string(),
        content: z.string(),
        author_id: z.string(),
        likes: z.number(),
        dislikes: z.number(),
        created_at: z.coerce.date(),
        updated_at: z.coerce.date(),
      })
    ),
  }),
  z.object({
    t: z.literal("DbErr"),
    message: z.string(),
  }),
]);

export type Props = z.infer<typeof PropsSchema>;
