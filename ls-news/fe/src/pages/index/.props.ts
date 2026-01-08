// Auto-generated from rs/src/pages/index/mod.rs

import { z } from "zod";

export const PostSchema = z.object({
  id: z.string(),
  title: z.string(),
  url: z.string(),
  content: z.string(),
  authorId: z.string(),
  likes: z.number(),
  dislikes: z.number(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date(),
});

export type Post = z.infer<typeof PostSchema>;

export const PropsSchema = z.discriminatedUnion("t", [
  z.object({
    t: z.literal("Ok"),
    posts: z.array(PostSchema),
  }),
  z.object({
    t: z.literal("DbErr"),
    message: z.string(),
  }),
]);

export type Props = z.infer<typeof PropsSchema>;
