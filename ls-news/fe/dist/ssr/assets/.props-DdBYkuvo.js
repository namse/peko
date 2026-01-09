import { z } from "zod";
const PostSchema = z.object({
  id: z.string(),
  title: z.string(),
  url: z.string(),
  content: z.string(),
  authorId: z.string(),
  likes: z.number(),
  dislikes: z.number(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date()
});
const CommentSchema = z.object({
  id: z.string(),
  content: z.string(),
  postId: z.string(),
  authorId: z.string(),
  parentCommentId: z.string().optional(),
  likes: z.number(),
  dislikes: z.number(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date()
});
const UserSchema = z.object({
  id: z.string(),
  username: z.string(),
  avatarUrl: z.string(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date()
});
const PropsSchema = z.discriminatedUnion("t", [
  z.object({
    t: z.literal("Ok"),
    post: PostSchema,
    comments: z.array(CommentSchema),
    users: z.record(z.string(), UserSchema)
  }),
  z.object({
    t: z.literal("NotFound")
  }),
  z.object({
    t: z.literal("DbErr"),
    message: z.string()
  })
]);
export {
  CommentSchema,
  PostSchema,
  PropsSchema,
  UserSchema
};
