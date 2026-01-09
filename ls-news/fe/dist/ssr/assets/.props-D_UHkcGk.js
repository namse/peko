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
const UserSchema = z.object({
  id: z.string(),
  username: z.string(),
  avatarUrl: z.string(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date()
});
const RowSchema = z.object({
  post: PostSchema,
  deletedAt: z.coerce.date().optional(),
  author: UserSchema
});
const PropsSchema = z.discriminatedUnion("t", [
  z.object({
    t: z.literal("Ok"),
    rows: z.array(RowSchema)
  }),
  z.object({
    t: z.literal("DbErr"),
    message: z.string()
  })
]);
export {
  PostSchema,
  PropsSchema,
  RowSchema,
  UserSchema
};
